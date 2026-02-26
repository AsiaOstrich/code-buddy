use tauri::State;

use crate::state::{AgentStatus, AppState, SessionInfo};
use crate::tray;

#[tauri::command]
pub fn switch_tray_icon(
    app: tauri::AppHandle,
    _state: State<AppState>,
    status: AgentStatus,
) -> Result<String, String> {
    tray::update_tray_icon(&app, &status).map_err(|e| e.to_string())?;
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
