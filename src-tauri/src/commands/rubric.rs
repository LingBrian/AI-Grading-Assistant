use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RubricData {
    pub questions: Vec<QuestionItem>,
    pub current_index: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionItem {
    pub question_title: String,
    pub max_score: f64,
    pub reference_answer: String,
    pub rubric: String,
    pub reference_answer_image: Option<String>,
}

#[tauri::command]
pub fn rubric_store_data(app: AppHandle, state: State<AppState>, data: RubricData) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_rubric();
    for (i, q) in data.questions.iter().enumerate() {
        db.insert_rubric_question(&serde_json::json!(q), i as i32);
    }
    drop(db);
    let _ = app.emit("rubric-updated", ());
    Ok(())
}

#[tauri::command]
pub fn rubric_get_data(state: State<AppState>) -> Result<Option<serde_json::Value>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let questions = db.get_rubric_questions();
    if questions.is_empty() {
        Ok(None)
    } else {
        Ok(Some(serde_json::json!({
            "questions": questions,
            "currentIndex": 0
        })))
    }
}

#[tauri::command]
pub async fn rubric_export_json(app: AppHandle, state: State<'_, AppState>) -> Result<bool, String> {
    let questions = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_rubric_questions()
    };
    if questions.is_empty() {
        return Err("暂无评分标准可导出".to_string());
    }
    let file = app.dialog()
        .file()
        .add_filter("JSON", &["json"])
        .set_file_name("评分标准.json")
        .blocking_save_file();
    match file {
        Some(path) => {
            let path_str = path.as_path().unwrap().to_string_lossy().to_string();
            let content = serde_json::to_string_pretty(&questions).map_err(|e| e.to_string())?;
            std::fs::write(&path_str, &content).map_err(|e| e.to_string())?;
            Ok(true)
        }
        None => Ok(false),
    }
}

#[tauri::command]
pub async fn rubric_import_json(app: AppHandle) -> Result<Option<serde_json::Value>, String> {
    let file = app.dialog()
        .file()
        .add_filter("JSON", &["json"])
        .blocking_pick_file();
    match file {
        Some(path) => {
            let path_str = path.as_path().unwrap().to_string_lossy().to_string();
            let content = std::fs::read_to_string(&path_str).map_err(|e| e.to_string())?;
            let questions: Vec<serde_json::Value> = serde_json::from_str(&content).map_err(|e| format!("JSON 格式错误：{}", e))?;
            Ok(Some(serde_json::json!({
                "questions": questions,
                "currentIndex": 0
            })))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub fn rubric_get_next_index(state: State<AppState>) -> Result<i32, String> {
    let idx = state.rubric_next_index.lock().map_err(|e| e.to_string())?;
    Ok(*idx)
}

#[tauri::command]
pub fn rubric_set_next_index(state: State<AppState>, idx: i32) -> Result<(), String> {
    let mut next = state.rubric_next_index.lock().map_err(|e| e.to_string())?;
    *next = idx;
    Ok(())
}

#[tauri::command]
pub fn rubric_reset_next_index(state: State<AppState>) -> Result<(), String> {
    let mut next = state.rubric_next_index.lock().map_err(|e| e.to_string())?;
    *next = 0;
    Ok(())
}
