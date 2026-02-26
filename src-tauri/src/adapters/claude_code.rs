use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;

use crate::state::{AgentStatus, AgentType, AppState, SessionInfo};

/// Claude Code Hook 事件名稱
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookEventName {
    SessionStart,
    SessionEnd,
    UserPromptSubmit,
    PostToolUse,
    PostToolUseFailure,
    Stop,
    Notification,
}

impl HookEventName {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "SessionStart" => Some(Self::SessionStart),
            "SessionEnd" => Some(Self::SessionEnd),
            "UserPromptSubmit" => Some(Self::UserPromptSubmit),
            "PostToolUse" => Some(Self::PostToolUse),
            "PostToolUseFailure" => Some(Self::PostToolUseFailure),
            "Stop" => Some(Self::Stop),
            "Notification" => Some(Self::Notification),
            _ => None,
        }
    }
}

/// Hook Script 送過來的 JSON payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookPayload {
    pub hook_event_name: String,
    pub session_id: String,
    #[serde(default)]
    pub project_path: String,
    #[serde(default)]
    pub notification_type: Option<String>,
    #[serde(default)]
    pub raw: Option<serde_json::Value>,
}

/// 處理 Hook 事件，更新 AppState，回傳新狀態
pub fn process_hook_event(
    state: &AppState,
    payload: &HookPayload,
) -> Result<AgentStatus, String> {
    let event = HookEventName::from_str(&payload.hook_event_name)
        .ok_or_else(|| format!("未知的 hook 事件: {}", payload.hook_event_name))?;

    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let mut failure_counter = state.failure_counter.lock().map_err(|e| e.to_string())?;

    match event {
        HookEventName::SessionStart => {
            let project_name = extract_project_name(&payload.project_path);
            sessions.insert(
                payload.session_id.clone(),
                SessionInfo {
                    id: payload.session_id.clone(),
                    agent_type: AgentType::ClaudeCode,
                    status: AgentStatus::Idle,
                    project_path: payload.project_path.clone(),
                    project_name,
                    last_updated: Some(Instant::now()),
                    duration_secs: 0,
                },
            );
            if let Ok(mut focus) = state.focus_session_id.lock() {
                *focus = Some(payload.session_id.clone());
            }
            failure_counter.reset(&payload.session_id);
            Ok(AgentStatus::Idle)
        }
        HookEventName::SessionEnd => {
            sessions.remove(&payload.session_id);
            failure_counter.reset(&payload.session_id);
            if let Ok(mut focus) = state.focus_session_id.lock() {
                if focus.as_ref() == Some(&payload.session_id) {
                    *focus = None;
                }
            }
            Ok(AgentStatus::Idle)
        }
        HookEventName::UserPromptSubmit => {
            update_session_status(&mut sessions, &payload.session_id, AgentStatus::Thinking, &payload.project_path);
            failure_counter.reset(&payload.session_id);
            Ok(AgentStatus::Thinking)
        }
        HookEventName::PostToolUse => {
            update_session_status(&mut sessions, &payload.session_id, AgentStatus::Working, &payload.project_path);
            failure_counter.reset(&payload.session_id);
            Ok(AgentStatus::Working)
        }
        HookEventName::PostToolUseFailure => {
            let count = failure_counter.increment(&payload.session_id);
            if count >= 3 {
                update_session_status(&mut sessions, &payload.session_id, AgentStatus::Error, &payload.project_path);
                Ok(AgentStatus::Error)
            } else {
                update_session_status(&mut sessions, &payload.session_id, AgentStatus::Working, &payload.project_path);
                Ok(AgentStatus::Working)
            }
        }
        HookEventName::Stop => {
            update_session_status(&mut sessions, &payload.session_id, AgentStatus::Completed, &payload.project_path);
            failure_counter.reset(&payload.session_id);
            Ok(AgentStatus::Completed)
        }
        HookEventName::Notification => {
            let status = match payload.notification_type.as_deref() {
                Some("idle_prompt") => AgentStatus::WaitingInput,
                Some("permission_prompt") => AgentStatus::WaitingConfirm,
                _ => AgentStatus::WaitingInput,
            };
            update_session_status(&mut sessions, &payload.session_id, status, &payload.project_path);
            failure_counter.reset(&payload.session_id);
            Ok(status)
        }
    }
}

fn update_session_status(
    sessions: &mut std::collections::HashMap<String, SessionInfo>,
    session_id: &str,
    status: AgentStatus,
    project_path: &str,
) {
    if let Some(session) = sessions.get_mut(session_id) {
        session.status = status;
        if let Some(started) = session.last_updated {
            session.duration_secs = started.elapsed().as_secs();
        }
    } else {
        // 容錯：可能漏接 SessionStart
        let project_name = extract_project_name(project_path);
        sessions.insert(
            session_id.to_string(),
            SessionInfo {
                id: session_id.to_string(),
                agent_type: AgentType::ClaudeCode,
                status,
                project_path: project_path.to_string(),
                project_name,
                last_updated: Some(Instant::now()),
                duration_secs: 0,
            },
        );
    }
}

fn extract_project_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}
