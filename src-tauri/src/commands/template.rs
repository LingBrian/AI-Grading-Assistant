use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateSaveData {
    pub name: String,
    pub course: String,
    pub questions: Option<Vec<serde_json::Value>>,
}

#[tauri::command]
pub fn template_list(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(db.list_templates())
}

#[tauri::command]
pub fn template_save(state: State<AppState>, data: TemplateSaveData) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let questions_json = serde_json::to_string(&data.questions.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string());
    Ok(db.save_template(&data.name, &data.course, &questions_json))
}

#[tauri::command]
pub fn template_delete(state: State<AppState>, id: i64) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(db.delete_template(id))
}

#[tauri::command]
pub fn template_apply(app: AppHandle, state: State<AppState>, id: i64) -> Result<bool, String> {
    let tpl = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_template_by_id(id)
    };
    match tpl {
        Some(t) => {
            let questions = t["questions"].as_array().ok_or("模板中没有题目")?.clone();
            if questions.is_empty() {
                return Err("模板中没有题目".to_string());
            }
            {
                let db = state.db.lock().map_err(|e| e.to_string())?;
                db.clear_rubric();
                for (i, q) in questions.iter().enumerate() {
                    db.insert_rubric_question(q, i as i32);
                }
            }
            {
                let mut idx = state.rubric_next_index.lock().map_err(|e| e.to_string())?;
                *idx = 0;
            }
            let _ = app.emit("rubric-updated", ());
            Ok(true)
        }
        None => Err("模板不存在".to_string()),
    }
}

#[tauri::command]
pub async fn template_export_json(app: AppHandle, state: State<'_, AppState>) -> Result<bool, String> {
    let templates = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.list_templates()
    };
    if templates.is_empty() {
        return Err("暂无模板可导出".to_string());
    }
    let file = app.dialog()
        .file()
        .add_filter("JSON", &["json"])
        .set_file_name("评分模板.json")
        .blocking_save_file();
    match file {
        Some(path) => {
            let path_str = path.as_path().unwrap().to_string_lossy().to_string();
            let export_data: Vec<serde_json::Value> = templates.iter().map(|t| {
                serde_json::json!({
                    "name": t["name"],
                    "course": t["course"],
                    "questions": t["questions"],
                    "created_at": t["created_at"]
                })
            }).collect();
            let content = serde_json::to_string_pretty(&export_data).map_err(|e| e.to_string())?;
            std::fs::write(&path_str, &content).map_err(|e| e.to_string())?;
            Ok(true)
        }
        None => Ok(false),
    }
}

#[tauri::command]
pub async fn template_import_json(app: AppHandle, state: State<'_, AppState>) -> Result<Option<serde_json::Value>, String> {
    let file = app.dialog()
        .file()
        .add_filter("JSON", &["json"])
        .blocking_pick_file();
    match file {
        Some(path) => {
            let path_str = path.as_path().unwrap().to_string_lossy().to_string();
            let content = std::fs::read_to_string(&path_str).map_err(|e| e.to_string())?;
            let data: Vec<serde_json::Value> = serde_json::from_str(&content).map_err(|e| format!("JSON 格式错误：应为数组 - {}", e))?;
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let mut count = 0;
            for tpl in &data {
                if let (Some(name), Some(course)) = (tpl["name"].as_str(), tpl["course"].as_str()) {
                    let questions_json = serde_json::to_string(&tpl["questions"]).unwrap_or_else(|_| "[]".to_string());
                    db.save_template(name, course, &questions_json);
                    count += 1;
                }
            }
            Ok(Some(serde_json::json!({ "count": count })))
        }
        None => Ok(None),
    }
}
