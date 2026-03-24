// [Source] SPEC-001 — AC-5, AC-6
// [Generated] TDD 測試骨架：Claude Code Adapter（Hook 事件處理）
// 測試目標：adapters/claude_code.rs 中的 process_hook_event

#[cfg(test)]
mod tests {
    use code_buddy_lib::adapters::claude_code::{process_hook_event, HookPayload};
    use code_buddy_lib::state::*;

    fn make_payload(event: &str, session_id: &str) -> HookPayload {
        HookPayload {
            hook_event_name: event.to_string(),
            session_id: session_id.to_string(),
            project_path: "/Users/user/my-project".to_string(),
            notification_type: None,
            raw: None,
        }
    }

    fn make_notification_payload(session_id: &str, notification_type: &str) -> HookPayload {
        HookPayload {
            hook_event_name: "Notification".to_string(),
            session_id: session_id.to_string(),
            project_path: "/Users/user/my-project".to_string(),
            notification_type: Some(notification_type.to_string()),
            raw: None,
        }
    }

    // === AC-5: Session 註冊與移除 ===

    #[test]
    fn test_session_start_registers_new_session() {
        // [Derived] AC-5: SessionStart 註冊新 session
        let state = AppState::default();
        let payload = make_payload("SessionStart", "test-001");
        let result = process_hook_event(&state, &payload);

        assert_eq!(result.unwrap(), AgentStatus::Idle);
        let sessions = state.sessions.lock().unwrap();
        assert!(sessions.contains_key("test-001"));
        let session = &sessions["test-001"];
        assert_eq!(session.agent_type, AgentType::ClaudeCode);
        assert_eq!(session.project_name, "my-project");
    }

    #[test]
    fn test_session_start_sets_focus() {
        // [Derived] AC-5: 新 session 自動成為焦點
        let state = AppState::default();
        let payload = make_payload("SessionStart", "test-001");
        process_hook_event(&state, &payload).unwrap();

        let focus = state.focus_session_id.lock().unwrap();
        assert_eq!(focus.as_deref(), Some("test-001"));
    }

