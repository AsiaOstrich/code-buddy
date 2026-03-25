// [Source] SPEC-001 — AC-7, AC-10
// [Status] ✅ 已由源碼內建測試完整覆蓋
// [Location] src-tauri/src/notification.rs #[cfg(test)] mod tests
//
// 覆蓋項目：
//   AC-7: should_notify() — 7 個測試（4 種應通知 + 3 種不通知）
//   AC-7: notification_text() — 4 個測試（completed, waiting_input, waiting_confirm, error）
//   AC-7: debounce 純邏輯 — 3 個測試（key 組合、無紀錄、間隔內）
//
// 執行方式：cd src-tauri && cargo test notification
//
// 以下骨架保留作為追蹤矩陣參考，不再需要獨立實作。
