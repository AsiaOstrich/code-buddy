use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Working,
    Thinking,
    WaitingInput,
    WaitingConfirm,
    Completed,
    Error,
}

impl AgentStatus {
    /// 狀態優先級（數字越大越優先）
    pub fn priority(self) -> u8 {
        match self {
            AgentStatus::Idle => 0,
            AgentStatus::Completed => 1,
            AgentStatus::Thinking => 2,
            AgentStatus::Working => 3,
            AgentStatus::Error => 4,
            AgentStatus::WaitingInput => 5,
            AgentStatus::WaitingConfirm => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    ClaudeCode,
    OpenCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub project_path: String,
    pub project_name: String,
    #[serde(skip)]
    pub last_updated: Option<Instant>,
    /// Session 存活秒數（供前端顯示）
    pub duration_secs: u64,
}

/// 失敗計數器（連續失敗追蹤，防抖用）
#[derive(Debug, Default)]
pub struct FailureCounter {
    pub counts: HashMap<String, u32>,
}

impl FailureCounter {
    pub fn increment(&mut self, session_id: &str) -> u32 {
        let count = self.counts.entry(session_id.to_string()).or_insert(0);
        *count += 1;
        *count
    }

    pub fn reset(&mut self, session_id: &str) {
        self.counts.remove(session_id);
    }
}

pub struct AppState {
    pub sessions: Mutex<HashMap<String, SessionInfo>>,
    pub focus_session_id: Mutex<Option<String>>,
    pub pinned_session_id: Mutex<Option<String>>,
    pub failure_counter: Mutex<FailureCounter>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            focus_session_id: Mutex::new(None),
            pinned_session_id: Mutex::new(None),
            failure_counter: Mutex::new(FailureCounter::default()),
        }
    }
}

impl AppState {
    /// 多 session 時取最高優先級狀態
    pub fn aggregate_status(&self) -> AgentStatus {
        let sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        sessions
            .values()
            .map(|s| s.status)
            .max_by_key(|s| s.priority())
            .unwrap_or(AgentStatus::Idle)
    }

    /// 取得焦點 session 的狀態（pinned > focus > aggregate）
    pub fn effective_status(&self) -> AgentStatus {
        let sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        let pinned = self
            .pinned_session_id
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let focus = self
            .focus_session_id
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        let target_id = pinned.as_ref().or(focus.as_ref());

        if let Some(id) = target_id {
            if let Some(session) = sessions.get(id) {
                return session.status;
            }
        }

        // 沒有焦點 session 時用 aggregate
        drop(sessions);
        drop(pinned);
        drop(focus);
        self.aggregate_status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === AC-2: AgentStatus 優先級 ===

    #[test]
    fn test_priority_order() {
        // [Derived] AC-2: waiting > error > working > thinking > completed > idle
        assert!(AgentStatus::WaitingInput.priority() > AgentStatus::Error.priority());
        assert!(AgentStatus::Error.priority() > AgentStatus::Working.priority());
        assert!(AgentStatus::Working.priority() > AgentStatus::Thinking.priority());
        assert!(AgentStatus::Thinking.priority() > AgentStatus::Completed.priority());
        assert!(AgentStatus::Completed.priority() > AgentStatus::Idle.priority());
    }

    #[test]
    fn test_waiting_input_and_confirm_same_priority() {
        assert_eq!(
            AgentStatus::WaitingInput.priority(),
            AgentStatus::WaitingConfirm.priority()
        );
    }

    // === AC-2: 序列化格式 ===

    #[test]
    fn test_status_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&AgentStatus::WaitingInput).unwrap(),
            "\"waiting_input\""
        );
        assert_eq!(
            serde_json::to_string(&AgentStatus::WaitingConfirm).unwrap(),
            "\"waiting_confirm\""
        );
    }

    #[test]
    fn test_agent_type_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&AgentType::ClaudeCode).unwrap(),
            "\"claude_code\""
        );
    }

    // === AC-6: 聚合狀態 ===

    fn make_session(id: &str, status: AgentStatus) -> SessionInfo {
        SessionInfo {
            id: id.to_string(),
            agent_type: AgentType::ClaudeCode,
            status,
            project_path: format!("/path/{}", id),
            project_name: id.to_string(),
            last_updated: None,
            duration_secs: 0,
        }
    }

    #[test]
    fn test_aggregate_returns_highest_priority() {
        let state = AppState::default();
        {
            let mut sessions = state.sessions.lock().unwrap();
            sessions.insert("s1".into(), make_session("s1", AgentStatus::Working));
            sessions.insert("s2".into(), make_session("s2", AgentStatus::WaitingInput));
        }
        assert_eq!(state.aggregate_status(), AgentStatus::WaitingInput);
    }

    #[test]
    fn test_aggregate_defaults_to_idle_when_empty() {
        let state = AppState::default();
        assert_eq!(state.aggregate_status(), AgentStatus::Idle);
    }

    #[test]
    fn test_effective_uses_pinned_over_aggregate() {
        let state = AppState::default();
        {
            let mut sessions = state.sessions.lock().unwrap();
            sessions.insert("s1".into(), make_session("s1", AgentStatus::WaitingInput));
            sessions.insert("s2".into(), make_session("s2", AgentStatus::Idle));
        }
        *state.pinned_session_id.lock().unwrap() = Some("s2".to_string());
        // pinned 是 s2 (idle)，即使 s1 更高優先
        assert_eq!(state.effective_status(), AgentStatus::Idle);
    }

    #[test]
    fn test_effective_falls_back_to_aggregate_when_no_pin() {
        let state = AppState::default();
        {
            let mut sessions = state.sessions.lock().unwrap();
            sessions.insert("s1".into(), make_session("s1", AgentStatus::Error));
            sessions.insert("s2".into(), make_session("s2", AgentStatus::Idle));
        }
        assert_eq!(state.effective_status(), AgentStatus::Error);
    }

    // === AC-6: 邊界案例 ===

    #[test]
    fn test_effective_falls_back_when_pinned_session_not_found() {
        let state = AppState::default();
        {
            let mut sessions = state.sessions.lock().unwrap();
            sessions.insert("s1".into(), make_session("s1", AgentStatus::Working));
        }
        // pin 指向不存在的 session
        *state.pinned_session_id.lock().unwrap() = Some("nonexistent".to_string());
        // 應 fallback 到 aggregate（Working）
        assert_eq!(state.effective_status(), AgentStatus::Working);
    }

    // === AC-6: FailureCounter ===

    #[test]
    fn test_failure_counter_increments() {
        let mut c = FailureCounter::default();
        assert_eq!(c.increment("s1"), 1);
        assert_eq!(c.increment("s1"), 2);
        assert_eq!(c.increment("s1"), 3);
    }

    #[test]
    fn test_failure_counter_resets() {
        let mut c = FailureCounter::default();
        c.increment("s1");
        c.increment("s1");
        c.reset("s1");
        assert_eq!(c.increment("s1"), 1);
    }

    #[test]
    fn test_failure_counter_independent_per_session() {
        let mut c = FailureCounter::default();
        c.increment("s1");
        c.increment("s1");
        assert_eq!(c.increment("s2"), 1);
        assert_eq!(c.increment("s1"), 3);
    }
}
