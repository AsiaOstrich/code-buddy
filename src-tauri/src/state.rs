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
        let sessions = self.sessions.lock().unwrap();
        sessions
            .values()
            .map(|s| s.status)
            .max_by_key(|s| s.priority())
            .unwrap_or(AgentStatus::Idle)
    }

    /// 取得焦點 session 的狀態（pinned > focus > aggregate）
    pub fn effective_status(&self) -> AgentStatus {
        let sessions = self.sessions.lock().unwrap();
        let pinned = self.pinned_session_id.lock().unwrap();
        let focus = self.focus_session_id.lock().unwrap();

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
