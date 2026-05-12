mod commands;
mod config;
mod discovery;
mod http;
mod tray;

use commands::auth::AuthState;
use tauri::{Emitter, Manager, Listener};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            // When a second instance is spawned (e.g. by a deep link on Windows/Linux),
            // the OS passes the URL as a CLI argument. Forward it to our handler.
            if let Some(url) = argv.iter().find(|a| a.starts_with("vinkius://")) {
                handle_deep_link(app, url.clone());
            }
        }))
        // Shared auth state — allows deep link handler to access pending device_code
        .manage(AuthState::default())
        .invoke_handler(tauri::generate_handler![
            // Auth
            commands::auth::get_session,
            commands::auth::get_access_token,
            commands::auth::login,
            commands::auth::logout,
            // Discovery
            commands::discover::discover_clients,
            // Servers
            commands::servers::list_servers,
            commands::servers::install_server,
            commands::servers::remove_server,
            commands::servers::sync_server,
            // Introspection
            commands::introspect::introspect_server,
            // Config
            commands::config::read_config,
            commands::config::check_health,
            // Settings
            commands::settings::get_settings,
            commands::settings::update_settings,
            // Tray
            tray::set_tray_visible,
        ])
        .setup(|app| {
            // Register deep link handler for macOS (receives URL directly)
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            {
                let handle = app.handle().clone();
                app.listen("deep-link://new-url", move |event| {
                    let payload = event.payload();
                    // payload is a JSON array of strings
                    if let Ok(parsed) = serde_json::from_str::<Vec<String>>(payload) {
                        if let Some(url) = parsed.into_iter().find(|u| u.starts_with("vinkius://")) {
                            handle_deep_link(&handle, url);
                        }
                    }
                });
            }

            // Set up system tray
            tray::create_tray(app)?;

            // Start file watcher for config changes
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                if let Err(e) = config::watcher::start_watching(app_handle) {
                    log::error!("File watcher failed: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Vinkius Desktop");
}

// ── Deep Link Handler ────────────────────────────────────────────────

/// Handle incoming `vinkius://` deep link URLs.
///
/// Supported routes:
///   - `vinkius://auth/callback` — OAuth device flow completion
///   - `vinkius://install/{slug}?command=...&args=...&transport=...&url=...&env_KEY=VALUE`
///     One-click MCP server install from the web. Emits event to frontend.
fn handle_deep_link(app: &tauri::AppHandle, url: String) {
    log::info!("Deep link received: {}", url);

    if url.starts_with("vinkius://install/") {
        handle_install_deep_link(app, &url);
        return;
    }

    if url.starts_with("vinkius://auth/callback") {
        handle_auth_deep_link(app);
        return;
    }

    log::warn!("Unrecognized deep link: {}", url);
}

/// Parse `vinkius://install/{slug}?command=npx&args=-y,@pkg/name&transport=stdio`
/// and emit a `deep-link:install` event to the frontend with the server payload.
fn handle_install_deep_link(app: &tauri::AppHandle, url: &str) {
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(e) => {
            log::error!("Failed to parse install deep link: {}", e);
            return;
        }
    };

    // Extract slug from path: "/github" → "github"
    let slug = parsed.path().trim_start_matches('/').to_string();
    if slug.is_empty() {
        log::error!("Install deep link missing server slug");
        return;
    }

    let params: std::collections::HashMap<String, String> =
        parsed.query_pairs().map(|(k, v)| (k.to_string(), v.to_string())).collect();

    let command = params.get("command").cloned().unwrap_or_default();
    let args_raw = params.get("args").cloned().unwrap_or_default();
    let args: Vec<String> = if args_raw.is_empty() {
        vec![]
    } else {
        args_raw.split(',').map(|s| s.to_string()).collect()
    };
    let transport = params.get("transport").cloned().unwrap_or_else(|| "stdio".to_string());
    let server_url = params.get("url").cloned().unwrap_or_default();

    // Collect env vars: any param starting with "env_" becomes an env var
    let env: std::collections::HashMap<String, String> = params
        .iter()
        .filter_map(|(k, v)| k.strip_prefix("env_").map(|key| (key.to_string(), v.clone())))
        .collect();

    let payload = serde_json::json!({
        "slug": slug,
        "name": slug,
        "command": command,
        "args": args,
        "transport": transport,
        "url": server_url,
        "env": env,
    });

    log::info!("Deep link install: {} → {:?}", slug, payload);

    // Bring the window to focus
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.unminimize();
    }

    // Emit event to frontend — it will show a confirmation UI and run install
    let _ = app.emit("deep-link:install", payload);
}

/// Handle `vinkius://auth/callback` — completes OAuth device flow.
fn handle_auth_deep_link(app: &tauri::AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let auth_state = app.state::<AuthState>();
        let pending = {
            let guard = auth_state.pending.lock().await;
            guard.as_ref().map(|p| (p.device_code.clone(), p.cancel.clone()))
        };

        let Some((device_code, cancel_token)) = pending else {
            log::warn!("Deep link received but no pending auth flow");
            return;
        };

        // Cancel the polling task immediately
        cancel_token.cancel();

        // Exchange device_code for tokens
        match commands::auth::exchange_device_code(&device_code).await {
            Ok(tokens) => {
                log::info!("Deep link: token exchange successful");
                commands::auth::complete_auth_flow(&app, tokens).await;
            }
            Err(e) => {
                log::error!("Deep link: token exchange failed: {}", e);
                let _ = app.emit("auth:error", format!("Deep link auth failed: {}", e));
            }
        }

        // Clear pending state
        let mut guard = auth_state.pending.lock().await;
        *guard = None;
    });
}
