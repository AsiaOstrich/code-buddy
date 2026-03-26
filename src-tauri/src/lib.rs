mod adapters;
mod commands;
#[cfg(target_os = "macos")]
mod dock;
mod float;
mod notification;
mod popover;
mod server;
mod state;
mod tray;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::switch_tray_icon,
            commands::get_current_status,
            commands::get_sessions,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Regular);

            tray::setup_tray(app)?;

            #[cfg(target_os = "macos")]
            if let Err(e) = dock::update_dock_icon(app.handle(), &state::AgentStatus::Idle) {
                tracing::error!("初始 Dock icon 設定失敗: {}", e);
            }

            // 啟動 HTTP server 接收 hook 事件
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                server::start_server(app_handle).await;
            });

            // Debug 模式下建立 Dev Panel 視窗
            #[cfg(debug_assertions)]
            {
                let _window = tauri::WebviewWindowBuilder::new(
                    app,
                    "devpanel",
                    tauri::WebviewUrl::App("index.html".into()),
                )
                .title("Code Buddy - Dev Panel")
                .inner_size(400.0, 600.0)
                .build()?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
