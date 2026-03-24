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

        let key = debounce_key(session_id, status);
        let mut last = self.last_notified.lock().unwrap_or_else(|e| e.into_inner());

        if is_within_debounce(&last, &key) {
            return;
        }

        let (title, body) = notification_text(project_name, status);
        if let Err(e) = app
            .notification()
            .builder()
            .title(&title)
            .body(&body)
            .show()
        {
            tracing::error!("通知發送失敗: {}", e);
        }

        last.insert(key, Instant::now());
    }
}

/// 建構 debounce key（session_id + status 組合）
pub(crate) fn debounce_key(session_id: &str, status: AgentStatus) -> String {
    format!("{}:{:?}", session_id, status)
}

/// 判斷是否在 debounce 間隔內（純邏輯，可測試）
pub(crate) fn is_within_debounce(last_notified: &HashMap<String, Instant>, key: &str) -> bool {
    if let Some(last_time) = last_notified.get(key) {
        last_time.elapsed().as_secs() < DEBOUNCE_SECS
    } else {
        false
    }
}

/// 判斷此狀態是否需要推送通知
pub(crate) fn should_notify(status: AgentStatus) -> bool {
    matches!(
        status,
        AgentStatus::Completed
            | AgentStatus::WaitingInput
            | AgentStatus::WaitingConfirm
            | AgentStatus::Error
    )
}

/// 取得對應狀態的中文通知文案
pub(crate) fn notification_text(project_name: &str, status: AgentStatus) -> (String, String) {
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

#[cfg(test)]
mod tests {
    use super::*;

    // === AC-7: 通知觸發條件 ===

    #[test]
    fn completed_should_notify() {
        assert!(should_notify(AgentStatus::Completed));
    }

    #[test]
    fn waiting_input_should_notify() {
        assert!(should_notify(AgentStatus::WaitingInput));
    }

    #[test]
    fn waiting_confirm_should_notify() {
        assert!(should_notify(AgentStatus::WaitingConfirm));
    }

    #[test]
    fn error_should_notify() {
        assert!(should_notify(AgentStatus::Error));
    }

    #[test]
    fn working_should_not_notify() {
        assert!(!should_notify(AgentStatus::Working));
    }

    #[test]
    fn thinking_should_not_notify() {
        assert!(!should_notify(AgentStatus::Thinking));
    }

    #[test]
    fn idle_should_not_notify() {
        assert!(!should_notify(AgentStatus::Idle));
    }

    // === AC-7: 通知文案 ===

    #[test]
    fn completed_text_contains_project_name() {
        let (title, body) = notification_text("my-app", AgentStatus::Completed);
        assert_eq!(title, "任務完成");
        assert!(body.contains("my-app"));
    }

    #[test]
    fn waiting_input_text() {
        let (title, body) = notification_text("my-app", AgentStatus::WaitingInput);
        assert_eq!(title, "等待輸入");
        assert!(body.contains("my-app"));
    }

    #[test]
    fn waiting_confirm_text() {
        let (title, _) = notification_text("my-app", AgentStatus::WaitingConfirm);
        assert_eq!(title, "需要確認");
    }

    #[test]
    fn error_text() {
        let (title, _) = notification_text("my-app", AgentStatus::Error);
        assert_eq!(title, "發生錯誤");
    }

    // === AC-7: debounce 純邏輯 ===

    #[test]
    fn debounce_key_combines_session_and_status() {
        let key = debounce_key("sess-001", AgentStatus::Completed);
        assert!(key.contains("sess-001"));
        assert!(key.contains("Completed"));
    }

    #[test]
    fn not_within_debounce_when_no_prior_notification() {
        let last: HashMap<String, Instant> = HashMap::new();
        assert!(!is_within_debounce(&last, "sess:Completed"));
    }

    #[test]
    fn within_debounce_when_recently_notified() {
        let mut last: HashMap<String, Instant> = HashMap::new();
        last.insert("sess:Completed".to_string(), Instant::now());
        assert!(is_within_debounce(&last, "sess:Completed"));
    }
}
