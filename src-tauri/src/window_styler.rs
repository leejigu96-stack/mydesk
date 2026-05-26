// 윈도우 창 스타일링 - 모서리 둥글게, z-order 등
// Windows DWM API 사용

#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE,
    DWMWCP_DEFAULT, DWMWCP_DONOTROUND, DWMWCP_ROUND, DWMWCP_ROUNDSMALL,
    DWM_WINDOW_CORNER_PREFERENCE,
};

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;

// 모든 창 모서리 둥글게 (Windows 11만 지원, Windows 10에선 무시됨)
pub fn apply_corners(enable: bool, _radius: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // TODO: EnumWindows로 모든 창 열거 후 DwmSetWindowAttribute 호출
        // 지금은 자체 창에만 적용
        let pref = if enable {
            DWMWCP_ROUND
        } else {
            DWMWCP_DEFAULT
        };
        // 자체 창은 Tauri 윈도우 객체로 직접 적용해야 함
        let _ = pref;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Windows 전용 기능".to_string())
    }
}

// 창을 z-order 맨 뒤로 (배경화면 창용)
#[cfg(target_os = "windows")]
pub fn send_to_back(window: &tauri::WebviewWindow) -> Result<(), String> {
    use windows::Win32::Foundation::HWND;
    let raw = window.hwnd().map_err(|e| e.to_string())?;
    // Tauri의 hwnd()는 raw HWND (*mut c_void) — windows 0.56 HWND(isize) 로 변환
    let hwnd = HWND(raw.0 as isize);
    unsafe {
        SetWindowPos(
            hwnd,
            HWND_BOTTOM,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn send_to_back(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}
