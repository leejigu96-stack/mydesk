// MyDesk - 메인 진입점
// 4개의 창을 띄우고 시스템 트레이로 관리

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewWindow,
};

mod config;
mod performance;
mod window_styler;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            toggle_module,
            get_module_status,
            get_system_stats,
            apply_window_corners,
            launch_app,
        ])
        .setup(|app| {
            // 시스템 트레이 생성
            let show_settings = MenuItem::with_id(app, "settings", "설정 열기", true, None::<&str>)?;
            let toggle_dock = MenuItem::with_id(app, "toggle_dock", "독 켜기/끄기", true, None::<&str>)?;
            let toggle_widgets = MenuItem::with_id(app, "toggle_widgets", "위젯 켜기/끄기", true, None::<&str>)?;
            let toggle_wallpaper = MenuItem::with_id(app, "toggle_wallpaper", "배경 켜기/끄기", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "종료", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_settings, &toggle_dock, &toggle_widgets, &toggle_wallpaper, &quit])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => show_window(app, "settings"),
                    "toggle_dock" => toggle_window(app, "dock"),
                    "toggle_widgets" => toggle_window(app, "widgets"),
                    "toggle_wallpaper" => toggle_window(app, "wallpaper"),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        show_window(app, "settings");
                    }
                })
                .build(app)?;

            // 설정 따라 자동으로 켜는 창들 표시
            let cfg = config::load(app.handle());
            if cfg.dock_enabled {
                position_dock(app.handle());
                if let Some(w) = app.get_webview_window("dock") {
                    let _ = w.show();
                }
            }
            if cfg.widgets_enabled {
                position_widgets(app.handle());
                if let Some(w) = app.get_webview_window("widgets") {
                    let _ = w.show();
                }
            }
            if cfg.wallpaper_enabled {
                if let Some(w) = app.get_webview_window("wallpaper") {
                    let _ = w.show();
                    // 배경은 모든 창 뒤로
                    let _ = window_styler::send_to_back(&w);
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("MyDesk 시작 실패");
}

// ==================== 명령 (프론트엔드에서 호출) ====================

#[tauri::command]
fn toggle_module(app: tauri::AppHandle, module: String, enable: bool) -> Result<(), String> {
    let mut cfg = config::load(&app);
    match module.as_str() {
        "dock" => {
            cfg.dock_enabled = enable;
            if let Some(w) = app.get_webview_window("dock") {
                if enable { let _ = w.show(); } else { let _ = w.hide(); }
            }
        }
        "widgets" => {
            cfg.widgets_enabled = enable;
            if let Some(w) = app.get_webview_window("widgets") {
                if enable { let _ = w.show(); } else { let _ = w.hide(); }
            }
        }
        "wallpaper" => {
            cfg.wallpaper_enabled = enable;
            if let Some(w) = app.get_webview_window("wallpaper") {
                if enable { let _ = w.show(); } else { let _ = w.hide(); }
            }
        }
        _ => return Err(format!("알 수 없는 모듈: {}", module)),
    }
    config::save(&app, &cfg);
    Ok(())
}

#[tauri::command]
fn get_module_status(app: tauri::AppHandle) -> serde_json::Value {
    let cfg = config::load(&app);
    serde_json::json!({
        "dock": cfg.dock_enabled,
        "widgets": cfg.widgets_enabled,
        "wallpaper": cfg.wallpaper_enabled,
        "rounded_corners": cfg.rounded_corners_enabled,
    })
}

#[tauri::command]
fn get_system_stats() -> serde_json::Value {
    performance::get_stats()
}

#[tauri::command]
fn apply_window_corners(enable: bool, radius: u32) -> Result<(), String> {
    window_styler::apply_corners(enable, radius)
}

#[tauri::command]
fn launch_app(path: String) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(&["/C", "start", "", &path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 헬퍼 ====================

fn show_window(app: &tauri::AppHandle, label: &str) {
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.show();
        let _ = w.set_focus();
        let _ = w.unminimize();
    }
}

fn toggle_window(app: &tauri::AppHandle, label: &str) {
    if let Some(w) = app.get_webview_window(label) {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            let _ = w.show();
        }
    }
}

fn position_dock(app: &tauri::AppHandle) {
    // 화면 아래 중앙에 배치
    if let Some(w) = app.get_webview_window("dock") {
        if let Ok(monitor) = w.current_monitor() {
            if let Some(m) = monitor {
                let size = m.size();
                let dock_size = w.outer_size().unwrap_or_default();
                let x = (size.width as i32 - dock_size.width as i32) / 2;
                let y = size.height as i32 - dock_size.height as i32 - 8;
                let _ = w.set_position(tauri::PhysicalPosition { x, y });
            }
        }
    }
}

fn position_widgets(app: &tauri::AppHandle) {
    // 화면 오른쪽 위에 배치
    if let Some(w) = app.get_webview_window("widgets") {
        if let Ok(monitor) = w.current_monitor() {
            if let Some(m) = monitor {
                let size = m.size();
                let widget_size = w.outer_size().unwrap_or_default();
                let x = size.width as i32 - widget_size.width as i32 - 20;
                let y = 20;
                let _ = w.set_position(tauri::PhysicalPosition { x, y });
            }
        }
    }
}
