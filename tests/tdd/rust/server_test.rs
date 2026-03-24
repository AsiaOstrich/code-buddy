// [Source] SPEC-001 — AC-5
// [Generated] TDD 測試骨架：HTTP Server 端點
// 測試目標：server.rs 中的 /health 和 /claude-code/event 端點

// [TODO] 整合測試需要 Tauri AppHandle，需使用 axum::test 或 mock
// 以下為預期的 HTTP 層測試骨架

#[cfg(test)]
mod tests {

    // === AC-5: /health 端點 ===

    #[test]
    fn test_health_returns_ok() {
        // [Derived] AC-5: GET /health 回傳 200
        // [TODO] 使用 axum::test helpers 測試
        // let response = app.oneshot(Request::get("/health").body(Body::empty()).unwrap()).await;
        // assert_eq!(response.status(), StatusCode::OK);
    }

    // === AC-5: /claude-code/event 端點 ===

    #[test]
    fn test_claude_code_event_accepts_valid_payload() {
        // [Derived] AC-5: POST /claude-code/event 接受有效 JSON
        // [TODO] 使用 axum::test helpers 測試
    }

    #[test]
    fn test_claude_code_event_rejects_invalid_event() {
        // [Derived] AC-5: 未知事件回傳 400
        // [TODO] 使用 axum::test helpers 測試
    }

    #[test]
    fn test_server_binds_to_localhost_only() {
        // [Derived] AC-5: Server 僅綁定 127.0.0.1，不對外暴露
        // [TODO] 驗證 BIND_ADDR 為 127.0.0.1
    }
}
