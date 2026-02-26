use tauri::State;

use crate::state::{AgentStatus, AppState};
use crate::tray;

#[tauri::command]
pub fn switch_tray_icon(
    app: tauri::AppHandle,
    state: State<AppState>,
    status: AgentStatus,
) -> Result<String, String> {
    tray::update_tray_icon(&app, &status).map_err(|e| e.to_string())?;
    let mut current = state.current_status.lock().map_err(|e| e.to_string())?;
    *current = status;
    Ok(format!("Tray icon switched to {:?}", status))
}

#[tauri::command]
pub fn get_current_status(state: State<AppState>) -> Result<AgentStatus, String> {
    let current = state.current_status.lock().map_err(|e| e.to_string())?;
    Ok(*current)
}
