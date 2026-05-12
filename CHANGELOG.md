# Changelog

All notable changes to Vinkius Desktop will be documented in this file.

## [0.1.4] - 2026-05-12

### Added
- **Deep link install protocol** — One-click MCP server installation from the web via `vinkius://install/{slug}` deep links. The Rust backend parses the URL, extracts server configuration (command, args, transport, env vars), and emits it to the frontend.
- **DeepLinkInstallModal** — Confirmation modal that shows server details, target AI clients, and installation progress with per-client status reporting.
- **useDeepLink composable** — Reactive listener for `deep-link:install` events from the Rust backend with shared modal state.
- **Centralized Rust constants** — New `config::constants` module (`API_BASE_URL`, `APP_BASE_URL`, `SITE_BASE_URL`) as single source of truth, mirroring the frontend `src/config/app.ts`.

### Changed
- **CTA standardized** — Replaced "Get MCP Server (Free)" with "Unlock for AI Agents" across all CTA buttons to align with marketplace branding.
- **Login modal copy** — Updated tagline to match the landing page messaging and replaced "No credit card required" with "No credit card · No commitment".
- **Listing card typography** — Adjusted description text size from `text-[13px]` to `text-sm` for consistency.

### Fixed
- **Panic-resistant backend** — Replaced environment-variable-based `api_base()` function with centralized constants, eliminating runtime panics when `.env` is missing.
- **Locale-free URLs** — Removed `/en/` locale prefix from all outbound URLs (share links, legal pages, community discovery) to enforce clean, global URL structure.

## [0.1.3] - 2026-05-11

### Fixed
- Add User-Agent header to bypass Cloudflare 1010 on Discord webhook.

## [0.1.2] - 2026-05-11

### Added
- Discord commit notification workflow.
- Discord community CTA in README.

## [0.1.1] - 2026-05-11

### Changed
- Auto-update Homebrew, Scoop, AUR, and Winget on release.
- Publish releases automatically without draft mode.
- Write permissions for release generation.

### Fixed
- Clean deep link payload unwrap for macOS.
- Missing Listener trait import for macOS deep linking.

## [0.1.0] - 2026-05-10

### Added
- Initial open source release.
