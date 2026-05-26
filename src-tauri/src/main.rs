// MyDesk - 메인 진입점
// 4개의 창을 띄우고 시스템 트레이로 관리 + 실제 기능 구현

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

mod config;
mod performance;
mod tuner;
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
            get_config,
            save_config,
            toggle_module,
            launch_app,
            launch_path,
            get_system_stats,
            get_widget_data,
            run_tuner_action,
            change_mode,
            apply_theme,
            quit_app,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();

            // 시스템 트레이
            let show_settings = MenuItem::with_id(app, "settings", "설정 열기", true, None::<&str>)?;
            let toggle_dock = MenuItem::with_id(app, "toggle_dock", "독 켜기/끄기", true, None::<&str>)?;
            let toggle_widgets = MenuItem::with_id(app, "toggle_widgets", "위젯 켜기/끄기", true, None::<&str>)?;
            let toggle_wallpaper = MenuItem::with_id(app, "toggle_wallpaper", "배경 켜기/끄기", true, None::<&str>)?;
            let separator = MenuItem::with_id(app, "_sep", "──────────", false, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "종료", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[
                &show_settings,
                &separator,
                &toggle_dock,
                &toggle_widgets,
                &toggle_wallpaper,
                &separator,
                &quit,
            ])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
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
                        show_window(tray.app_handle(), "settings");
                    }
                })
                .build(app)?;

            // 설정 따라 자동으로 켜는 창들 표시
            let cfg = config::load(&app_handle);

            if cfg.dock_enabled {
                position_dock(&app_handle);
                if let Some(w) = app.get_webview_window("dock") {
                    let _ = w.show();
                }
            }
            if cfg.widgets_enabled {
                position_widgets(&app_handle);
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
fn get_config(app: tauri::AppHandle) -> config::AppConfig {
    config::load(&app)
}

#[tauri::command]
fn save_config(app: tauri::AppHandle, cfg: config::AppConfig) -> Result<(), String> {
    config::save(&app, &cfg);
    // 다른 창에 설정 변경 알림
    let _ = app.emit("config-changed", &cfg);
    Ok(())
}

#[tauri::command]
fn toggle_module(app: tauri::AppHandle, module: String, enable: bool) -> Result<(), String> {
    let mut cfg = config::load(&app);
    match module.as_str() {
        "dock" => {
            cfg.dock_enabled = enable;
            if let Some(w) = app.get_webview_window("dock") {
                if enable {
                    position_dock(&app);
                    let _ = w.show();
                } else {
                    let _ = w.hide();
                }
            }
        }
        "widgets" => {
            cfg.widgets_enabled = enable;
            if let Some(w) = app.get_webview_window("widgets") {
                if enable {
                    position_widgets(&app);
                    let _ = w.show();
                } else {
                    let _ = w.hide();
                }
            }
        }
        "wallpaper" => {
            cfg.wallpaper_enabled = enable;
            if let Some(w) = app.get_webview_window("wallpaper") {
                if enable {
                    let _ = w.show();
                    let _ = window_styler::send_to_back(&w);
                } else {
                    let _ = w.hide();
                }
            }
        }
        _ => return Err(format!("알 수 없는 모듈: {}", module)),
    }
    config::save(&app, &cfg);
    Ok(())
}

#[tauri::command]
fn launch_app(name: String) -> Result<(), String> {
    // 자주 쓰는 앱 매핑 (한국 사용자 환경)
    let app_paths: &[(&str, &str)] = &[
        ("chrome", "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe"),
        ("claude", "claude"),
        ("resellon", "F:\\resellon\\program\\resellon-admin.exe"),
        ("nokki", "F:\\nokki\\app\\nokki-gui.exe"),
        ("vscode", "code"),
        ("photoshop", "C:\\Program Files\\Adobe\\Adobe Photoshop 2025\\Photoshop.exe"),
        ("kakao", "C:\\Program Files (x86)\\Kakao\\KakaoTalk\\KakaoTalk.exe"),
        ("files", "explorer.exe"),
        ("settings", "ms-settings:"),
        ("trash", "shell:RecycleBinFolder"),
    ];

    let target = app_paths
        .iter()
        .find(|(key, _)| *key == name.as_str())
        .map(|(_, path)| path.to_string());

    match target {
        Some(path) => {
            std::process::Command::new("cmd")
                .args(&["/C", "start", "", &path])
                .spawn()
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        None => Err(format!("등록된 앱 아님: {}", name)),
    }
}

#[tauri::command]
fn launch_path(path: String) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(&["/C", "start", "", &path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_system_stats() -> serde_json::Value {
    performance::get_stats()
}

#[derive(Serialize, Deserialize)]
pub struct WidgetData {
    pub clock_time: String,
    pub clock_date: String,
    pub sales_today: String,
    pub sales_week: String,
    pub nokki_pending: u32,
    pub nokki_done: u32,
    pub nokki_errors: u32,
    pub claude_count: u32,
    pub claude_sessions: Vec<(String, String, String)>, // (name, status, dot_color)
    pub weather_temp: String,
    pub weather_desc: String,
}

#[tauri::command]
fn get_widget_data() -> WidgetData {
    use chrono::{Datelike, Local, Timelike};

    let now = Local::now();
    let weekdays = ["일", "월", "화", "수", "목", "금", "토"];
    let weekday = weekdays[now.weekday().num_days_from_sunday() as usize];

    let clock_time = format!("{:02}:{:02}", now.hour(), now.minute());
    let clock_date = format!(
        "{}년 {}월 {}일 {}요일",
        now.year(),
        now.month(),
        now.day(),
        weekday
    );

    // Claude 세션 카운트 (프로세스 기반)
    let claude_count = performance::count_processes("claude");

    // 누끼 큐 — F:\nokki\queue\ 또는 비슷한 폴더 모니터링
    // 폴더 없으면 0
    let nokki_pending = performance::count_files_in("F:\\nokki\\queue\\pending");
    let nokki_done = performance::count_files_in("F:\\nokki\\queue\\done");
    let nokki_errors = performance::count_files_in("F:\\nokki\\queue\\errors");

    let mut sessions = Vec::new();
    if claude_count > 0 {
        for i in 1..=claude_count.min(8) {
            sessions.push((
                format!("Claude 세션 {}", i),
                "활성".to_string(),
                "green".to_string(),
            ));
        }
    }

    WidgetData {
        clock_time,
        clock_date,
        sales_today: "₩2,340,000".to_string(), // 추후 ResellOn DB 연동
        sales_week: "₩18,420,000".to_string(),
        nokki_pending,
        nokki_done,
        nokki_errors,
        claude_count,
        claude_sessions: sessions,
        weather_temp: "19°".to_string(),
        weather_desc: "구름 조금".to_string(),
    }
}

#[tauri::command]
fn run_tuner_action(action: String) -> Result<String, String> {
    match action.as_str() {
        "clean_temp" => tuner::clean_temp_files(),
        "kill_zombies" => tuner::kill_zombie_processes(),
        "auto_optimize" => tuner::auto_optimize(),
        "clear_recycle" => tuner::clear_recycle_bin(),
        _ => Err(format!("알 수 없는 액션: {}", action)),
    }
}

#[tauri::command]
fn change_mode(app: tauri::AppHandle, mode: String) -> Result<(), String> {
    let mut cfg = config::load(&app);
    cfg.current_mode = mode.clone();

    // 모드별 효과 적용
    match mode.as_str() {
        "ai_work" => {
            // AI 작업 모드: 배경 끄고 위젯 최소화
            if let Some(w) = app.get_webview_window("wallpaper") { let _ = w.hide(); }
        }
        "nokki" => {
            // 누끼 모드: 배경 끄기, 누끼 위젯 강조
            if let Some(w) = app.get_webview_window("wallpaper") { let _ = w.hide(); }
        }
        "focus" => {
            // 포커스 모드: 알림 끄기
        }
        "night" => {
            // 야간 모드: 다크 + 화면 어둡게
        }
        _ => {}
    }

    config::save(&app, &cfg);
    let _ = app.emit("mode-changed", &mode);
    Ok(())
}

#[tauri::command]
fn apply_theme(app: tauri::AppHandle, theme: String) -> Result<(), String> {
    let mut cfg = config::load(&app);
    cfg.theme = theme.clone();
    config::save(&app, &cfg);
    // 모든 창에 테마 변경 알림
    let _ = app.emit("theme-changed", &theme);
    Ok(())
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
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

// Primary 모니터 가져오기 (멀티 모니터 환경에서 안전)
fn primary_monitor(w: &tauri::WebviewWindow) -> Option<tauri::Monitor> {
    // Tauri 2 API: primary_monitor 시도, 없으면 첫 모니터
    w.primary_monitor().ok().flatten()
        .or_else(|| w.available_monitors().ok().and_then(|v| v.into_iter().next()))
}

fn position_dock(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("dock") {
        if let Some(m) = primary_monitor(&w) {
            let pos = m.position();
            let size = m.size();
            let dock_size = w.outer_size().unwrap_or_default();
            // Primary 모니터 기준 좌표 (다른 모니터 안 가도록)
            let x = pos.x + (size.width as i32 - dock_size.width as i32) / 2;
            // 작업표시줄(48px) 위에 8px 띄우기
            let y = pos.y + size.height as i32 - dock_size.height as i32 - 56;
            let _ = w.set_position(tauri::PhysicalPosition { x, y });
        }
    }
}

fn position_widgets(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("widgets") {
        if let Some(m) = primary_monitor(&w) {
            let pos = m.position();
            let size = m.size();
            let widget_size = w.outer_size().unwrap_or_default();
            // 우측 안쪽으로 40px 여유 두기 (잘림 방지)
            let x = pos.x + size.width as i32 - widget_size.width as i32 - 40;
            let y = pos.y + 16;
            let _ = w.set_position(tauri::PhysicalPosition { x, y });
        }
    }
}
