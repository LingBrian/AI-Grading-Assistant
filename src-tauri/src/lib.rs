mod ai;
mod commands;
mod db;

use db::Database;
use std::sync::Mutex;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

pub struct AppState {
    pub db: Mutex<Database>,
    pub grading_payload: Mutex<Option<String>>,
    pub rubric_next_index: Mutex<i32>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let db = Database::new(app_handle.path().app_data_dir()?.join("grading.db"));
            app.manage(AppState {
                db: Mutex::new(db),
                grading_payload: Mutex::new(None),
                rubric_next_index: Mutex::new(0),
            });

            // tray icon
            let show_item = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
            let img = image::load_from_memory(include_bytes!("../icons/icon.png"))
                .expect("Failed to decode tray icon")
                .into_rgba8();
            let (iw, ih) = img.dimensions();
            let icon = Image::new_owned(img.into_raw(), iw, ih);
            TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("AI 阅卷助手")
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            let _ = create_main_window(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::settings_get,
            commands::settings::settings_set,
            commands::screenshot::screenshot_start,
            commands::screenshot::screenshot_capture,
            commands::screenshot::screenshot_cancel,
            commands::ai_grade,
            commands::history::history_list,
            commands::history::history_delete,
            commands::history::history_get_by_id,
            commands::grading::grading_submit,
            commands::grading::grading_store_data,
            commands::grading::grading_get_data,
            commands::rubric::rubric_store_data,
            commands::rubric::rubric_get_data,
            commands::rubric::rubric_export_json,
            commands::rubric::rubric_import_json,
            commands::rubric::rubric_get_next_index,
            commands::rubric::rubric_set_next_index,
            commands::rubric::rubric_reset_next_index,
            commands::template::template_list,
            commands::template::template_save,
            commands::template::template_delete,
            commands::template::template_apply,
            commands::template::template_export_json,
            commands::template::template_import_json,
            commands::dialog_open,
            commands::dialog_open_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn create_main_window(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let (wx, wy) = app.primary_monitor()
        .ok()
        .flatten()
        .map(|m| {
            let scale = m.scale_factor();
            let w = m.size().width as f64 / scale;
            let x = (w - 600.0) / 2.0 + m.position().x as f64 / scale;
            let y = m.position().y as f64 / scale;
            (x, y)
        })
        .unwrap_or((0.0, 0.0));

    let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("AI 阅卷助手")
        .inner_size(600.0, 64.0)
        .min_inner_size(200.0, 64.0)
        .max_inner_size(1200.0, 76.0)
        .decorations(false)
        .always_on_top(true)
        .resizable(false)
        .build()?;
    window.set_position(PhysicalPosition::new(wx as i32, wy as i32))?;
    Ok(())
}

pub fn create_dialog_window(
    app: &tauri::AppHandle,
    dialog_type: &str,
    width: f64,
    height: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let label = format!("dialog-{}", dialog_type);
    if let Some(window) = app.get_webview_window(&label) {
        window.set_focus()?;
        return Ok(());
    }
    let (dx, dy) = get_dialog_position(app, width, height);
    WebviewWindowBuilder::new(app, &label, WebviewUrl::App(format!("index.html?dialog={}", dialog_type).into()))
        .title(match dialog_type {
            "settings" => "设置",
            "history" => "历史记录",
            "templates" => "模板管理",
            "rubric" => "评分标准",
            "result" => "评分结果",
            _ => "AI 阅卷助手",
        })
        .inner_size(width, height)
        .position(dx, dy)
        .decorations(false)
        .resizable(false)
        .build()?;
    Ok(())
}

fn get_dialog_position(app: &tauri::AppHandle, width: f64, _height: f64) -> (f64, f64) {
    if let Some(main) = app.get_webview_window("main") {
        if let (Ok(pos), Ok(size)) = (main.outer_position(), main.outer_size()) {
            let mx = pos.x as f64;
            let my = pos.y as f64;
            let mw = size.width as f64;
            let mh = size.height as f64;
            let dx = mx + (mw - width) / 2.0;
            let dy = my + mh;
            return (dx.max(0.0), dy.max(0.0));
        }
    }
    (0.0, 0.0)
}

fn get_screenshot_window(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(sw) = app.get_webview_window("screenshot") {
        sw.set_focus()?;
        return Ok(());
    }
    let all_monitors = xcap::Monitor::all()?;
    let cursor = app.get_webview_window("main").and_then(|w| w.outer_position().ok());
    let monitor = all_monitors.into_iter().find(|m| {
        if let (Ok(mx), Ok(my)) = (m.x(), m.y()) {
            if let (Ok(mw), Ok(mh)) = (m.width(), m.height()) {
                if let Some(c) = cursor {
                    let cx = c.x as i32;
                    let cy = c.y as i32;
                    return cx >= mx && cx < (mx + mw as i32) && cy >= my && cy < (my + mh as i32);
                }
            }
        }
        false
    }).unwrap_or_else(|| {
        xcap::Monitor::all().ok().and_then(|m| m.into_iter().next())
            .expect("No monitor found")
    });

    let scale = monitor.scale_factor().unwrap_or(1.0) as f64;
    let w = monitor.width().unwrap_or(1920) as f64 / scale;
    let h = monitor.height().unwrap_or(1080) as f64 / scale;
    let x = monitor.x().unwrap_or(0) as f64;
    let y = monitor.y().unwrap_or(0) as f64;

    WebviewWindowBuilder::new(app, "screenshot", WebviewUrl::App("index.html#/screenshot".into()))
        .title("截图")
        .inner_size(w, h)
        .position(x, y)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .build()?;
    Ok(())
}
