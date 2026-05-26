// MyDesk 설정 — JSON 파일로 저장
// 위치: %APPDATA%\MyDesk\config.json

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub dock_enabled: bool,
    pub widgets_enabled: bool,
    pub wallpaper_enabled: bool,
    pub rounded_corners_enabled: bool,
    pub rounded_corners_radius: u32,

    pub autostart: bool,
    pub pause_on_ai_work: bool,
    pub gpu_threshold: u32,

    pub wallpaper_path: Option<String>,
    pub wallpaper_brightness: u32,
    pub wallpaper_speed: u32,

    pub dock_position: String,    // "bottom" / "left" / "right"
    pub dock_size: u32,
    pub dock_autohide: bool,
    pub dock_magnification: bool,

    pub widget_position: String,  // "top-right" / "top-left" / "bottom-right" / "bottom-left"
    pub widget_opacity: u32,
    pub widget_update_interval: String,  // "realtime" / "30s" / "1m" / "5m"

    pub accent_color: String,  // hex 코드
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            dock_enabled: true,
            widgets_enabled: true,
            wallpaper_enabled: true,
            rounded_corners_enabled: false,
            rounded_corners_radius: 10,
            autostart: true,
            pause_on_ai_work: true,
            gpu_threshold: 70,
            wallpaper_path: None,
            wallpaper_brightness: 80,
            wallpaper_speed: 100,
            dock_position: "bottom".to_string(),
            dock_size: 56,
            dock_autohide: false,
            dock_magnification: true,
            widget_position: "top-right".to_string(),
            widget_opacity: 85,
            widget_update_interval: "30s".to_string(),
            accent_color: "#8b5cf6".to_string(),
        }
    }
}

fn config_path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app.path().app_config_dir().expect("config dir 없음");
    std::fs::create_dir_all(&dir).ok();
    dir.join("config.json")
}

pub fn load(app: &tauri::AppHandle) -> AppConfig {
    let path = config_path(app);
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<AppConfig>(&text) {
            return cfg;
        }
    }
    AppConfig::default()
}

pub fn save(app: &tauri::AppHandle, cfg: &AppConfig) {
    let path = config_path(app);
    if let Ok(text) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(&path, text);
    }
}
