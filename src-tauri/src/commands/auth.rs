use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const STORE_KEY: &str = "credentials";

use crate::config::constants::API_BASE_URL;

/// Returns the API base URL from the centralized configuration.
fn api_base() -> &'static str {
    API_BASE_URL
}

// ── Types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionState {
    pub authenticated: bool,
    pub user: Option<UserInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub uuid: String,
    pub name: String,
    pub email: String,
    pub avatar: Option<String>,
}

/// Returned to the frontend immediately after calling `login`.
/// The Desktop **does not** display the code — it opens the browser directly
/// to `verification_url` (which has the code embedded in the URL).
/// The Desktop simply shows "Waiting for authorization..." until polling completes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginResult {
    pub verification_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StoredCredentials {
    access_token: String,
    refresh_token: String,
    token_type: String,
    /// Unix timestamp (seconds) when the access token expires.
    expires_at: i64,
}

#[derive(Debug, Deserialize)]
struct DeviceResponse {
    device_code: String,
    #[allow(dead_code)]
    user_code: String,
    verification_uri_complete: String,
    #[allow(dead_code)]
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TokenResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: i64,
}

#[derive(Debug, Deserialize)]
struct UserResponse {
    data: UserData,
}

#[derive(Debug, Deserialize)]
struct UserData {
    uuid: String,
    name: String,
    email: String,
    avatar: Option<String>,
}

// ── Shared Auth State ────────────────────────────────────────────────

/// Shared state for the active Device Flow login.
///
/// When a login is initiated, the `device_code` and a `CancellationToken`
/// are stored here. The deep link handler can then:
///   1. Read the device_code to exchange tokens immediately
///   2. Cancel the polling task to avoid duplicate token exchanges
pub struct PendingAuth {
    pub device_code: String,
    pub cancel: CancellationToken,
}

/// Thread-safe container for the pending auth flow.
/// Managed as Tauri app state via `app.manage()`.
#[derive(Default, Clone)]
pub struct AuthState {
    pub pending: Arc<Mutex<Option<PendingAuth>>>,
}

// ── Token Persistence ────────────────────────────────────────────────

fn save_credentials(app: &tauri::AppHandle, creds: &StoredCredentials) -> Result<(), String> {
    let store = app
        .store("credentials.json")
        .map_err(|e| format!("Failed to open store: {}", e))?;

    let value = serde_json::to_value(creds)
        .map_err(|e| format!("Failed to serialize credentials: {}", e))?;

    store.set(STORE_KEY, value);
    store.save().map_err(|e| format!("Failed to save store: {}", e))?;

    Ok(())
}

fn load_credentials(app: &tauri::AppHandle) -> Option<StoredCredentials> {
    let store = app.store("credentials.json").ok()?;
    let value = store.get(STORE_KEY)?;
    serde_json::from_value(value).ok()
}

fn clear_credentials(app: &tauri::AppHandle) {
    if let Ok(store) = app.store("credentials.json") {
        store.delete(STORE_KEY);
        let _ = store.save();
    }
}

// ── Token Refresh ────────────────────────────────────────────────────

