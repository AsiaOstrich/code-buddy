use axum::{
    extract::State as AxumState,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::TcpListener;

use crate::adapters::claude_code::{process_hook_event, HookPayload};
use crate::notification::NotificationManager;
use crate::state::{AgentStatus, AppState};
use crate::tray;

const BIND_ADDR: &str = "127.0.0.1:19199";

/// Server 共用狀態
struct ServerState {
    app_handle: AppHandle,
    notification_manager: NotificationManager,
}

/// 純邏輯測試用狀態（不含 Tauri）
#[cfg(test)]
type TestState = Arc<AppState>;

/// 事件處理結果（純資料，可測試）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResult {
    pub ok: bool,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AgentStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 純邏輯：處理 hook 事件，回傳結果和 HTTP 狀態碼（無副作用）
pub fn handle_hook_event(
    app_state: &AppState,
    payload: &HookPayload,
) -> (StatusCode, EventResult) {
    match process_hook_event(app_state, payload) {
        Ok(new_status) => (
            StatusCode::OK,
            EventResult {
                ok: true,
                session_id: payload.session_id.clone(),
                status: Some(new_status),
                error: None,
            },
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            EventResult {
                ok: false,
                session_id: payload.session_id.clone(),
                status: None,
                error: Some(e),
            },
        ),
    }
}

/// 啟動 HTTP server（由 lib.rs setup() 呼叫）
pub async fn start_server(app_handle: AppHandle) {
    let server_state = Arc::new(ServerState {
        app_handle,
        notification_manager: NotificationManager::default(),
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/claude-code/event", post(claude_code_event))
        .with_state(server_state);

    let listener = match TcpListener::bind(BIND_ADDR).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("HTTP server bind 失敗 ({}): {}", BIND_ADDR, e);
            return;
        }
    };

    tracing::info!("Code Buddy HTTP server 啟動於 {}", BIND_ADDR);

    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("HTTP server 錯誤: {}", e);
    }
}

/// GET /health
async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": "0.2.0"
    }))
}

/// POST /claude-code/event
async fn claude_code_event(
    AxumState(server): AxumState<Arc<ServerState>>,
    Json(payload): Json<HookPayload>,
) -> impl IntoResponse {
    let app_state = server.app_handle.state::<AppState>();

    // 純邏輯處理
    let (status_code, result) = handle_hook_event(&app_state, &payload);

    // Tauri 副作用（僅在成功時執行）
    if result.ok {
        let project_name = {
            let sessions = app_state.sessions.lock().unwrap_or_else(|e| e.into_inner());
            sessions
                .get(&payload.session_id)
                .map(|s| s.project_name.clone())
                .unwrap_or_default()
        };

        let effective = app_state.effective_status();
        if let Err(e) = tray::update_tray_icon(&server.app_handle, &effective) {
            tracing::error!("Tray icon 更新失敗: {}", e);
        }

        if let Some(new_status) = result.status {
            server.notification_manager.try_notify(
                &server.app_handle,
                &payload.session_id,
                &project_name,
                new_status,
            );
        }

        let sessions_snapshot: Vec<_> = {
            let sessions = app_state.sessions.lock().unwrap_or_else(|e| e.into_inner());
            sessions.values().cloned().collect()
        };
        let _ = server.app_handle.emit(
            "state-changed",
            json!({
                "session_id": payload.session_id,
                "status": result.status,
                "effective_status": effective,
                "sessions": sessions_snapshot,
            }),
        );
    }

    (status_code, Json(result))
}

