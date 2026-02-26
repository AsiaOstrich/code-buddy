use axum::{
    extract::State as AxumState,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::TcpListener;

use crate::adapters::claude_code::{process_hook_event, HookPayload};
use crate::notification::NotificationManager;
use crate::state::AppState;
use crate::tray;

const BIND_ADDR: &str = "127.0.0.1:19199";

/// Server 共用狀態
struct ServerState {
    app_handle: AppHandle,
    notification_manager: NotificationManager,
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
            eprintln!("HTTP server bind 失敗 ({}): {}", BIND_ADDR, e);
            return;
        }
    };

    println!("Code Buddy HTTP server 啟動於 {}", BIND_ADDR);

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("HTTP server 錯誤: {}", e);
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

    // 處理 hook 事件
    match process_hook_event(&app_state, &payload) {
        Ok(new_status) => {
            // 取得 project_name（用於通知）
            let project_name = {
                let sessions = app_state.sessions.lock().unwrap();
                sessions
                    .get(&payload.session_id)
                    .map(|s| s.project_name.clone())
                    .unwrap_or_default()
            };

            // 更新 tray icon（使用 effective_status）
            let effective = app_state.effective_status();
            if let Err(e) = tray::update_tray_icon(&server.app_handle, &effective) {
                eprintln!("Tray icon 更新失敗: {}", e);
            }

            // 推送通知
            server.notification_manager.try_notify(
                &server.app_handle,
                &payload.session_id,
                &project_name,
                new_status,
            );

            // Emit state-changed 事件給前端
            let sessions_snapshot: Vec<_> = {
                let sessions = app_state.sessions.lock().unwrap();
                sessions.values().cloned().collect()
            };
            let _ = server.app_handle.emit(
                "state-changed",
                json!({
                    "session_id": payload.session_id,
                    "status": new_status,
                    "effective_status": effective,
                    "sessions": sessions_snapshot,
                }),
            );

            (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "session_id": payload.session_id,
                    "status": new_status,
                })),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": e,
            })),
        ),
    }
}