/// Refresh an expired access token using the stored refresh token.
async fn refresh_tokens(
    app: &tauri::AppHandle,
    refresh_token: &str,
) -> Result<StoredCredentials, String> {
    let client = reqwest::Client::new();
    let base = api_base();

    let response = client
        .post(format!("{}/auth/refresh", base))
        .json(&serde_json::json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .map_err(|e| format!("Refresh request failed: {}", e))?;

    if !response.status().is_success() {
        return Err("Refresh token expired or revoked".into());
    }

    let tokens: TokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Invalid refresh response: {}", e))?;

    let creds = StoredCredentials {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: tokens.token_type,
        expires_at: chrono::Utc::now().timestamp() + tokens.expires_in,
    };

    save_credentials(app, &creds)?;

    Ok(creds)
}

/// Get a valid access token — refreshes automatically if expired.
async fn get_valid_token(app: &tauri::AppHandle) -> Result<String, String> {
    let creds = load_credentials(app).ok_or("Not authenticated")?;

    // If token expires within the next 5 minutes, refresh proactively
    let now = chrono::Utc::now().timestamp();
    if now + 300 >= creds.expires_at {
        let refreshed = refresh_tokens(app, &creds.refresh_token).await?;
        return Ok(refreshed.access_token);
    }

    Ok(creds.access_token)
}

// ── Fetch User Profile ───────────────────────────────────────────────

async fn fetch_user(access_token: &str) -> Result<UserInfo, String> {
    let client = reqwest::Client::new();
    let base = api_base();

    let response = client
        .get(format!("{}/auth/user", base))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("User fetch failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("User API returned {}", response.status()));
    }

    let user_response: UserResponse = response
        .json()
        .await
        .map_err(|e| format!("Invalid user response: {}", e))?;

    Ok(UserInfo {
        uuid: user_response.data.uuid,
        name: user_response.data.name,
        email: user_response.data.email,
        avatar: user_response.data.avatar,
    })
}

// ── Single Token Exchange ────────────────────────────────────────────

/// Exchange a device_code for tokens — single attempt (no retry loop).
///
/// Used by the deep link handler when `vinkius://auth/callback` is received.
/// Returns `Ok(TokenResponse)` if the server has already approved the code,
/// or `Err` if the code is still pending or invalid.
pub async fn exchange_device_code(device_code: &str) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    let base = api_base();

    let response = client
        .post(format!("{}/auth/device/poll", base))
        .json(&serde_json::json!({ "device_code": device_code }))
        .send()
        .await
        .map_err(|e| format!("Token exchange failed: {}", e))?;

    let status = response.status();
    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Invalid exchange response: {}", e))?;

    if let Some(error) = body.get("error").and_then(|e| e.as_str()) {
        return Err(format!("Authorization error: {}", error));
    }

    if status.is_success() {
        let tokens: TokenResponse = serde_json::from_value(body)
            .map_err(|e| format!("Failed to parse tokens: {}", e))?;
        return Ok(tokens);
    }

    Err(format!("Token exchange returned {}", status))
}

/// Complete the auth flow — store tokens, fetch user, emit session event.
///
/// Shared by both the polling fallback and the deep link handler.
pub async fn complete_auth_flow(app: &tauri::AppHandle, tokens: TokenResponse) {
    let creds = StoredCredentials {
        access_token: tokens.access_token.clone(),
        refresh_token: tokens.refresh_token,
        token_type: tokens.token_type,
        expires_at: chrono::Utc::now().timestamp() + tokens.expires_in,
    };

    if let Err(e) = save_credentials(app, &creds) {
        log::error!("Failed to save credentials: {}", e);
        return;
    }

    match fetch_user(&tokens.access_token).await {
        Ok(user) => {
            let session = SessionState {
                authenticated: true,
                user: Some(user),
            };
            let _ = app.emit("auth:session-ready", session);
        }
        Err(e) => {
            log::error!("User fetch failed after token exchange: {}", e);
        }
    }
}

// ── IPC Commands ─────────────────────────────────────────────────────

/// Check current authentication session.
/// Reads stored tokens, refreshes if needed, and validates against the API.
#[tauri::command]
pub async fn get_session(app: tauri::AppHandle) -> Result<SessionState, String> {
    let token = match get_valid_token(&app).await {
        Ok(t) => t,
        Err(_) => {
            return Ok(SessionState {
                authenticated: false,
                user: None,
            });
        }
    };

    match fetch_user(&token).await {
        Ok(user) => Ok(SessionState {
            authenticated: true,
            user: Some(user),
        }),
        Err(_) => {
            // Token might be revoked — clear stored credentials
            clear_credentials(&app);
            Ok(SessionState {
                authenticated: false,
                user: None,
            })
        }
    }
}

/// Retrieve a valid access token for authenticated API calls.
///
/// The frontend calls this before making requests to Cloud endpoints
/// that require authentication (e.g. `POST /marketplace/free-access`).
/// Automatically refreshes the token if it's about to expire.
#[tauri::command]
pub async fn get_access_token(app: tauri::AppHandle) -> Result<String, String> {
    get_valid_token(&app).await
}

