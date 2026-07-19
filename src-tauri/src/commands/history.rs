use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn history_list(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(db.list_history())
}

#[tauri::command]
pub fn history_delete(state: State<AppState>, id: i64) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(db.delete_history(id))
}

#[tauri::command]
pub fn history_get_by_id(state: State<AppState>, id: i64) -> Result<Option<serde_json::Value>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(db.get_history_by_id(id))
}
