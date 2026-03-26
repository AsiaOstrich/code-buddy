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
    let float_toggle =
        MenuItem::with_id(app, "float", "浮動模式", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "關於 Code Buddy", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出 Code Buddy", true, None::<&str>)?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&float_toggle)
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
            "float" => {
                if let Err(e) = crate::float::toggle_float_window(app) {
                    tracing::error!("浮動視窗切換失敗: {}", e);
                }
            }
            "quit" => app.exit(0),
            "about" => {
                tracing::info!("Code Buddy v0.2.0");
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

#[cfg(test)]
mod tests {
    use super::*;

    // === get_icon_bytes：每個狀態回傳非空且唯一的圖示資料 ===

    #[test]
    fn idle_returns_non_empty_icon() {
        let bytes = get_icon_bytes(&AgentStatus::Idle);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn working_returns_non_empty_icon() {
        let bytes = get_icon_bytes(&AgentStatus::Working);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn thinking_returns_non_empty_icon() {
        let bytes = get_icon_bytes(&AgentStatus::Thinking);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn waiting_input_returns_non_empty_icon() {
        let bytes = get_icon_bytes(&AgentStatus::WaitingInput);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn waiting_confirm_returns_non_empty_icon() {
        let bytes = get_icon_bytes(&AgentStatus::WaitingConfirm);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn completed_returns_non_empty_icon() {
        let bytes = get_icon_bytes(&AgentStatus::Completed);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn error_returns_non_empty_icon() {
        let bytes = get_icon_bytes(&AgentStatus::Error);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn each_status_has_distinct_icon() {
        let all = [
            get_icon_bytes(&AgentStatus::Idle),
            get_icon_bytes(&AgentStatus::Working),
            get_icon_bytes(&AgentStatus::Thinking),
            get_icon_bytes(&AgentStatus::WaitingInput),
            get_icon_bytes(&AgentStatus::WaitingConfirm),
            get_icon_bytes(&AgentStatus::Completed),
            get_icon_bytes(&AgentStatus::Error),
        ];
        // 每個圖示的指標位置應不同（各自 include_bytes! 產生不同 static）
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(
                    all[i].as_ptr(),
                    all[j].as_ptr(),
                    "狀態 {} 和 {} 共用了相同圖示",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn all_icons_are_valid_png() {
        let statuses = [
            AgentStatus::Idle,
            AgentStatus::Working,
            AgentStatus::Thinking,
            AgentStatus::WaitingInput,
            AgentStatus::WaitingConfirm,
            AgentStatus::Completed,
            AgentStatus::Error,
        ];
        for status in &statuses {
            let bytes = get_icon_bytes(status);
            // PNG magic bytes: 0x89 0x50 0x4E 0x47
            assert!(
                bytes.len() >= 4 && bytes[0] == 0x89 && bytes[1] == 0x50,
                "{:?} 的圖示不是有效的 PNG 格式",
                status
            );
        }
    }
}

pub fn update_tray_icon(
    app: &AppHandle,
    status: &AgentStatus,
) -> Result<(), Box<dyn std::error::Error>> {
    let tray = app.tray_by_id("main").ok_or("Tray icon not found")?;
    let icon_bytes = get_icon_bytes(status);
    let icon = Image::from_bytes(icon_bytes)?;
    tray.set_icon(Some(icon))?;
    tray.set_icon_as_template(false)?;
    tray.set_tooltip(Some(&format!("Code Buddy - {:?}", status)))?;
    Ok(())
}
