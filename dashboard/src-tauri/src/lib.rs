use std::collections::HashMap;
use std::sync::Arc;

use tauri::Manager;
use tokio::sync::Mutex;
use tuitbot_core::auth::passphrase;
use tuitbot_core::startup::data_dir;
use tuitbot_core::storage;
use tuitbot_server::auth;
use tuitbot_server::state::AppState;
use tuitbot_server::ws::WsEvent;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
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

                let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);

                Arc::new(AppState {
                    db: pool,
                    config_path,
                    data_dir: dir,
                    event_tx,
                    api_token,
                    passphrase_hash: tokio::sync::RwLock::new(passphrase_hash),
                    bind_host: "127.0.0.1".to_string(),
                    bind_port: 3001,
                    login_attempts: Mutex::new(HashMap::new()),
                    runtimes: Mutex::new(HashMap::new()),
                    content_generators: Mutex::new(HashMap::new()),
                    circuit_breaker: None,
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

            // --- System tray ---
            build_system_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_api_token])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, _event| {});
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
