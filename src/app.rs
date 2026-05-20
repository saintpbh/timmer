use crate::config::AppConfig;
use crate::control_panel;
use crate::overlay;
#[cfg(target_os = "macos")]
use crate::platform_macos;
#[cfg(target_os = "windows")]
use crate::platform_windows;
use crate::style::TimerStyle;
use crate::timer::TimerState;

use egui::{ViewportBuilder, ViewportId};
use std::sync::{Arc, Mutex};

/// 오버레이 뷰포트 ID
const OVERLAY_VIEWPORT: &str = "timer_overlay";

/// 공유 상태 (컨트롤 패널 ↔ 오버레이 간)
pub struct SharedState {
    pub timer: TimerState,
    pub style: TimerStyle,
    pub config: AppConfig,
    pub available_fonts: Vec<(String, String)>,
}

pub struct TimerApp {
    pub shared: Arc<Mutex<SharedState>>,
    /// 오디오 플레이어 (초기화 성공 시)
    pub audio_player: Option<crate::audio::AudioPlayer>,
    /// 현재 로드된 폰트 정의 (동적 로드를 위함)
    pub font_defs: egui::FontDefinitions,
    /// 커스텀 시간 입력 필드
    pub custom_minutes: String,
    pub custom_seconds: String,
}

impl TimerApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        // 한글 지원 폰트 로드
        let font_defs = setup_fonts(cc.egui_ctx.clone());

        // 다크 테마 적용
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        // 설정 로드
        let config = AppConfig::load();
        let style = config.style.clone();
        let mut timer = TimerState::new(config.last_time_secs);
        timer.thresholds = config.thresholds.clone();

        let mut app = Self {
            shared: Arc::new(Mutex::new(SharedState {
                timer,
                style,
                config,
                available_fonts: crate::font_scanner::scan_system_fonts(),
            })),
            audio_player: {
                let ap = crate::audio::AudioPlayer::new();
                if ap.is_some() {
                    log::info!("오디오 플레이어 초기화 성공");
                } else {
                    log::error!("오디오 플레이어 초기화 실패!");
                }
                ap
            },
            font_defs,
            custom_minutes: String::new(),
            custom_seconds: String::new(),
        };
        app.shared.lock().unwrap().timer.start();
        app
    }
}

/// 시스템 폰트를 로드하여 한글 지원 및 폰트 선택 기능 제공
fn setup_fonts(ctx: egui::Context) -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    // 로드할 폰트 목록 (이름, 경로) - 크로스 플랫폼 지원
    let font_options = if cfg!(target_os = "windows") {
        let sys_root = std::env::var("SystemRoot")
            .or_else(|_| std::env::var("windir"))
            .unwrap_or_else(|_| "C:\\Windows".to_string());
        vec![
            ("맑은 고딕".to_string(), format!("{}\\Fonts\\malgun.ttf", sys_root)),
            ("굴림".to_string(), format!("{}\\Fonts\\gulim.ttc", sys_root)),
            ("돋움".to_string(), format!("{}\\Fonts\\dotum.ttc", sys_root)),
            ("바탕".to_string(), format!("{}\\Fonts\\batang.ttc", sys_root)),
        ]
    } else {
        vec![
            ("DIN Alternate Bold".to_string(), "/System/Library/Fonts/Supplemental/DIN Alternate Bold.ttf".to_string()),
            ("애플 고딕".to_string(), "/System/Library/Fonts/AppleSDGothicNeo.ttc".to_string()),
            ("애플 명조".to_string(), "/System/Library/Fonts/Supplemental/AppleMyungjo.ttf".to_string()),
        ]
    };

    let mut korean_font_name: Option<String> = None;

    for (name, path) in font_options {
        if let Ok(font_data) = std::fs::read(&path) {
            let font_name = name.clone();
            fonts.font_data.insert(
                font_name.clone(),
                std::sync::Arc::new(egui::FontData::from_owned(font_data)),
            );

            // 해당 이름의 폰트 패밀리 생성 (나중에 한글 폴백 추가)
            fonts.families.insert(
                egui::FontFamily::Name(font_name.clone().into()),
                vec![font_name.clone()],
            );

            // 첫 번째 한글 폰트를 기억 (DIN 등 비-한글 폰트는 건너뜀)
            let is_korean_font = name == "애플 고딕" || name == "애플 명조"
                || name == "맑은 고딕" || name == "굴림" || name == "돋움" || name == "바탕";
            if korean_font_name.is_none() && is_korean_font {
                korean_font_name = Some(font_name.clone());
            }

            log::info!("폰트 로드: {} ({})", name, path);
        }
    }

    // 한글 폰트를 모든 패밀리의 폴백으로 추가
    if let Some(ref kr_font) = korean_font_name {
        // Proportional, Monospace 기본 패밀리에 한글 폴백 추가
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.push(kr_font.clone());
        }
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.push(kr_font.clone());
        }

        // DIN 등 비-한글 폰트 패밀리에도 한글 폴백 추가
        let family_keys: Vec<_> = fonts.families.keys().cloned().collect();
        for key in family_keys {
            if let egui::FontFamily::Name(ref name) = key {
                let name_str = name.to_string();
                if name_str != *kr_font {
                    if let Some(family) = fonts.families.get_mut(&key) {
                        if !family.contains(kr_font) {
                            family.push(kr_font.clone());
                        }
                    }
                }
            }
        }
        log::info!("한글 폴백 폰트 설정: {}", kr_font);
    } else {
        log::warn!("한글 시스템 폰트를 찾을 수 없습니다");
    }

    ctx.set_fonts(fonts.clone());
    fonts
}


