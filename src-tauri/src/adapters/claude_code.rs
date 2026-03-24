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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    fn make_payload(event: &str, session_id: &str) -> HookPayload {
        HookPayload {
            hook_event_name: event.to_string(),
            session_id: session_id.to_string(),
            project_path: "/Users/user/my-project".to_string(),
            notification_type: None,
            raw: None,
        }
    }

    fn make_notification(session_id: &str, ntype: &str) -> HookPayload {
        HookPayload {
            hook_event_name: "Notification".to_string(),
            session_id: session_id.to_string(),
            project_path: "/Users/user/my-project".to_string(),
            notification_type: Some(ntype.to_string()),
            raw: None,
        }
    }

    // === AC-5: Session 註冊 ===

    #[test]
    fn session_start_registers_session() {
        let state = AppState::default();
        let result = process_hook_event(&state, &make_payload("SessionStart", "s1"));
        assert_eq!(result.unwrap(), AgentStatus::Idle);

        let sessions = state.sessions.lock().unwrap();
        assert!(sessions.contains_key("s1"));
        assert_eq!(sessions["s1"].agent_type, AgentType::ClaudeCode);
        assert_eq!(sessions["s1"].project_name, "my-project");
    }

    #[test]
    fn session_start_sets_focus() {
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "s1")).unwrap();
        assert_eq!(
            *state.focus_session_id.lock().unwrap(),
            Some("s1".to_string())
        );
    }

    #[test]
    fn session_end_removes_session() {
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "s1")).unwrap();
        process_hook_event(&state, &make_payload("SessionEnd", "s1")).unwrap();

        let sessions = state.sessions.lock().unwrap();
        assert!(!sessions.contains_key("s1"));
    }

    #[test]
    fn session_end_clears_focus() {
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "s1")).unwrap();
        process_hook_event(&state, &make_payload("SessionEnd", "s1")).unwrap();
        assert_eq!(*state.focus_session_id.lock().unwrap(), None);
    }

    // === AC-6: 狀態轉換 ===

    #[test]
    fn user_prompt_submit_sets_thinking() {
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "s1")).unwrap();
        let r = process_hook_event(&state, &make_payload("UserPromptSubmit", "s1"));
        assert_eq!(r.unwrap(), AgentStatus::Thinking);

        let sessions = state.sessions.lock().unwrap();
        assert_eq!(sessions["s1"].status, AgentStatus::Thinking);
    }

    #[test]
    fn post_tool_use_sets_working() {
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "s1")).unwrap();
        let r = process_hook_event(&state, &make_payload("PostToolUse", "s1"));
        assert_eq!(r.unwrap(), AgentStatus::Working);
    }

    #[test]
    fn stop_sets_completed() {
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "s1")).unwrap();
        let r = process_hook_event(&state, &make_payload("Stop", "s1"));
        assert_eq!(r.unwrap(), AgentStatus::Completed);
    }

    #[test]
    fn notification_idle_prompt_sets_waiting_input() {
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "s1")).unwrap();
        let r = process_hook_event(&state, &make_notification("s1", "idle_prompt"));
        assert_eq!(r.unwrap(), AgentStatus::WaitingInput);
    }

    #[test]
    fn notification_permission_prompt_sets_waiting_confirm() {
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "s1")).unwrap();
        let r = process_hook_event(&state, &make_notification("s1", "permission_prompt"));
        assert_eq!(r.unwrap(), AgentStatus::WaitingConfirm);
    }

    // === AC-6: 防抖機制 ===

    #[test]
    fn single_failure_keeps_working() {
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "s1")).unwrap();
        process_hook_event(&state, &make_payload("PostToolUse", "s1")).unwrap();
        let r = process_hook_event(&state, &make_payload("PostToolUseFailure", "s1"));
        assert_eq!(r.unwrap(), AgentStatus::Working);
    }

    #[test]
    fn three_failures_trigger_error() {
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "s1")).unwrap();
        process_hook_event(&state, &make_payload("PostToolUseFailure", "s1")).unwrap();
        process_hook_event(&state, &make_payload("PostToolUseFailure", "s1")).unwrap();
        let r = process_hook_event(&state, &make_payload("PostToolUseFailure", "s1"));
        assert_eq!(r.unwrap(), AgentStatus::Error);
    }

    #[test]
    fn success_resets_failure_counter() {
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "s1")).unwrap();
        process_hook_event(&state, &make_payload("PostToolUseFailure", "s1")).unwrap();
        process_hook_event(&state, &make_payload("PostToolUseFailure", "s1")).unwrap();
        // 成功重置計數
        process_hook_event(&state, &make_payload("PostToolUse", "s1")).unwrap();
        let r = process_hook_event(&state, &make_payload("PostToolUseFailure", "s1"));
        assert_eq!(r.unwrap(), AgentStatus::Working); // 重新計數，只有 1 次
    }

    // === AC-5: 容錯 ===

    #[test]
    fn event_without_session_start_creates_session() {
        let state = AppState::default();
        let r = process_hook_event(&state, &make_payload("PostToolUse", "unknown"));
        assert_eq!(r.unwrap(), AgentStatus::Working);

        let sessions = state.sessions.lock().unwrap();
        assert!(sessions.contains_key("unknown"));
    }

    #[test]
    fn unknown_event_returns_error() {
        let state = AppState::default();
        let r = process_hook_event(&state, &make_payload("UnknownEvent", "s1"));
        assert!(r.is_err());
    }

    // === AC-5: extract_project_name ===

    #[test]
    fn extracts_last_segment_from_path() {
        assert_eq!(extract_project_name("/Users/dev/projects/my-app"), "my-app");
    }

    #[test]
    fn handles_empty_path() {
        assert_eq!(extract_project_name(""), "");
    }

    #[test]
    fn handles_single_segment() {
        assert_eq!(extract_project_name("my-app"), "my-app");
    }
}
