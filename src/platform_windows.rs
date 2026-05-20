#![cfg(target_os = "windows")]

use egui::{Context, ViewportId, ViewportCommand};
use windows_sys::Win32::Foundation::{HWND, LPARAM, TRUE, RECT};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos,
    GWL_EXSTYLE, WS_EX_TRANSPARENT, WS_EX_LAYERED, HWND_TOPMOST, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOACTIVATE, SWP_SHOWWINDOW
};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HMONITOR, HDC, MONITORINFOEXW
};

/// 모니터 정보
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
}

// "Timer Overlay" 타이틀을 가진 창의 HWND를 찾기 위한 콜백 구조
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> i32 {
    let mut title = [0u16; 512];
    let len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
    if len > 0 {
        let text = String::from_utf16_lossy(&title[..len as usize]);
        if text.contains("Timer Overlay") {
            let target_ptr = lparam as *mut HWND;
            *target_ptr = hwnd;
            return 0; // 찾았으므로 열거 중단
        }
    }
    1 // 계속 열거
}

/// Windows 전용 — Winit 윈도우를 풀스크린 프레젠테이션 최상위에 항상 고정시키고 투명 마우스 클릭 무시를 강제 지정
pub fn configure_overlay_window(
    ctx: &Context,
    viewport_id: ViewportId,
    target_monitor: usize,
) {
    // 1. egui 레벨 AlwaysOnTop 전송
    ctx.send_viewport_cmd_to(
        viewport_id,
        ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop),
    );

    // 2. Win32 네이티브 핸들 조회
    let mut overlay_hwnd: HWND = 0;
    unsafe {
        EnumWindows(Some(enum_windows_callback), &mut overlay_hwnd as *mut HWND as LPARAM);
    }

    if overlay_hwnd != 0 {
        unsafe {
            // 3. WS_EX_TRANSPARENT 및 WS_EX_LAYERED 지정을 통한 완벽한 마우스 클릭 무시 및 투명 레이어 보장
            let current_exstyle = GetWindowLongPtrW(overlay_hwnd, GWL_EXSTYLE);
            let target_exstyle = current_exstyle | (WS_EX_TRANSPARENT | WS_EX_LAYERED) as isize;
            if current_exstyle != target_exstyle {
                SetWindowLongPtrW(overlay_hwnd, GWL_EXSTYLE, target_exstyle);
            }

            // 4. HWND_TOPMOST 강제 지정 (PPT 전체화면 오버레이 최상단 보장)
            // SWP_NOACTIVATE: 포커스를 뺏지 않아 PPT 조작을 방해하지 않음
            SetWindowPos(
                overlay_hwnd,
                HWND_TOPMOST,
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW
            );

            // 5. 지정 모니터로 오버레이 강제 정렬
            let monitors = get_monitors();
            if let Some(monitor) = monitors.iter().find(|m| m.index == target_monitor).or_else(|| monitors.first()) {
                // 네이티브 창 좌표를 모니터의 Rect로 이동
                SetWindowPos(
                    overlay_hwnd,
                    HWND_TOPMOST,
                    monitor.x as i32,
                    monitor.y as i32,
                    monitor.width as i32,
                    monitor.height as i32,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW
                );
            }
        }
    }
}

// Windows 모니터 열거용 콜백 데이터 구조
struct MonitorEnumData {
    monitors: Vec<MonitorInfo>,
}

unsafe extern "system" fn enum_monitors_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    rect: *mut RECT,
    lparam: LPARAM,
) -> i32 {
    let data = &mut *(lparam as *mut MonitorEnumData);
    let mut info: MONITORINFOEXW = std::mem::zeroed();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    if GetMonitorInfoW(hmonitor, &mut info as *mut MONITORINFOEXW as *mut _) != 0 {
        let r = &*rect;
        let index = data.monitors.len();
        
        // 널 종단 문자 제거하며 장치 이름 파싱
        let name_len = info.szDevice.iter().position(|&c| c == 0).unwrap_or(32);
        let name = String::from_utf16_lossy(&info.szDevice[..name_len]);

        let width = (r.right - r.left).abs() as f32;
        let height = (r.bottom - r.top).abs() as f32;

        data.monitors.push(MonitorInfo {
            index,
            name: if index == 0 {
                format!("메인 모니터 ({})", name)
            } else {
                format!("모니터 {} ({})", index + 1, name)
            },
            width,
            height,
            x: r.left as f32,
            y: r.top as f32,
        });
    }
    TRUE // 계속 열거
}

/// Windows 시스템의 실제 모든 연결 모니터 좌표 및 크기를 수집하여 반환
pub fn get_monitors() -> Vec<MonitorInfo> {
    let mut data = MonitorEnumData { monitors: Vec::new() };
    unsafe {
        EnumDisplayMonitors(
            0,
            std::ptr::null(),
            Some(enum_monitors_callback),
            &mut data as *mut MonitorEnumData as LPARAM,
        );
    }
    if data.monitors.is_empty() {
        // 모니터 열거 실패 시 기본 폴백
        vec![MonitorInfo {
            index: 0,
            name: "기본 모니터".to_string(),
            width: 1920.0,
            height: 1080.0,
            x: 0.0,
            y: 0.0,
        }]
    } else {
        data.monitors
    }
}
