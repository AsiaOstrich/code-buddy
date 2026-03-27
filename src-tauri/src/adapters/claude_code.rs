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
    /// Claude Code 原始 hook JSON 使用 `cwd` 而非 `project_path`
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub notification_type: Option<String>,
    #[serde(default)]
    pub raw: Option<serde_json::Value>,
}

/// 處理 Hook 事件，更新 AppState，回傳新狀態
pub fn process_hook_event(state: &AppState, payload: &HookPayload) -> Result<AgentStatus, String> {
    let event = HookEventName::from_str(&payload.hook_event_name)
        .ok_or_else(|| format!("未知的 hook 事件: {}", payload.hook_event_name))?;

    // project_path 優先；若為空則 fallback 到 cwd（Claude Code 原始格式）
    let effective_path = if payload.project_path.is_empty() {
        payload.cwd.as_deref().unwrap_or("")
    } else {
        &payload.project_path
    };

    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let mut failure_counter = state.failure_counter.lock().map_err(|e| e.to_string())?;

    match event {
        HookEventName::SessionStart => {
            let project_name = extract_project_name(effective_path);
            sessions.insert(
                payload.session_id.clone(),
                SessionInfo {
                    id: payload.session_id.clone(),
                    agent_type: AgentType::ClaudeCode,
                    status: AgentStatus::Idle,
                    project_path: effective_path.to_string(),
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
            update_session_status(
                &mut sessions,
                &payload.session_id,
                AgentStatus::Thinking,
                effective_path,
            );
            failure_counter.reset(&payload.session_id);
            Ok(AgentStatus::Thinking)
        }
        HookEventName::PostToolUse => {
            update_session_status(
                &mut sessions,
                &payload.session_id,
                AgentStatus::Working,
                effective_path,
            );
            failure_counter.reset(&payload.session_id);
            Ok(AgentStatus::Working)
        }
        HookEventName::PostToolUseFailure => {
            let count = failure_counter.increment(&payload.session_id);
            if count >= 3 {
                update_session_status(
                    &mut sessions,
                    &payload.session_id,
                    AgentStatus::Error,
                    effective_path,
                );
                Ok(AgentStatus::Error)
            } else {
                update_session_status(
                    &mut sessions,
                    &payload.session_id,
                    AgentStatus::Working,
                    effective_path,
                );
                Ok(AgentStatus::Working)
            }
        }
        HookEventName::Stop => {
            update_session_status(
                &mut sessions,
                &payload.session_id,
                AgentStatus::Completed,
                effective_path,
            );
            failure_counter.reset(&payload.session_id);
            Ok(AgentStatus::Completed)
        }
        HookEventName::Notification => {
            let status = match payload.notification_type.as_deref() {
                Some("idle_prompt") => AgentStatus::WaitingInput,
                Some("permission_prompt") => AgentStatus::WaitingConfirm,
                _ => AgentStatus::WaitingInput,
            };
            update_session_status(
                &mut sessions,
                &payload.session_id,
                status,
                effective_path,
            );
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
            cwd: None,
            notification_type: None,
            raw: None,
        }
    }

    fn make_notification(session_id: &str, ntype: &str) -> HookPayload {
        HookPayload {
            hook_event_name: "Notification".to_string(),
            session_id: session_id.to_string(),
            project_path: "/Users/user/my-project".to_string(),
            cwd: None,
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

    // ==========================================================
    // SPEC-004: Raw JSON 格式 — cwd fallback 測試
    // ==========================================================

    // [Derived] AC-4: server 接受含 cwd 的原始 JSON 格式
    // [TODO] 需先在 HookPayload 新增 cwd: Option<String> 欄位

    /// [Generated] AC-4: 僅有 cwd、無 project_path 時，使用 cwd 作為 project_path
    #[test]
    fn raw_json_cwd_used_when_project_path_empty() {
        let state = AppState::default();
        let payload = HookPayload {
            hook_event_name: "SessionStart".to_string(),
            session_id: "raw-001".to_string(),
            project_path: "".to_string(),
            cwd: Some("/Users/dev/my-project".to_string()),
            notification_type: None,
            raw: None,
        };
        let result = process_hook_event(&state, &payload);
        assert_eq!(result.unwrap(), AgentStatus::Idle);

        let sessions = state.sessions.lock().unwrap();
        assert_eq!(sessions["raw-001"].project_path, "/Users/dev/my-project");
        assert_eq!(sessions["raw-001"].project_name, "my-project");
    }

    /// [Generated] AC-5: 同時有 project_path 和 cwd 時，優先使用 project_path
    #[test]
    fn project_path_takes_priority_over_cwd() {
        let state = AppState::default();
        let payload = HookPayload {
            hook_event_name: "SessionStart".to_string(),
            session_id: "priority-001".to_string(),
            project_path: "/from/project_path".to_string(),
            cwd: Some("/from/cwd".to_string()),
            notification_type: None,
            raw: None,
        };
        let result = process_hook_event(&state, &payload);
        assert_eq!(result.unwrap(), AgentStatus::Idle);

        let sessions = state.sessions.lock().unwrap();
        assert_eq!(sessions["priority-001"].project_path, "/from/project_path");
    }

    /// [Generated] AC-8: 兩欄位皆空時使用空字串（不 panic）
    #[test]
    fn both_path_fields_empty_uses_empty_string() {
        let state = AppState::default();
        let payload = HookPayload {
            hook_event_name: "PostToolUse".to_string(),
            session_id: "empty-001".to_string(),
            project_path: "".to_string(),
            cwd: None,
            notification_type: None,
            raw: None,
        };
        let result = process_hook_event(&state, &payload);
        assert_eq!(result.unwrap(), AgentStatus::Working);

        let sessions = state.sessions.lock().unwrap();
        assert_eq!(sessions["empty-001"].project_path, "");
    }

    /// [Generated] AC-8: cwd 為 None、project_path 為空時，fallback 到空字串
    #[test]
    fn cwd_none_project_path_empty_falls_back_to_empty() {
        let state = AppState::default();
        let payload = HookPayload {
            hook_event_name: "SessionStart".to_string(),
            session_id: "fallback-001".to_string(),
            project_path: "".to_string(),
            cwd: None,
            notification_type: None,
            raw: None,
        };
        process_hook_event(&state, &payload).unwrap();

        let sessions = state.sessions.lock().unwrap();
        assert_eq!(sessions["fallback-001"].project_path, "");
        assert_eq!(sessions["fallback-001"].project_name, "");
    }

    /// [Generated] AC-4: Windows 路徑的 cwd 正確提取 project_name
    #[test]
    fn cwd_windows_path_extracts_project_name() {
        let state = AppState::default();
        let payload = HookPayload {
            hook_event_name: "SessionStart".to_string(),
            session_id: "win-001".to_string(),
            project_path: "".to_string(),
            cwd: Some("C:\\Users\\dev\\Documents\\my-app".to_string()),
            notification_type: None,
            raw: None,
        };
        process_hook_event(&state, &payload).unwrap();

        let sessions = state.sessions.lock().unwrap();
        assert_eq!(sessions["win-001"].project_name, "my-app");
    }

    /// [Generated] AC-6: raw JSON 格式下完整生命週期（cwd 欄位）
    #[test]
    fn raw_json_full_lifecycle_with_cwd() {
        let state = AppState::default();

        // SessionStart with cwd
        let p = HookPayload {
            hook_event_name: "SessionStart".to_string(),
            session_id: "lc-001".to_string(),
            project_path: "".to_string(),
            cwd: Some("/Users/dev/app".to_string()),
            notification_type: None,
            raw: None,
        };
        assert_eq!(process_hook_event(&state, &p).unwrap(), AgentStatus::Idle);

        // UserPromptSubmit
        let p = HookPayload {
            hook_event_name: "UserPromptSubmit".to_string(),
            session_id: "lc-001".to_string(),
            project_path: "".to_string(),
            cwd: Some("/Users/dev/app".to_string()),
            notification_type: None,
            raw: None,
        };
        assert_eq!(process_hook_event(&state, &p).unwrap(), AgentStatus::Thinking);

        // PostToolUse
        let p = HookPayload {
            hook_event_name: "PostToolUse".to_string(),
            session_id: "lc-001".to_string(),
            project_path: "".to_string(),
            cwd: Some("/Users/dev/app".to_string()),
            notification_type: None,
            raw: None,
        };
        assert_eq!(process_hook_event(&state, &p).unwrap(), AgentStatus::Working);

        // Stop
        let p = HookPayload {
            hook_event_name: "Stop".to_string(),
            session_id: "lc-001".to_string(),
            project_path: "".to_string(),
            cwd: Some("/Users/dev/app".to_string()),
            notification_type: None,
            raw: None,
        };
        assert_eq!(process_hook_event(&state, &p).unwrap(), AgentStatus::Completed);

        // SessionEnd
        let p = HookPayload {
            hook_event_name: "SessionEnd".to_string(),
            session_id: "lc-001".to_string(),
            project_path: "".to_string(),
            cwd: Some("/Users/dev/app".to_string()),
            notification_type: None,
            raw: None,
        };
        assert_eq!(process_hook_event(&state, &p).unwrap(), AgentStatus::Idle);

        let sessions = state.sessions.lock().unwrap();
        assert!(!sessions.contains_key("lc-001"));
    }

    /// [Generated] AC-4+AC-6: raw JSON Notification 正確解析 notification_type
    #[test]
    fn raw_json_notification_with_cwd() {
        let state = AppState::default();

        let p = HookPayload {
            hook_event_name: "SessionStart".to_string(),
            session_id: "notif-001".to_string(),
            project_path: "".to_string(),
            cwd: Some("/Users/dev/app".to_string()),
            notification_type: None,
            raw: None,
        };
        process_hook_event(&state, &p).unwrap();

        let p = HookPayload {
            hook_event_name: "Notification".to_string(),
            session_id: "notif-001".to_string(),
            project_path: "".to_string(),
            cwd: Some("/Users/dev/app".to_string()),
            notification_type: Some("permission_prompt".to_string()),
            raw: None,
        };
        assert_eq!(process_hook_event(&state, &p).unwrap(), AgentStatus::WaitingConfirm);
    }

    // [Derived] AC-7: 現有 server.rs 測試用 raw JSON POST 格式
    // → 見 server.rs 中的整合測試（需新增 raw JSON 格式的測試案例）
}
