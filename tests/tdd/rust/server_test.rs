// [Source] SPEC-001 — AC-5
// [Status] ✅ 已由源碼內建測試完整覆蓋
// [Location] src-tauri/src/server.rs #[cfg(test)] mod tests
//
// 覆蓋項目：
//   AC-5: /health 端點 — health_returns_ok_with_version
//   AC-5: /claude-code/event — event_accepts_valid_session_start, event_returns_working_on_post_tool_use
//   AC-5: 未知事件 — event_rejects_unknown_hook_event
//   AC-5: 格式錯誤 — event_rejects_malformed_json
//   AC-5: 完整生命週期 — event_full_lifecycle_session_start_to_end
//   AC-5: BIND_ADDR — server_binds_to_localhost_only
//   AC-5: 純邏輯 — handle_hook_event_returns_ok_for_valid_event, handle_hook_event_returns_bad_request_for_unknown
//
// 執行方式：cd src-tauri && cargo test server
//
// 以下骨架保留作為追蹤矩陣參考，不再需要獨立實作。
