use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub api_key: String,
    pub api_key_presets: std::collections::HashMap<String, String>,
    pub base_url: String,
    pub model: String,
    pub shortcut: String,
    pub theme: String,
    pub font_size: i32,
    pub proxy: String,
    pub timeout: i32,
    pub screenshot_save_path: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            api_key: String::new(),
            api_key_presets: std::collections::HashMap::new(),
            base_url: "https://api.siliconflow.cn/v1".to_string(),
            model: String::new(),
            shortcut: "Alt+Q".to_string(),
            theme: "light".to_string(),
            font_size: 14,
            proxy: String::new(),
            timeout: 120,
            screenshot_save_path: String::new(),
        }
    }
}

#[tauri::command]
pub async fn settings_get(app: AppHandle) -> Result<AppSettings, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    Ok(AppSettings {
        api_key: store.get("apiKey").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default(),
        api_key_presets: store.get("apiKeyPresets").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default(),
        base_url: store.get("baseUrl").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_else(|| "https://api.siliconflow.cn/v1".to_string()),
        model: store.get("model").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default(),
        shortcut: store.get("shortcut").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_else(|| "Alt+Q".to_string()),
        theme: store.get("theme").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_else(|| "light".to_string()),
        font_size: store.get("fontSize").and_then(|v| v.as_i64().map(|i| i as i32)).unwrap_or(14),
        proxy: store.get("proxy").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default(),
        timeout: store.get("timeout").and_then(|v| v.as_i64().map(|i| {
            let t = i as i32;
            if t < 60 { 120 } else { t }
        })).unwrap_or(120),
        screenshot_save_path: store.get("screenshotSavePath").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default(),
    })
}

#[tauri::command]
pub async fn settings_set(app: AppHandle, settings: AppSettings) -> Result<bool, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("apiKey", serde_json::Value::String(settings.api_key));
    store.set("apiKeyPresets", serde_json::to_value(&settings.api_key_presets).unwrap_or(serde_json::Value::Object(Default::default())));
    store.set("baseUrl", serde_json::Value::String(settings.base_url));
    store.set("model", serde_json::Value::String(settings.model));
    store.set("shortcut", serde_json::Value::String(settings.shortcut));
    store.set("theme", serde_json::Value::String(settings.theme));
    store.set("fontSize", serde_json::Value::Number(serde_json::Number::from(settings.font_size)));
    store.set("proxy", serde_json::Value::String(settings.proxy));
    store.set("timeout", serde_json::Value::Number(serde_json::Number::from(settings.timeout)));
    store.set("screenshotSavePath", serde_json::Value::String(settings.screenshot_save_path));
    store.save().map_err(|e| e.to_string())?;
    Ok(true)
}
