use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::state::AgentStatus;

/// 防重複通知的 debounce 間隔（秒）
const DEBOUNCE_SECS: u64 = 30;

/// 通知管理器 — 管理防重複邏輯
pub struct NotificationManager {
    /// Key: "session_id:status"，Value: 最後通知時間
    last_notified: Mutex<HashMap<String, Instant>>,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self {
            last_notified: Mutex::new(HashMap::new()),
        }
    }
}

impl NotificationManager {
    /// 嘗試發送通知（自動防重複）
    pub fn try_notify(
        &self,
        app: &AppHandle,
        session_id: &str,
        project_name: &str,
        status: AgentStatus,
    ) {
        if !should_notify(status) {
            return;
        }

        let key = format!("{}:{:?}", session_id, status);
        let mut last = self.last_notified.lock().unwrap();

        if let Some(last_time) = last.get(&key) {
            if last_time.elapsed().as_secs() < DEBOUNCE_SECS {
                return;
            }
        }

        let (title, body) = notification_text(project_name, status);
        if let Err(e) = app
            .notification()
            .builder()
            .title(&title)
            .body(&body)
            .show()
        {
            eprintln!("通知發送失敗: {}", e);
        }

        last.insert(key, Instant::now());
    }
}

/// 判斷此狀態是否需要推送通知
fn should_notify(status: AgentStatus) -> bool {
    matches!(
        status,
        AgentStatus::Completed
            | AgentStatus::WaitingInput
            | AgentStatus::WaitingConfirm
            | AgentStatus::Error
    )
}

/// 取得對應狀態的中文通知文案
fn notification_text(project_name: &str, status: AgentStatus) -> (String, String) {
    match status {
        AgentStatus::Completed => (
            "任務完成".to_string(),
            format!("{} — Agent 已完成任務", project_name),
        ),
        AgentStatus::WaitingInput => (
            "等待輸入".to_string(),
            format!("{} — Agent 等待你的指示", project_name),
        ),
        AgentStatus::WaitingConfirm => (
            "需要確認".to_string(),
            format!("{} — Agent 需要你授權操作", project_name),
        ),
        AgentStatus::Error => (
            "發生錯誤".to_string(),
            format!("{} — Agent 遇到連續錯誤", project_name),
        ),
        _ => (
            "Code Buddy".to_string(),
            format!("{} — {:?}", project_name, status),
        ),
    }
}
