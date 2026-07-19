use crate::get_screenshot_window;
use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[tauri::command]
pub async fn screenshot_start(app: AppHandle) -> Result<(), String> {
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.hide();
    }
    get_screenshot_window(&app).map_err(|e| e.to_string())?;
    Ok(())
}

fn get_monitor_at_cursor(app: &AppHandle) -> Result<xcap::Monitor, String> {
    let monitors = xcap::Monitor::all().map_err(|e| format!("Failed to list monitors: {}", e))?;
    let cursor = app.get_webview_window("main").and_then(|w| w.outer_position().ok());
    for m in monitors {
        let mx = m.x().map_err(|e| format!("Monitor x error: {}", e))?;
        let my = m.y().map_err(|e| format!("Monitor y error: {}", e))?;
        let mw = m.width().map_err(|e| format!("Monitor width error: {}", e))? as i32;
        let mh = m.height().map_err(|e| format!("Monitor height error: {}", e))? as i32;
        if let Some(c) = cursor {
            let cx = c.x as i32;
            let cy = c.y as i32;
            if cx >= mx && cx < (mx + mw) && cy >= my && cy < (my + mh) {
                return Ok(m);
            }
        }
    }
    xcap::Monitor::all()
        .map_err(|e| format!("Failed to list monitors: {}", e))?
        .into_iter()
        .next()
        .ok_or_else(|| "No monitor found".to_string())
}

#[tauri::command]
pub async fn screenshot_capture(app: AppHandle, region: CaptureRegion) -> Result<Option<String>, String> {
    let target = get_monitor_at_cursor(&app)?;

    let img = target.capture_image().map_err(|e| format!("Failed to capture screen: {}", e))?;
    let (img_w, img_h) = (img.width() as f64, img.height() as f64);
    let (disp_w, disp_h) = (
        target.width().map_err(|e| format!("Monitor width error: {}", e))? as f64,
        target.height().map_err(|e| format!("Monitor height error: {}", e))? as f64,
    );

    let ratio_x = img_w / disp_w;
    let ratio_y = img_h / disp_h;

    let crop_x = (region.x as f64 * ratio_x).max(0.0) as u32;
    let crop_y = (region.y as f64 * ratio_y).max(0.0) as u32;
    let crop_w = (region.width as f64 * ratio_x).min((img_w - crop_x as f64).max(0.0)) as u32;
    let crop_h = (region.height as f64 * ratio_y).min((img_h - crop_y as f64).max(0.0)) as u32;

    let cropped = image::DynamicImage::ImageRgba8(img).crop_imm(crop_x, crop_y, crop_w, crop_h);
    let mut png_bytes: Vec<u8> = Vec::new();
    cropped.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;

    let base64_str = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

    if let Ok(store) = tauri_plugin_store::StoreExt::store(&app, "settings.json") {
        if let Some(path) = store.get("screenshotSavePath").and_then(|v| v.as_str().map(|s| s.to_string())) {
            if !path.is_empty() {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
                let filename = format!("screenshot_{}.png", timestamp);
                let full_path = std::path::Path::new(&path).join(&filename);
                let _ = std::fs::write(&full_path, &png_bytes);
            }
        }
    }

    if let Some(sw) = app.get_webview_window("screenshot") {
        let _ = sw.close();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
    let _ = app.emit("screenshot-completed", &base64_str);
    Ok(Some(base64_str))
}

#[tauri::command]
pub async fn screenshot_cancel(app: AppHandle) -> Result<(), String> {
    if let Some(sw) = app.get_webview_window("screenshot") {
        let _ = sw.close();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
    Ok(())
}
