mod adapters;
mod commands;
mod notification;
mod popover;
mod server;
mod state;
mod tray;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::switch_tray_icon,
            commands::get_current_status,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::setup_tray(app)?;

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
