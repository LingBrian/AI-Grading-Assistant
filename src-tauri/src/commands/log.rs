use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn log_write(state: State<AppState>, level: String, message: String, source: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.write_log(&level, &message, &source);
    Ok(())
}

#[tauri::command]
pub fn log_list(state: State<AppState>, limit: Option<i64>) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(db.list_logs(limit.unwrap_or(200)))
}

#[tauri::command]
pub fn log_clear(state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_logs();
    Ok(())
}