/// Initiate Device Flow login (RFC 8628) with deep link support.
///
/// 1. Requests device_code + user_code from Cloud API
/// 2. Stores device_code in `AuthState` for the deep link handler
/// 3. Opens browser to `verification_uri_complete` (code already in URL)
/// 4. Spawns background polling as fallback
/// 5. Returns verification_url immediately (frontend shows "waiting...")
/// 6. Emits `auth:session-ready` when either deep link or polling succeeds
///
/// The browser page redirects to `vinkius://auth/callback` after authorization.
/// If the deep link fires first, it cancels the polling task.
#[tauri::command]
pub async fn login(app: tauri::AppHandle) -> Result<LoginResult, String> {
    let client = reqwest::Client::new();
    let base = api_base();

    // Step 1: Request device code
    let response = client
        .post(format!("{}/auth/device", base))
        .json(&serde_json::json!({ "client_name": "Vinkius Desktop" }))
        .send()
        .await
        .map_err(|e| format!("Failed to request device code: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Device code request failed: {}", response.status()));
    }

    let device: DeviceResponse = response
        .json()
        .await
        .map_err(|e| format!("Invalid device response: {}", e))?;

    let result = LoginResult {
        verification_url: device.verification_uri_complete.clone(),
    };

    // Step 2: Store device_code + cancellation token in shared state
    let cancel_token = CancellationToken::new();
    let auth_state = app.state::<AuthState>();
    {
        let mut pending = auth_state.pending.lock().await;
        *pending = Some(PendingAuth {
            device_code: device.device_code.clone(),
            cancel: cancel_token.clone(),
        });
    }

    // Step 3: Open browser — the browser page has the code, user just clicks Authorize
    let _ = app.opener().open_url(&device.verification_uri_complete, None::<&str>);

    // Step 4: Spawn background polling as fallback (cancelled if deep link arrives first)
    let app_handle = app.clone();
    let device_code = device.device_code.clone();
    let interval = device.interval;
    let cancel = cancel_token.clone();

    tauri::async_runtime::spawn(async move {
        match poll_for_approval(&device_code, interval, &cancel).await {
            Ok(tokens) => {
                complete_auth_flow(&app_handle, tokens).await;
            }
            Err(e) => {
                // Don't emit error if we were cancelled by deep link
                if !cancel.is_cancelled() {
                    log::error!("Device flow polling failed: {}", e);
                    let _ = app_handle.emit("auth:error", e);
                }
            }
        }

        // Clear pending state
        let auth_state = app_handle.state::<AuthState>();
        let mut pending = auth_state.pending.lock().await;
        *pending = None;
    });

    Ok(result)
}

/// Logout — revoke token server-side and clear local storage.
#[tauri::command]
pub async fn logout(app: tauri::AppHandle) -> Result<(), String> {
    let base = api_base();

    // Best-effort revoke on server
    if let Some(creds) = load_credentials(&app) {
        let _ = reqwest::Client::new()
            .post(format!("{}/auth/logout", base))
            .bearer_auth(&creds.access_token)
            .send()
            .await;
    }

    // Clear local storage
    clear_credentials(&app);

    // Notify frontend
    let _ = app.emit("auth:logged-out", ());

    Ok(())
}

// ── Polling (fallback) ───────────────────────────────────────────────

/// Poll the device authorization endpoint until approval, cancellation, or timeout.
///
/// The `cancel` token is triggered by the deep link handler when
/// `vinkius://auth/callback` arrives — this stops the polling loop
/// immediately so only one code path completes the auth flow.
async fn poll_for_approval(
    device_code: &str,
    interval: u64,
    cancel: &CancellationToken,
) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    let base = api_base();
    let poll_interval = std::time::Duration::from_secs(interval.max(5));
    let max_attempts = 180; // 15 minutes at 5s intervals

    for _ in 0..max_attempts {
        // Check if deep link already completed the flow
        tokio::select! {
            _ = cancel.cancelled() => {
                return Err("Cancelled by deep link".into());
            }
            _ = tokio::time::sleep(poll_interval) => {}
        }

        // Double-check after sleep
        if cancel.is_cancelled() {
            return Err("Cancelled by deep link".into());
        }

        let response = client
            .post(format!("{}/auth/device/poll", base))
            .json(&serde_json::json!({ "device_code": device_code }))
            .send()
            .await
            .map_err(|e| format!("Poll request failed: {}", e))?;

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Invalid poll response: {}", e))?;

        // Check for error responses (400)
        if let Some(error) = body.get("error").and_then(|e| e.as_str()) {
            match error {
                "authorization_pending" => continue,
                "slow_down" => {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    continue;
                }
                "expired_token" => return Err("Authorization code expired".into()),
                "invalid_grant" => return Err("Authorization request invalid".into()),
                _ => return Err(format!("Authorization error: {}", error)),
            }
        }

        // Success — parse tokens
        if status.is_success() {
            let tokens: TokenResponse = serde_json::from_value(body)
                .map_err(|e| format!("Failed to parse tokens: {}", e))?;
            return Ok(tokens);
        }
    }

    Err("Authorization timed out".into())
}
