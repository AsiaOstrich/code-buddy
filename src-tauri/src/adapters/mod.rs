pub mod claude_code;
pub mod opencode;

use async_trait::async_trait;

use crate::state::{AgentStatus, SessionInfo};

/// Agent 通訊介面 — 所有 AI 編輯器 adapter 必須實作此 trait（v0.2.0 啟用）
#[allow(dead_code)]
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// 取得目前 agent 狀態
    async fn get_status(&self) -> Result<AgentStatus, Box<dyn std::error::Error>>;

    /// 取得所有活躍 session
    async fn get_sessions(&self) -> Result<Vec<SessionInfo>, Box<dyn std::error::Error>>;
}
