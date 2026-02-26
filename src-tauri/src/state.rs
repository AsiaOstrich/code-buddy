use serde::{Deserialize, Serialize};
use std::sync::Mutex;

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
}

pub struct AppState {
    pub current_status: Mutex<AgentStatus>,
    #[allow(dead_code)] // v0.2.0 啟用
    pub sessions: Mutex<Vec<SessionInfo>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_status: Mutex::new(AgentStatus::Idle),
            sessions: Mutex::new(Vec::new()),
        }
    }
}