impl eframe::App for TimerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // 완전 투명 — 오버레이 뷰포트가 실제로 투명해지기 위해 필수
        // 컨트롤 패널은 ui()에서 자체 불투명 배경을 그림
        [0.0, 0.0, 0.0, 0.0]
    }

    /// 논리 처리: 타이머 tick + 오버레이 뷰포트 관리
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 동적 폰트 로딩 체크
        {
            let shared = self.shared.lock().unwrap();
            let target_font = shared.style.font_family.clone();
            let font_path = shared.available_fonts.iter()
                .find(|(n, _)| *n == target_font)
                .map(|(_, p)| p.clone());
            drop(shared); // Mutex 해제

            if target_font != "기본 (고정폭)" && !self.font_defs.font_data.contains_key(&target_font) {
                if let Some(path) = font_path {
                    if let Ok(data) = std::fs::read(&path) {
                        // .ttc 등 일부 폰트 파일은 egui가 파싱하지 못해 panic 발생 가능
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let mut test_defs = self.font_defs.clone();
                            test_defs.font_data.insert(
                                target_font.clone(),
                                std::sync::Arc::new(egui::FontData::from_owned(data.clone())),
                            );
                            test_defs.families.insert(
                                egui::FontFamily::Name(target_font.clone().into()),
                                vec![target_font.clone()],
                            );
                            // Proportional/Monospace 폴백에도 추가
                            if let Some(family) = test_defs.families.get_mut(&egui::FontFamily::Proportional) {
                                if !family.contains(&target_font) {
                                    family.push(target_font.clone());
                                }
                            }
                            if let Some(family) = test_defs.families.get_mut(&egui::FontFamily::Monospace) {
                                if !family.contains(&target_font) {
                                    family.push(target_font.clone());
                                }
                            }
                            test_defs
                        }));

                        match result {
                            Ok(new_defs) => {
                                self.font_defs = new_defs;
                                ctx.set_fonts(self.font_defs.clone());
                                log::info!("동적 폰트 로드 성공: {}", target_font);
                            }
                            Err(_) => {
                                log::warn!("폰트 로드 실패 (호환되지 않는 형식): {}", target_font);
                                // 실패 시 기본 폰트로 되돌림
                                let mut shared = self.shared.lock().unwrap();
                                shared.style.font_family = "기본 (고정폭)".to_string();
                            }
                        }
                    }
                }
            }
        }

        // 타이머 tick 및 키보드 입력 처리
        {
            let mut shared = self.shared.lock().unwrap();
            
            let old_remaining = shared.timer.remaining_secs();
            shared.timer.tick();
            let new_remaining = shared.timer.remaining_secs();

            // 오디오 알람 체크: 남은 시간이 설정값 이하가 되면 1회만 재생
            if shared.style.sound_enabled && !shared.timer.alarm_fired {
                let alert_time = shared.style.sound_alert_secs as f64;
                // 타이머가 동작 중이었고(old > new), 임계값을 지나갔을 때
                if old_remaining > new_remaining && new_remaining <= alert_time {
                    shared.timer.alarm_fired = true;
                    log::info!(
                        "알람 트리거! old={:.2}, new={:.2}, alert_time={}",
                        old_remaining,
                        new_remaining,
                        alert_time
                    );
                    if let Some(player) = &self.audio_player {
                        player.play_alarm(shared.style.sound_volume);
                    }
                }
            }

            // ESC 키 입력 시 타이머 리셋 (오버레이 숨김)
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                shared.timer.reset();
            }
        }

        // 오버레이 뷰포트: 타이머가 활성화된 경우에만 생성
        let should_show_overlay = {
            let shared = self.shared.lock().unwrap();
            shared.timer.overlay_active
        };

        if should_show_overlay {
            let shared_clone = Arc::clone(&self.shared);
            let viewport_id = ViewportId::from_hash_of(OVERLAY_VIEWPORT);

            let target_monitor = {
                let shared = self.shared.lock().unwrap();
                shared.config.target_monitor
            };

            ctx.show_viewport_deferred(
                viewport_id,
                ViewportBuilder::default()
                    .with_title("Timer Overlay")
                    .with_transparent(true)
                    .with_decorations(false)
                    .with_always_on_top()
                    .with_taskbar(false)
                    .with_mouse_passthrough(true),
                move |ctx, _class| {
                    // 오버레이 뷰포트의 GPU clear color를 투명으로 설정
                    // (deferred viewport는 App::clear_color()를 사용하지 않고
                    //  Visuals::panel_fill을 clear color로 사용함)
                    {
                        let mut visuals = ctx.style().visuals.clone();
                        visuals.panel_fill = egui::Color32::TRANSPARENT;
                        visuals.window_fill = egui::Color32::TRANSPARENT;
                        ctx.set_visuals(visuals);
                    }

                    let shared = shared_clone.lock().unwrap();

                    // macOS 및 Windows 네이티브 오버레이 설정 (모니터 위치/크기 자동 지정)
                    #[cfg(target_os = "macos")]
                    {
                        platform_macos::configure_overlay_window(ctx, viewport_id, target_monitor);
                    }
                    #[cfg(target_os = "windows")]
                    {
                        platform_windows::configure_overlay_window(ctx, viewport_id, target_monitor);
                    }

                    overlay::render_overlay(ctx, &shared.timer, &shared.style);

                    // 타이머 실행 중이면 지속 repaint
                    if shared.timer.is_running
                        || shared.timer.warning_level() != crate::timer::WarningLevel::None
                    {
                        ctx.request_repaint();
                    }
                },
            );
        }

        // 메인 윈도우도 타이머 실행 중이면 repaint
        let shared = self.shared.lock().unwrap();
        if shared.timer.is_running
            || shared.timer.warning_level() != crate::timer::WarningLevel::None
        {
            ctx.request_repaint();
        }
    }

    /// UI 렌더링: 컨트롤 패널
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // 컨트롤 패널은 불투명 어두운 배경 (clear_color가 투명이므로 직접 그려야 함)
        let panel_frame = egui::Frame::NONE
            .fill(egui::Color32::from_rgb(25, 25, 30))
            .inner_margin(egui::Margin::same(8));

        panel_frame.show(ui, |ui| {
            let mut shared = self.shared.lock().unwrap();
            let SharedState {
                ref mut timer,
                ref mut style,
                ref mut config,
                ref available_fonts,
            } = *shared;
            control_panel::render_control_panel(
                ui,
                timer,
                style,
                config,
                &mut self.custom_minutes,
                &mut self.custom_seconds,
                available_fonts,
            );
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // 종료 시 설정 자동 저장
        let shared = self.shared.lock().unwrap();
        let mut config = shared.config.clone();
        config.style = shared.style.clone();
        config.thresholds = shared.timer.thresholds.clone();
        config.last_time_secs = shared.timer.total_secs;
        config.save();
    }
}
