use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradingSubmitData {
    pub question_id: Option<i64>,
    pub image_path: Option<String>,
    pub ai_score: f64,
    pub teacher_score: f64,
    pub confidence: f64,
    pub student_answer: Option<String>,
    pub question_title: Option<String>,
    pub max_score: Option<f64>,
    pub reference_answer: Option<String>,
    pub rubric: Option<String>,
    pub deductions: Option<Vec<String>>,
    pub comment: Option<String>,
    pub result_json: Option<String>,
    pub image_data: Option<String>,
    pub reference_answer_image: Option<String>,
}

#[tauri::command]
pub fn grading_submit(state: State<AppState>, data: GradingSubmitData) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let json = serde_json::json!({
        "questionId": data.question_id.unwrap_or(0),
        "imagePath": data.image_path.unwrap_or_default(),
        "studentAnswer": data.student_answer.unwrap_or_default(),
        "aiScore": data.ai_score,
        "teacherScore": data.teacher_score,
        "confidence": data.confidence,
        "questionTitle": data.question_title.unwrap_or_default(),
        "maxScore": data.max_score.unwrap_or(100.0),
        "referenceAnswer": data.reference_answer.unwrap_or_default(),
        "rubric": data.rubric.unwrap_or_default(),
        "deductions": serde_json::to_string(&data.deductions.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string()),
        "comment": data.comment.unwrap_or_default(),
        "resultJson": data.result_json.unwrap_or_default(),
        "imageData": data.image_data.unwrap_or_default(),
        "referenceAnswerImage": data.reference_answer_image.unwrap_or_default(),
    });
    Ok(db.insert_history(&json))
}

#[tauri::command]
pub fn grading_store_data(state: State<AppState>, data: serde_json::Value) -> Result<(), String> {
    let mut payload = state.grading_payload.lock().map_err(|e| e.to_string())?;
    *payload = Some(data.to_string());
    Ok(())
}

#[tauri::command]
pub fn grading_get_data(state: State<AppState>) -> Result<Option<serde_json::Value>, String> {
    let mut payload = state.grading_payload.lock().map_err(|e| e.to_string())?;
    let data = payload.take();
    match data {
        Some(s) => serde_json::from_str(&s).map(Some).map_err(|e| e.to_string()),
        None => Ok(None),
    }
}
