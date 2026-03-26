use serde_json::json;
use tauri::{Emitter, State};

use crate::float;
use crate::state::{AgentStatus, AppState, SessionInfo};
use crate::tray;

#[tauri::command]
pub fn switch_tray_icon(
    app: tauri::AppHandle,
    _state: State<AppState>,
    status: AgentStatus,
) -> Result<String, String> {
    tray::update_tray_icon(&app, &status).map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    if let Err(e) = crate::dock::update_dock_icon(&app, &status) {
        tracing::error!("Dock icon 更新失敗: {}", e);
    }

    // Emit event 讓浮動視窗也收到狀態變化
    let _ = app.emit(
        "state-changed",
        json!({
            "session_id": "manual",
            "status": format!("{:?}", status).to_lowercase(),
            "effective_status": status,
            "sessions": [],
        }),
    );

    Ok(format!("Tray icon switched to {:?}", status))
}

#[tauri::command]
pub fn get_current_status(state: State<AppState>) -> Result<AgentStatus, String> {
    Ok(state.effective_status())
}

#[tauri::command]
pub fn get_sessions(state: State<AppState>) -> Result<Vec<SessionInfo>, String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    Ok(sessions.values().cloned().collect())
}

#[tauri::command]
pub fn toggle_float_window(app: tauri::AppHandle) -> Result<bool, String> {
    float::toggle_float_window(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_float_opacity(app: tauri::AppHandle, opacity: f64) -> Result<(), String> {
    float::set_opacity(&app, opacity).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_float_position(app: tauri::AppHandle) -> Result<(), String> {
    float::save_position(&app).map_err(|e| e.to_string())
}
