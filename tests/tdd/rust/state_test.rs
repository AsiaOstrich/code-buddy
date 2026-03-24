// [Source] SPEC-001 — AC-1, AC-2, AC-4, AC-6, AC-11
// [Generated] TDD 測試骨架：AppState、AgentStatus、Session 管理
// 測試目標：state.rs 中的核心型別與邏輯

#[cfg(test)]
mod tests {
    use code_buddy_lib::state::*;
    use std::collections::HashMap;

    // === AC-2: AgentStatus 優先級 ===

    #[test]
    fn test_agent_status_priority_order() {
        // [Derived] AC-2: 7 種狀態各有明確優先級
        assert!(AgentStatus::WaitingInput.priority() > AgentStatus::Error.priority());
        assert!(AgentStatus::Error.priority() > AgentStatus::Working.priority());
        assert!(AgentStatus::Working.priority() > AgentStatus::Thinking.priority());
        assert!(AgentStatus::Thinking.priority() > AgentStatus::Completed.priority());
        assert!(AgentStatus::Completed.priority() > AgentStatus::Idle.priority());
    }

    #[test]
    fn test_waiting_input_and_waiting_confirm_same_priority() {
        // [Derived] AC-2: waiting_input 與 waiting_confirm 同優先級
        assert_eq!(
            AgentStatus::WaitingInput.priority(),
            AgentStatus::WaitingConfirm.priority()
        );
    }

    // === AC-6: 多 Session 聚合狀態 ===

    #[test]
    fn test_aggregate_status_returns_highest_priority() {
        // [Derived] AC-6: Tray 顯示最高優先狀態
        let state = AppState::default();
        {
            let mut sessions = state.sessions.lock().unwrap();
            sessions.insert(
                "s1".to_string(),
                SessionInfo {
                    id: "s1".to_string(),
                    agent_type: AgentType::ClaudeCode,
                    status: AgentStatus::Working,
                    project_path: "/path/a".to_string(),
                    project_name: "a".to_string(),
                    last_updated: None,
                    duration_secs: 0,
                },
            );
            sessions.insert(
                "s2".to_string(),
                SessionInfo {
                    id: "s2".to_string(),
                    agent_type: AgentType::ClaudeCode,
                    status: AgentStatus::WaitingInput,
                    project_path: "/path/b".to_string(),
                    project_name: "b".to_string(),
                    last_updated: None,
                    duration_secs: 0,
                },
            );
        }
        assert_eq!(state.aggregate_status(), AgentStatus::WaitingInput);
    }

    #[test]
    fn test_aggregate_status_defaults_to_idle_when_empty() {
        // [Derived] AC-6: 無 session 時回傳 idle
        let state = AppState::default();
        assert_eq!(state.aggregate_status(), AgentStatus::Idle);
    }

    #[test]
    fn test_effective_status_uses_pinned_session() {
        // [Derived] AC-9: pinned session 優先於 aggregate
        let state = AppState::default();
        {
            let mut sessions = state.sessions.lock().unwrap();
            sessions.insert(
                "s1".to_string(),
                SessionInfo {
                    id: "s1".to_string(),
                    agent_type: AgentType::ClaudeCode,
                    status: AgentStatus::WaitingInput,
                    project_path: "/path/a".to_string(),
                    project_name: "a".to_string(),
                    last_updated: None,
                    duration_secs: 0,
                },
            );
            sessions.insert(
                "s2".to_string(),
                SessionInfo {
                    id: "s2".to_string(),
                    agent_type: AgentType::ClaudeCode,
                    status: AgentStatus::Idle,
                    project_path: "/path/b".to_string(),
                    project_name: "b".to_string(),
                    last_updated: None,
                    duration_secs: 0,
                },
            );
        }
        *state.pinned_session_id.lock().unwrap() = Some("s2".to_string());
        assert_eq!(state.effective_status(), AgentStatus::Idle);
    }

    // === AC-6: FailureCounter 防抖機制 ===

    #[test]
    fn test_failure_counter_increment() {
        // [Derived] AC-6: 失敗計數遞增
        let mut counter = FailureCounter::default();
        assert_eq!(counter.increment("s1"), 1);
        assert_eq!(counter.increment("s1"), 2);
        assert_eq!(counter.increment("s1"), 3);
    }

    #[test]
    fn test_failure_counter_reset() {
        // [Derived] AC-6: 成功後重置計數
        let mut counter = FailureCounter::default();
        counter.increment("s1");
        counter.increment("s1");
        counter.reset("s1");
        assert_eq!(counter.increment("s1"), 1);
    }

    #[test]
    fn test_failure_counter_independent_per_session() {
        // [Derived] AC-6: 不同 session 獨立計數
        let mut counter = FailureCounter::default();
        counter.increment("s1");
        counter.increment("s1");
        assert_eq!(counter.increment("s2"), 1);
        assert_eq!(counter.increment("s1"), 3);
    }

    // === AC-4: 序列化格式 ===

    #[test]
    fn test_agent_status_serializes_to_snake_case() {
        // [Derived] AC-2: JSON 序列化為 snake_case
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
        // [Derived] AC-5: agent_type 序列化
        assert_eq!(
            serde_json::to_string(&AgentType::ClaudeCode).unwrap(),
            "\"claude_code\""
        );
    }
}
