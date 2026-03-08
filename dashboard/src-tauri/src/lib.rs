use std::collections::HashMap;
use std::sync::Arc;

use tauri::{Emitter, Manager};
use tauri::menu::{MenuBuilder, SubmenuBuilder};
use tokio::sync::Mutex;
use tuitbot_core::auth::passphrase;
use tuitbot_core::config::{ContentSourcesConfig, DeploymentMode};
use tuitbot_core::startup::data_dir;
use tuitbot_core::storage;
use tuitbot_server::auth;
use tuitbot_server::state::AppState;
use tuitbot_server::ws::AccountWsEvent;

/// Shared reference to the embedded server's application state.
struct EmbeddedState(Arc<AppState>);

/// Read the API token from ~/.tuitbot/api_token.
fn read_api_token() -> Result<String, String> {
    let token_path = data_dir().join("api_token");
    std::fs::read_to_string(&token_path)
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("Failed to read API token at {}: {}", token_path.display(), e))
}

/// Tauri command: returns the API token to the frontend.
#[tauri::command]
fn get_api_token() -> Result<String, String> {
    read_api_token()
}

/// Tauri command: open an isolated OAuth webview that intercepts the callback.
///
/// When X redirects to the callback URL, the webview extracts the `code`
/// and `state` query params, emits them via a `oauth-callback` event to
/// the main window, and closes itself.
#[tauri::command]
fn open_oauth_window(app_handle: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri::webview::WebviewWindowBuilder;

    let label = format!("oauth-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis());

    let handle = app_handle.clone();
    WebviewWindowBuilder::new(&app_handle, &label, tauri::WebviewUrl::External(
        url.parse().map_err(|e| format!("invalid URL: {e}"))?,
    ))
    .title("Authorize X Account")
    .inner_size(520.0, 720.0)
    .center()
    .on_navigation(move |nav_url| {
        let url_str = nav_url.as_str();
        // Intercept the callback redirect (e.g. http://localhost:3001/callback?code=...&state=...)
        if url_str.contains("/callback") && url_str.contains("code=") {
            if let Ok(parsed) = url::Url::parse(url_str) {
                let code = parsed.query_pairs()
                    .find(|(k, _)| k == "code")
                    .map(|(_, v)| v.to_string())
                    .unwrap_or_default();
                let state = parsed.query_pairs()
                    .find(|(k, _)| k == "state")
                    .map(|(_, v)| v.to_string())
                    .unwrap_or_default();

                let _ = handle.emit("oauth-callback", serde_json::json!({
                    "code": code,
                    "state": state,
                }));

                // Can't close inside on_navigation — schedule it.
                let h2 = handle.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    // Find and close the oauth window.
                    for (_, wv) in h2.webview_windows() {
                        if wv.label().starts_with("oauth-") {
                            let _ = wv.close();
                        }
                    }
                });

                // Block the navigation — don't load the callback URL.
                return false;
            }
        }
        true
    })
    .build()
    .map_err(|e| format!("failed to open OAuth window: {e}"))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize the embedded server: DB + token + broadcast channel.
            let state = tauri::async_runtime::block_on(async {
                let dir = data_dir();
                std::fs::create_dir_all(&dir).expect("failed to create ~/.tuitbot/");

                let config_path = dir.join("config.toml");
                let db_path = dir.join("tuitbot.db");

                let pool = storage::init_db(&db_path.to_string_lossy())
                    .await
                    .expect("failed to init database");

                let api_token =
                    auth::ensure_api_token(&dir).expect("failed to create API token");

                // Ensure passphrase exists (Tauri uses bearer tokens, but the
                // hash is needed if the user later accesses via web browser).
                if let Err(e) = passphrase::ensure_passphrase(&dir) {
                    log::warn!("Failed to initialize passphrase: {}", e);
                }
                let passphrase_hash = passphrase::load_passphrase_hash(&dir)
                    .ok()
                    .flatten();

                let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

                // Load config for x_client_id.
                let config_contents = std::fs::read_to_string(&config_path).unwrap_or_default();
                let loaded_config: Result<tuitbot_core::config::Config, _> = toml::from_str(&config_contents);
                let x_client_id = loaded_config
                    .as_ref()
                    .map(|c| c.x_api.client_id.clone())
                    .unwrap_or_default();

                let passphrase_mtime = passphrase::passphrase_hash_mtime(&dir);

                Arc::new(AppState {
                    db: pool,
                    config_path,
                    data_dir: dir,
                    event_tx,
                    api_token,
                    passphrase_hash: tokio::sync::RwLock::new(passphrase_hash),
                    passphrase_hash_mtime: tokio::sync::RwLock::new(passphrase_mtime),
                    bind_host: "127.0.0.1".to_string(),
                    bind_port: 3001,
                    login_attempts: Mutex::new(HashMap::new()),
                    runtimes: Mutex::new(HashMap::new()),
                    content_generators: Mutex::new(HashMap::new()),
                    circuit_breaker: None,
                    watchtower_cancel: None,
                    content_sources: ContentSourcesConfig::default(),
                    connector_config: Default::default(),
                    deployment_mode: DeploymentMode::Desktop,
                    pending_oauth: Mutex::new(HashMap::new()),
                    token_managers: Mutex::new(HashMap::new()),
                    x_client_id,
                })
            });

            // Spawn the axum server on a background tokio task.
            let router = tuitbot_server::build_router(state.clone());
            tauri::async_runtime::spawn(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
                    .await
                    .expect("failed to bind to port 3001");
                log::info!("Embedded server listening on http://127.0.0.1:3001");
                if let Err(e) = axum::serve(listener, router).await {
                    log::error!("Embedded server error: {}", e);
                }
            });

            app.manage(EmbeddedState(state));

            // --- Application menu ---
            // Custom Paste menu item emits a Tauri event so the webview
            // can handle both text and image paste via the clipboard plugin.
            let app_submenu = SubmenuBuilder::new(app, &app.package_info().name)
                .about(None)
                .separator()
                .services()
                .separator()
                .hide()
                .hide_others()
                .show_all()
                .separator()
                .quit()
                .build()?;

            let edit_submenu = SubmenuBuilder::new(app, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                // No .paste() — Cmd+V must reach JavaScript for image paste support.
                // Text paste is handled via Tauri clipboard plugin readText().
                .separator()
                .select_all()
                .build()?;

            let window_submenu = SubmenuBuilder::new(app, "Window")
                .minimize()
                .close_window()
                .separator()
                .fullscreen()
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&app_submenu)
                .item(&edit_submenu)
                .item(&window_submenu)
                .build()?;
            app.set_menu(menu)?;

            // --- System tray ---
            build_system_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_api_token, open_oauth_window])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| {
        if let tauri::RunEvent::ExitRequested { .. } = event {
            // Ensure the process fully terminates so the embedded server
            // doesn't keep port 3001 bound after the window closes.
            std::process::exit(0);
        }
    });
}

/// Build the system tray icon and context menu.
fn build_system_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
    use tauri::tray::TrayIconBuilder;

    let open_dashboard = MenuItemBuilder::with_id("open_dashboard", "Open Dashboard").build(app)?;
    let toggle_automation =
        MenuItemBuilder::with_id("toggle_automation", "Start Automation").build(app)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let approval_queue =
        MenuItemBuilder::with_id("approval_queue", "Approval Queue").build(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit Tuitbot").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&open_dashboard)
        .item(&toggle_automation)
        .item(&sep1)
        .item(&approval_queue)
        .item(&sep2)
        .item(&quit)
        .build()?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().unwrap())
        .icon_as_template(true)
        .menu(&menu)
        .on_menu_event(move |app_handle, event| {
            let id = event.id().as_ref();
            match id {
                "open_dashboard" => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "toggle_automation" => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "approval_queue" => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.eval("window.location.href = '/approval'");
                    }
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
