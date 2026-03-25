// [Source] SPEC-001 — AC-5, AC-6
// [Status] ✅ 已由源碼內建測試完整覆蓋
// [Location] src-tauri/src/adapters/claude_code.rs #[cfg(test)] mod tests
//
// 覆蓋項目：
//   AC-5: Session 註冊 — session_start_registers_session, session_start_sets_focus
//   AC-6: Session 移除 — session_end_removes_session, session_end_clears_focus
//   AC-6: 狀態轉換 — user_prompt_submit_sets_thinking, post_tool_use_sets_working, stop_sets_completed
//   AC-6: Notification — notification_idle_prompt_sets_waiting_input, notification_permission_prompt_sets_waiting_confirm
//   AC-6: 防抖機制 — single_failure_keeps_working, three_failures_trigger_error, success_resets_failure_counter
//   AC-5: 容錯 — event_without_session_start_creates_session
//   AC-5: 未知事件 — unknown_event_returns_error
//   AC-5: 專案名 — extracts_last_segment_from_path, handles_single_segment, handles_empty_path
//
// 執行方式：cd src-tauri && cargo test claude_code
//
// 以下骨架保留作為追蹤矩陣參考，不再需要獨立實作。