    #[test]
    fn test_session_end_removes_session() {
        // [Derived] AC-6: SessionEnd 移除 session
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "test-001")).unwrap();
        process_hook_event(&state, &make_payload("SessionEnd", "test-001")).unwrap();

        let sessions = state.sessions.lock().unwrap();
        assert!(!sessions.contains_key("test-001"));
    }

    #[test]
    fn test_session_end_clears_focus_if_focused() {
        // [Derived] AC-6: SessionEnd 清除該 session 的焦點
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "test-001")).unwrap();
        process_hook_event(&state, &make_payload("SessionEnd", "test-001")).unwrap();

        let focus = state.focus_session_id.lock().unwrap();
        assert_eq!(*focus, None);
    }

    // === AC-6: 狀態轉換 ===

    #[test]
    fn test_user_prompt_submit_sets_thinking() {
        // [Derived] AC-6: UserPromptSubmit → thinking
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "test-001")).unwrap();
        let result = process_hook_event(&state, &make_payload("UserPromptSubmit", "test-001"));

        assert_eq!(result.unwrap(), AgentStatus::Thinking);
        let sessions = state.sessions.lock().unwrap();
        assert_eq!(sessions["test-001"].status, AgentStatus::Thinking);
    }

    #[test]
    fn test_post_tool_use_sets_working() {
        // [Derived] AC-6: PostToolUse → working
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "test-001")).unwrap();
        let result = process_hook_event(&state, &make_payload("PostToolUse", "test-001"));

        assert_eq!(result.unwrap(), AgentStatus::Working);
    }

    #[test]
    fn test_stop_sets_completed() {
        // [Derived] AC-6: Stop → completed
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "test-001")).unwrap();
        let result = process_hook_event(&state, &make_payload("Stop", "test-001"));

        assert_eq!(result.unwrap(), AgentStatus::Completed);
    }

    #[test]
    fn test_notification_idle_prompt_sets_waiting_input() {
        // [Derived] AC-6: Notification(idle_prompt) → waiting_input
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "test-001")).unwrap();
        let result =
            process_hook_event(&state, &make_notification_payload("test-001", "idle_prompt"));

        assert_eq!(result.unwrap(), AgentStatus::WaitingInput);
    }

    #[test]
    fn test_notification_permission_prompt_sets_waiting_confirm() {
        // [Derived] AC-6: Notification(permission_prompt) → waiting_confirm
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "test-001")).unwrap();
        let result = process_hook_event(
            &state,
            &make_notification_payload("test-001", "permission_prompt"),
        );

        assert_eq!(result.unwrap(), AgentStatus::WaitingConfirm);
    }

    // === AC-6: 狀態防抖機制 ===

    #[test]
    fn test_single_failure_keeps_working() {
        // [Derived] AC-6: 單次 PostToolUseFailure 維持 working
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "test-001")).unwrap();
        process_hook_event(&state, &make_payload("PostToolUse", "test-001")).unwrap();
        let result =
            process_hook_event(&state, &make_payload("PostToolUseFailure", "test-001"));

        assert_eq!(result.unwrap(), AgentStatus::Working);
    }

    #[test]
    fn test_three_consecutive_failures_trigger_error() {
        // [Derived] AC-6: 連續 3 次失敗觸發 error
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "test-001")).unwrap();

        process_hook_event(&state, &make_payload("PostToolUseFailure", "test-001")).unwrap();
        process_hook_event(&state, &make_payload("PostToolUseFailure", "test-001")).unwrap();
        let result =
            process_hook_event(&state, &make_payload("PostToolUseFailure", "test-001"));

        assert_eq!(result.unwrap(), AgentStatus::Error);
    }

    #[test]
    fn test_success_resets_failure_counter() {
        // [Derived] AC-6: 成功的 PostToolUse 重置失敗計數
        let state = AppState::default();
        process_hook_event(&state, &make_payload("SessionStart", "test-001")).unwrap();

        process_hook_event(&state, &make_payload("PostToolUseFailure", "test-001")).unwrap();
        process_hook_event(&state, &make_payload("PostToolUseFailure", "test-001")).unwrap();
        // 中間穿插成功
        process_hook_event(&state, &make_payload("PostToolUse", "test-001")).unwrap();
        // 重新計數
        let result =
            process_hook_event(&state, &make_payload("PostToolUseFailure", "test-001"));

        assert_eq!(result.unwrap(), AgentStatus::Working); // 只有 1 次，不觸發 error
    }

    // === AC-5: 容錯：漏接 SessionStart ===

    #[test]
    fn test_event_without_session_start_creates_session() {
        // [Derived] AC-5: 即使漏接 SessionStart，也能處理後續事件
        let state = AppState::default();
        let result = process_hook_event(&state, &make_payload("PostToolUse", "unknown-001"));

        assert_eq!(result.unwrap(), AgentStatus::Working);
        let sessions = state.sessions.lock().unwrap();
        assert!(sessions.contains_key("unknown-001"));
    }

    // === AC-5: 未知事件 ===

    #[test]
    fn test_unknown_event_returns_error() {
        // [Derived] AC-5: 未知 hook 事件回傳錯誤
        let state = AppState::default();
        let payload = make_payload("UnknownEvent", "test-001");
        let result = process_hook_event(&state, &payload);

        assert!(result.is_err());
    }

    // === AC-5: extract_project_name ===

    #[test]
    fn test_project_name_extracted_from_path() {
        // [Derived] AC-5: 從 cwd 路徑提取最後一段作為專案名
        let state = AppState::default();
        let mut payload = make_payload("SessionStart", "test-001");
        payload.project_path = "/Users/dev/projects/my-app".to_string();
        process_hook_event(&state, &payload).unwrap();

        let sessions = state.sessions.lock().unwrap();
        assert_eq!(sessions["test-001"].project_name, "my-app");
    }
}
