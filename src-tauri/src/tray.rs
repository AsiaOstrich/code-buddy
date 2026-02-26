use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::state::AgentStatus;

const ICON_IDLE: &[u8] = include_bytes!("../icons/tray/idle.png");
const ICON_WORKING: &[u8] = include_bytes!("../icons/tray/working.png");
const ICON_THINKING: &[u8] = include_bytes!("../icons/tray/thinking.png");
const ICON_WAITING_INPUT: &[u8] = include_bytes!("../icons/tray/waiting_input.png");
const ICON_WAITING_CONFIRM: &[u8] = include_bytes!("../icons/tray/waiting_confirm.png");
const ICON_COMPLETED: &[u8] = include_bytes!("../icons/tray/completed.png");
const ICON_ERROR: &[u8] = include_bytes!("../icons/tray/error.png");

pub fn get_icon_bytes(status: &AgentStatus) -> &'static [u8] {
    match status {
        AgentStatus::Idle => ICON_IDLE,
        AgentStatus::Working => ICON_WORKING,
        AgentStatus::Thinking => ICON_THINKING,
        AgentStatus::WaitingInput => ICON_WAITING_INPUT,
        AgentStatus::WaitingConfirm => ICON_WAITING_CONFIRM,
        AgentStatus::Completed => ICON_COMPLETED,
        AgentStatus::Error => ICON_ERROR,
    }
}

/// 顯示並聚焦 Dev Panel 視窗
fn show_and_focus_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("devpanel") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, "show", "顯示面板", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "關於 Code Buddy", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出 Code Buddy", true, None::<&str>)?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .separator()
        .item(&about)
        .separator()
        .item(&quit)
        .build()?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(Image::from_bytes(ICON_IDLE)?)
        .icon_as_template(false)
        .tooltip("Code Buddy - Idle")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_and_focus_window(app),
            "quit" => app.exit(0),
            "about" => {
                // TODO: v0.3.0 — 顯示 About 視窗
                println!("Code Buddy v0.2.0");
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_and_focus_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub fn update_tray_icon(
    app: &AppHandle,
    status: &AgentStatus,
) -> Result<(), Box<dyn std::error::Error>> {
    let tray = app
        .tray_by_id("main")
        .ok_or("Tray icon not found")?;
    let icon_bytes = get_icon_bytes(status);
    let icon = Image::from_bytes(icon_bytes)?;
    tray.set_icon(Some(icon))?;
    tray.set_icon_as_template(false)?;
    tray.set_tooltip(Some(&format!("Code Buddy - {:?}", status)))?;
    Ok(())
}
