#![windows_subsystem = "windows"]

mod app;
mod audio;
mod config;
mod control_panel;
mod font_scanner;
mod overlay;
#[cfg(target_os = "macos")]
mod platform_macos;
#[cfg(target_os = "windows")]
mod platform_windows;
mod style;
mod timer;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("프레젠테이션 타이머")
            .with_transparent(true)
            .with_inner_size(egui::vec2(500.0, 750.0))
            .with_min_inner_size(egui::vec2(400.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "프레젠테이션 타이머",
        native_options,
        Box::new(|cc| Ok(Box::new(app::TimerApp::new(cc)))),
    )
}