/// 建立測試用 Router（僅純邏輯，無 Tauri 副作用）
#[cfg(test)]
fn test_router(app_state: Arc<AppState>) -> Router {
    async fn test_event(
        AxumState(state): AxumState<TestState>,
        Json(payload): Json<HookPayload>,
    ) -> impl IntoResponse {
        let (status_code, result) = handle_hook_event(&state, &payload);
        (status_code, Json(result))
    }

    Router::new()
        .route("/health", get(health))
        .route("/claude-code/event", post(test_event))
        .with_state(app_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use hyper::Request;
    use tower::ServiceExt;

    // === AC-5: /health 端點 ===

    #[tokio::test]
    async fn health_returns_ok_with_version() {
        let state = Arc::new(AppState::default());
        let app = test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["version"], "0.2.0");
    }

    // === AC-5: /claude-code/event 端點 ===

    #[tokio::test]
    async fn event_accepts_valid_session_start() {
        let state = Arc::new(AppState::default());
        let app = test_router(state.clone());

        let payload = serde_json::json!({
            "hook_event_name": "SessionStart",
            "session_id": "test-001",
            "project_path": "/Users/dev/my-project"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/claude-code/event")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["session_id"], "test-001");
        assert_eq!(json["status"], "idle");

        // 確認 session 已註冊
        let sessions = state.sessions.lock().unwrap();
        assert!(sessions.contains_key("test-001"));
    }

    #[tokio::test]
    async fn event_returns_working_on_post_tool_use() {
        let state = Arc::new(AppState::default());
        let app = test_router(state.clone());

        let payload = serde_json::json!({
            "hook_event_name": "PostToolUse",
            "session_id": "test-002",
            "project_path": "/Users/dev/my-project"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/claude-code/event")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "working");
    }

    #[tokio::test]
    async fn event_rejects_unknown_hook_event() {
        let state = Arc::new(AppState::default());
        let app = test_router(state);

        let payload = serde_json::json!({
            "hook_event_name": "FakeEvent",
            "session_id": "test-003",
            "project_path": "/path"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/claude-code/event")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["ok"], false);
        assert!(json["error"].as_str().unwrap().contains("未知"));
    }

    #[tokio::test]
    async fn event_rejects_malformed_json() {
        let state = Arc::new(AppState::default());
        let app = test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/claude-code/event")
                    .header("Content-Type", "application/json")
                    .body(Body::from("{invalid json}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        // axum 會回傳 422 (Unprocessable Entity) 或 400
        assert!(
            response.status() == StatusCode::BAD_REQUEST
                || response.status() == StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[tokio::test]
    async fn event_full_lifecycle_session_start_to_end() {
        let state = Arc::new(AppState::default());

        // SessionStart
        let app = test_router(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/claude-code/event")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "hook_event_name": "SessionStart",
                            "session_id": "lifecycle-001",
                            "project_path": "/Users/dev/my-app"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // PostToolUse → working
        let app = test_router(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/claude-code/event")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "hook_event_name": "PostToolUse",
                            "session_id": "lifecycle-001",
                            "project_path": "/Users/dev/my-app"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "working");

        // Stop → completed
        let app = test_router(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/claude-code/event")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "hook_event_name": "Stop",
                            "session_id": "lifecycle-001",
                            "project_path": "/Users/dev/my-app"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "completed");

        // SessionEnd → 移除
        let app = test_router(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/claude-code/event")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "hook_event_name": "SessionEnd",
                            "session_id": "lifecycle-001",
                            "project_path": "/Users/dev/my-app"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let sessions = state.sessions.lock().unwrap();
        assert!(!sessions.contains_key("lifecycle-001"));
    }

    // === AC-5: BIND_ADDR 安全性 ===

    #[test]
    fn server_binds_to_localhost_only() {
        assert!(BIND_ADDR.starts_with("127.0.0.1:"));
    }

    // === handle_hook_event 純邏輯 ===

    #[test]
    fn handle_hook_event_returns_ok_for_valid_event() {
        let state = AppState::default();
        let payload = HookPayload {
            hook_event_name: "SessionStart".to_string(),
            session_id: "s1".to_string(),
            project_path: "/path/proj".to_string(),
            notification_type: None,
            raw: None,
        };
        let (code, result) = handle_hook_event(&state, &payload);
        assert_eq!(code, StatusCode::OK);
        assert!(result.ok);
        assert_eq!(result.status, Some(AgentStatus::Idle));
    }

    #[test]
    fn handle_hook_event_returns_bad_request_for_unknown() {
        let state = AppState::default();
        let payload = HookPayload {
            hook_event_name: "Bogus".to_string(),
            session_id: "s1".to_string(),
            project_path: "/path".to_string(),
            notification_type: None,
            raw: None,
        };
        let (code, result) = handle_hook_event(&state, &payload);
        assert_eq!(code, StatusCode::BAD_REQUEST);
        assert!(!result.ok);
        assert!(result.error.is_some());
    }
}
