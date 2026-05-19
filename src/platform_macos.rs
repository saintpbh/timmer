/// macOS 전용 — NSWindow를 풀스크린 프레젠테이션 위에 표시하도록 설정
#[cfg(target_os = "macos")]
pub fn configure_overlay_window(
    ctx: &egui::Context,
    viewport_id: egui::ViewportId,
    target_monitor: usize,
) {
    // ViewportCommand를 통해 항상 최상위 설정 (egui 레벨)
    ctx.send_viewport_cmd_to(
        viewport_id,
        egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop),
    );

    // 네이티브 레벨 설정: NSApplication에서 해당 윈도우를 찾아 풀스크린 앱 위에 오도록 설정
    use objc2_app_kit::{NSApplication, NSColor, NSScreen, NSWindowCollectionBehavior};
    use objc2_foundation::MainThreadMarker;

    if let Some(mtm) = MainThreadMarker::new() {
        let app = NSApplication::sharedApplication(mtm);
        let screens = NSScreen::screens(mtm);
        let target_screen = screens.iter().nth(target_monitor).or_else(|| screens.iter().next());

        for window in app.windows() {
            let title = window.title().to_string();
            // "Timer Overlay" 창을 찾아 설정
            if title.contains("Timer Overlay") {
                // 1. 윈도우 레벨을 최고 레벨로 (NSScreenSaverWindowLevel + 1)
                window.setLevel(1001);

                // 2. 모든 공간(Space)에 나타나도록 설정 (풀스크린 PPT 포함)
                let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::FullScreenAuxiliary
                    | NSWindowCollectionBehavior::Stationary;
                window.setCollectionBehavior(behavior);

                // 3. 마우스 통과 및 투명 배경
                window.setIgnoresMouseEvents(true);
                window.setOpaque(false);
                let clear = NSColor::clearColor();
                window.setBackgroundColor(Some(&clear));
                window.setHasShadow(false);

                // 4. 모니터 크기에 맞게 네이티브로 위치/크기 설정 (winit 좌표계 문제 우회)
                if let Some(ref screen) = target_screen {
                    window.setFrame_display(screen.frame(), true);
                }
            }
        }
    }
}

/// 사용 가능한 모니터 목록 반환 (이름, 크기)
#[cfg(target_os = "macos")]
pub fn get_monitors() -> Vec<MonitorInfo> {
    use objc2_app_kit::NSScreen;
    use objc2_foundation::MainThreadMarker;

    // GUI 앱에서는 update/logic이 항상 메인 스레드에서 호출되므로 안전
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let screens = NSScreen::screens(mtm);
    let mut monitors = Vec::new();

    for (i, screen) in screens.iter().enumerate() {
        let frame = screen.frame();
        monitors.push(MonitorInfo {
            index: i,
            name: if i == 0 {
                "메인 모니터".to_string()
            } else {
                format!("모니터 {}", i + 1)
            },
            width: frame.size.width as f32,
            height: frame.size.height as f32,
            x: frame.origin.x as f32,
            y: frame.origin.y as f32,
        });
    }

    monitors
}

#[cfg(not(target_os = "macos"))]
pub fn get_monitors() -> Vec<MonitorInfo> {
    vec![MonitorInfo {
        index: 0,
        name: "기본 모니터".to_string(),
        width: 1920.0,
        height: 1080.0,
        x: 0.0,
        y: 0.0,
    }]
}

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
