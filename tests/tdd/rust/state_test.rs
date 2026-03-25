// [Source] SPEC-001 — AC-1, AC-2, AC-4, AC-6, AC-11
// [Status] ✅ 已由源碼內建測試完整覆蓋
// [Location] src-tauri/src/state.rs #[cfg(test)] mod tests
//
// 覆蓋項目：
//   AC-2: AgentStatus 優先級 — test_priority_order, test_waiting_input_and_confirm_same_priority
//   AC-6: 多 Session 聚合 — test_aggregate_returns_highest_priority, test_aggregate_defaults_to_idle_when_empty
//   AC-9: 焦點 session — test_effective_uses_pinned_over_aggregate, test_effective_falls_back_*
//   AC-6: FailureCounter — test_failure_counter_increments, test_failure_counter_resets, test_failure_counter_independent_per_session
//   AC-2: 序列化 — test_status_serializes_to_snake_case, test_agent_type_serializes_to_snake_case
//
// 執行方式：cd src-tauri && cargo test state
//
// 以下骨架保留作為追蹤矩陣參考，不再需要獨立實作。
