use tauri::AppHandle;

use crate::state::AgentStatus;

const ICON_IDLE: &[u8] = include_bytes!("../icons/dock/idle.png");
const ICON_WORKING: &[u8] = include_bytes!("../icons/dock/working.png");
const ICON_THINKING: &[u8] = include_bytes!("../icons/dock/thinking.png");
const ICON_WAITING_INPUT: &[u8] = include_bytes!("../icons/dock/waiting_input.png");
const ICON_WAITING_CONFIRM: &[u8] = include_bytes!("../icons/dock/waiting_confirm.png");
const ICON_COMPLETED: &[u8] = include_bytes!("../icons/dock/completed.png");
const ICON_ERROR: &[u8] = include_bytes!("../icons/dock/error.png");

pub fn get_dock_icon_bytes(status: &AgentStatus) -> &'static [u8] {
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

/// 動態更新 macOS Dock 圖示
pub fn update_dock_icon(
    app: &AppHandle,
    status: &AgentStatus,
) -> Result<(), Box<dyn std::error::Error>> {
    let icon_bytes = get_dock_icon_bytes(status).to_vec();
    app.run_on_main_thread(move || {
        use objc2::{AnyThread, MainThreadMarker};
        use objc2_app_kit::{NSApplication, NSImage};
        use objc2_foundation::NSData;
        // Safety: run_on_main_thread 保證此 closure 在主執行緒執行
        // Safety: run_on_main_thread 保證此 closure 在主執行緒執行
        unsafe {
            let mtm = MainThreadMarker::new_unchecked();
            let ns_app = NSApplication::sharedApplication(mtm);
            let data = NSData::with_bytes(&icon_bytes);
            if let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) {
                ns_app.setApplicationIconImage(Some(&image));
            }
        }
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_status_returns_non_empty_icon() {
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
            let bytes = get_dock_icon_bytes(status);
            assert!(!bytes.is_empty(), "{:?} icon is empty", status);
        }
    }

    #[test]
    fn each_status_has_distinct_icon() {
        let all = [
            get_dock_icon_bytes(&AgentStatus::Idle),
            get_dock_icon_bytes(&AgentStatus::Working),
            get_dock_icon_bytes(&AgentStatus::Thinking),
            get_dock_icon_bytes(&AgentStatus::WaitingInput),
            get_dock_icon_bytes(&AgentStatus::WaitingConfirm),
            get_dock_icon_bytes(&AgentStatus::Completed),
            get_dock_icon_bytes(&AgentStatus::Error),
        ];
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
            let bytes = get_dock_icon_bytes(status);
            assert!(
                bytes.len() >= 4 && bytes[0] == 0x89 && bytes[1] == 0x50,
                "{:?} 的圖示不是有效的 PNG 格式",
                status
            );
        }
    }
}
