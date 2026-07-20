pub mod settings;
pub mod screenshot;
pub mod history;
pub mod grading;
pub mod rubric;
pub mod template;
pub mod log;

use crate::ai;
use crate::create_dialog_window;
use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_store::StoreExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageResult {
    pub base64: String,
    pub mime: String,
    pub file_path: String,
}

#[tauri::command]
pub async fn dialog_open(app: AppHandle, dialog_type: String) -> Result<(), String> {
    let sizes: [(&str, f64, f64); 6] = [
        ("settings", 520.0, 540.0),
        ("history", 860.0, 660.0),
        ("templates", 720.0, 540.0),
        ("rubric", 650.0, 700.0),
        ("result", 620.0, 680.0),
        ("logs", 860.0, 660.0),
    ];
    let (_, w, h) = sizes.iter().find(|(t, _, _)| *t == dialog_type).unwrap_or(&("result", 600.0, 780.0));
    create_dialog_window(&app, &dialog_type, *w, *h).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dialog_open_image(app: AppHandle) -> Result<Option<ImageResult>, String> {
    let file = app.dialog()
        .file()
        .add_filter("图片", &["png", "jpg", "jpeg", "bmp", "gif", "webp"])
        .blocking_pick_file();
    match file {
        Some(path) => {
            let path_str = path.as_path().unwrap().to_string_lossy().to_string();
            let ext = std::path::Path::new(&path_str)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png")
                .to_lowercase();
            let mime = if ext == "jpg" { "jpeg".to_string() } else { ext.clone() };
            let bytes = std::fs::read(&path_str).map_err(|e| format!("Failed to read file: {}", e))?;
            let base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            Ok(Some(ImageResult { base64, mime, file_path: path_str }))
        }
        None => Ok(None),
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeRequest {
    pub image_base64: String,
    pub rubric: String,
    pub api_config: ai::ApiConfig,
    pub max_score: Option<f64>,
    pub reference_answer: Option<String>,
    pub question_title: Option<String>,
    pub reference_answer_image: Option<String>,
}

#[tauri::command]
pub async fn ai_grade(app: AppHandle, request: GradeRequest) -> Result<ai::GradeResult, String> {
    let timeout = app.store("settings.json")
        .ok()
        .and_then(|store| store.get("timeout").and_then(|v| v.as_i64().map(|i| i as u64)))
        .unwrap_or(120);

    ai::call_ai_api(
        &request.image_base64,
        &request.rubric,
        &request.api_config,
        request.max_score,
        request.reference_answer.as_deref(),
        request.question_title.as_deref(),
        request.reference_answer_image.as_deref(),
        timeout,
    )
    .await
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchModelsRequest {
    pub api_key: String,
    pub base_url: String,
}

#[tauri::command]
pub async fn ai_fetch_models(request: FetchModelsRequest) -> Result<Vec<String>, String> {
    let url = format!("{}/models", request.base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", request.api_key))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("API 返回错误 ({}): {}", status, text));
    }

    let json: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析返回数据失败: {}", e))?;

    let models = json["data"]
        .as_array()
        .ok_or_else(|| "返回数据格式异常：缺少 data 字段".to_string())?;

    let ids: Vec<String> = models
        .iter()
        .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
        .collect();

    if ids.is_empty() {
        return Err("未获取到任何模型".to_string());
    }

    Ok(ids)
}
