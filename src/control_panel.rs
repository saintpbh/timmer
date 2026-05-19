use crate::config::AppConfig;
use crate::style::{TimerPosition, TimerStyle};
use crate::timer::{TimerState, WarningLevel};
use egui::{Color32, RichText, Vec2};

/// 시간 프리셋 (라벨, 초)
const TIME_PRESETS: &[(&str, u64)] = &[
    ("1분", 60),
    ("2분", 120),
    ("3분", 180),
];

/// 컨트롤 패널 UI 렌더링 (eframe 0.34 — &mut Ui 직접 수신)
pub fn render_control_panel(
    ui: &mut egui::Ui,
    timer: &mut TimerState,
    style: &mut TimerStyle,
    config: &mut AppConfig,
    custom_minutes: &mut String,
    custom_seconds: &mut String,
    available_fonts: &[(String, String)],
) {
    ui.spacing_mut().item_spacing = Vec2::new(8.0, 10.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // ═══════════════════════════════════════════
        // 타이머 미리보기
        // ═══════════════════════════════════════════
        ui.vertical_centered(|ui| {
            let warning = timer.warning_level();
            let color = style.color_for_level(warning);
            let time_text = timer.formatted();

            // 폰트 설정 — 사용자가 선택한 폰트 또는 기본 고정폭
            let family = if style.font_family == "기본 (고정폭)" {
                egui::FontFamily::Monospace
            } else {
                egui::FontFamily::Name(style.font_family.clone().into())
            };
            ui.add_space(8.0);
            ui.label(
                RichText::new(&time_text)
                    .size(72.0)
                    .color(color)
                    .strong()
                    .family(family),
            );

            // 상태 표시
            let status = if timer.is_running {
                RichText::new("● 실행 중")
                    .color(Color32::from_rgb(76, 175, 80))
                    .size(14.0)
            } else if timer.remaining_secs() <= 0.0 && timer.total_secs > 0 {
                RichText::new("● 시간 종료")
                    .color(Color32::from_rgb(244, 67, 54))
                    .size(14.0)
            } else {
                RichText::new("● 대기")
                    .color(Color32::from_rgb(158, 158, 158))
                    .size(14.0)
            };
            ui.label(status);

            // 미니 진행률 바
            if timer.total_secs > 0 {
                let progress =
                    (timer.remaining_secs() / timer.total_secs as f64).clamp(0.0, 1.0) as f32;
                let bar = egui::ProgressBar::new(progress)
                    .desired_width(250.0)
                    .fill(color);
                ui.add(bar);
            }
            ui.add_space(4.0);
        });

        ui.separator();

        // ═══════════════════════════════════════════
        // 타이머 제어 버튼
        // ═══════════════════════════════════════════
        ui.heading("⏱  타이머 제어");
        ui.add_space(2.0);

        ui.horizontal(|ui| {
            let btn_size = Vec2::new(80.0, 36.0);

            if timer.is_running {
                if ui
                    .add_sized(
                        btn_size,
                        egui::Button::new(RichText::new("⏸ 일시정지").size(14.0)),
                    )
                    .clicked()
                {
                    timer.pause();
                }
            } else {
                let can_start = timer.remaining_secs() > 0.0;
                if ui
                    .add_enabled(
                        can_start,
                        egui::Button::new(RichText::new("▶ 시작").size(14.0)).min_size(btn_size),
                    )
                    .clicked()
                {
                    timer.start();
                }
            }

            if ui
                .add_sized(
                    btn_size,
                    egui::Button::new(RichText::new("⏹ 리셋").size(14.0)),
                )
                .clicked()
            {
                timer.reset();
            }

            if ui
                .add_sized(
                    Vec2::new(55.0, 36.0),
                    egui::Button::new(RichText::new("+1분").size(13.0)),
                )
                .clicked()
            {
                let new_total = timer.total_secs + 60;
                timer.set_time(new_total);
            }

            if ui
                .add_sized(
                    Vec2::new(55.0, 36.0),
                    egui::Button::new(RichText::new("-1분").size(13.0)),
                )
                .clicked()
            {
                let new_total = timer.total_secs.saturating_sub(60);
                if new_total > 0 {
                    timer.set_time(new_total);
                }
            }
        });

        ui.add_space(4.0);
        ui.separator();

        // ═══════════════════════════════════════════
        // 출력 모니터 선택 (상단 배치 — 가장 먼저 설정)
        // ═══════════════════════════════════════════
        ui.heading("🖥  타이머 출력 모니터");
        ui.add_space(2.0);

        let monitors = crate::platform_macos::get_monitors();
        if monitors.is_empty() {
            ui.label("모니터를 찾을 수 없습니다.");
        } else {
            ui.horizontal_wrapped(|ui| {
                for monitor in &monitors {
                    let is_selected = config.target_monitor == monitor.index;
                    let label = format!(
                        "{} ({}x{})",
                        monitor.name, monitor.width as u32, monitor.height as u32
                    );
                    let btn = if is_selected {
                        egui::Button::new(RichText::new(&label).size(13.0).strong())
                            .fill(Color32::from_rgb(0, 137, 123))
                    } else {
                        egui::Button::new(RichText::new(&label).size(13.0))
                    };
                    if ui.add_sized(Vec2::new(200.0, 30.0), btn).clicked() {
                        config.target_monitor = monitor.index;
                    }
                }
            });
            if monitors.len() <= 1 {
                ui.label(
                    RichText::new("💡 세컨드 모니터를 연결하면 별도 화면에 타이머를 표시할 수 있습니다.")
                        .size(11.0)
                        .color(Color32::from_rgb(158, 158, 158)),
                );
            }
        }

        ui.add_space(4.0);
        ui.separator();

        // ═══════════════════════════════════════════
        // 시간 프리셋
        // ═══════════════════════════════════════════
        ui.heading("🕐  시간 설정");
        ui.add_space(2.0);

        ui.horizontal_wrapped(|ui| {
            for (label, secs) in TIME_PRESETS {
                let is_selected = timer.total_secs == *secs;
                let btn = if is_selected {
                    egui::Button::new(RichText::new(*label).size(13.0).strong())
                        .fill(Color32::from_rgb(33, 150, 243))
                } else {
                    egui::Button::new(RichText::new(*label).size(13.0))
                };

                if ui.add_sized(Vec2::new(55.0, 30.0), btn).clicked() {
                    timer.set_time(*secs);
                    config.last_time_secs = *secs;
                }
            }
        });

        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label("커스텀:");
            ui.add(
                egui::TextEdit::singleline(custom_minutes)
                    .desired_width(35.0)
                    .hint_text("분"),
            );
            ui.label("분");
            ui.add(
                egui::TextEdit::singleline(custom_seconds)
                    .desired_width(35.0)
                    .hint_text("초"),
            );
            ui.label("초");

            if ui.button("적용").clicked() {
                let mins: u64 = custom_minutes.parse().unwrap_or(0);
                let secs: u64 = custom_seconds.parse().unwrap_or(0);
                let total = mins * 60 + secs;
                if total > 0 {
                    timer.set_time(total);
                    config.last_time_secs = total;
                }
            }
        });

        ui.add_space(4.0);
        ui.separator();

        // ═══════════════════════════════════════════
        // 경고 임계값 설정
        // ═══════════════════════════════════════════
        ui.heading("⚠  경고 설정");
        ui.add_space(2.0);

        ui.horizontal(|ui| {
            ui.checkbox(&mut timer.thresholds.caution_enabled, "");
            ui.label(RichText::new("주의").color(style.caution_color));
            let mut val = timer.thresholds.caution_secs as f32;
            if ui
                .add(egui::Slider::new(&mut val, 30.0..=600.0).suffix("초"))
                .changed()
            {
                timer.thresholds.caution_secs = val as u64;
            }
        });

        ui.horizontal(|ui| {
            ui.checkbox(&mut timer.thresholds.warning_enabled, "");
            ui.label(RichText::new("경고").color(style.warning_color));
            let mut val = timer.thresholds.warning_secs as f32;
            if ui
                .add(egui::Slider::new(&mut val, 10.0..=300.0).suffix("초"))
                .changed()
            {
                timer.thresholds.warning_secs = val as u64;
            }
        });

        ui.horizontal(|ui| {
            ui.checkbox(&mut timer.thresholds.critical_enabled, "");
            ui.label(RichText::new("위험").color(style.critical_color));
            let mut val = timer.thresholds.critical_secs as f32;
            if ui
                .add(egui::Slider::new(&mut val, 5.0..=120.0).suffix("초"))
                .changed()
            {
                timer.thresholds.critical_secs = val as u64;
            }
        });

        ui.add_space(4.0);
        ui.separator();

        // ═══════════════════════════════════════════
        // 오디오 경고음 설정
        // ═══════════════════════════════════════════
        ui.heading("🔊  오디오 경고음 설정");
        ui.add_space(2.0);

        ui.horizontal(|ui| {
            ui.checkbox(&mut style.sound_enabled, "경고음 재생 켜기");
        });

        if style.sound_enabled {
            ui.horizontal(|ui| {
                ui.label("울리는 시간:");
                ui.add(egui::Slider::new(&mut style.sound_alert_secs, 0..=120).suffix("초 남았을 때"));
            });

            ui.horizontal(|ui| {
                ui.label("알림 볼륨:");
                ui.add(egui::Slider::new(&mut style.sound_volume, 0.0..=1.0).fixed_decimals(2));
            });
        }

        ui.add_space(4.0);
        ui.separator();

        // ═══════════════════════════════════════════
        // 스타일 설정
        // ═══════════════════════════════════════════
        ui.heading("🎨  스타일 설정");
        ui.add_space(2.0);

        ui.horizontal(|ui| {
            ui.label("폰트 선택:");
            egui::ComboBox::from_id_source("font_family_combo")
                .selected_text(&style.font_family)
                .show_ui(ui, |ui| {
                    for (name, _) in available_fonts {
                        ui.selectable_value(&mut style.font_family, name.clone(), name);
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("폰트 크기:");
            ui.add(egui::Slider::new(&mut style.font_size, 40.0..=300.0).suffix("px"));
        });

        ui.horizontal(|ui| {
            ui.label("텍스트 불투명도:");
            ui.add(egui::Slider::new(&mut style.opacity, 0.1..=1.0).fixed_decimals(2));
        });

        ui.horizontal(|ui| {
            ui.label("배경 투명도:");
            ui.add(egui::Slider::new(&mut style.bg_opacity, 0.0..=1.0).fixed_decimals(2));
            // 현재 상태 라벨
            let state_label = if style.bg_opacity <= 0.01 {
                "완전 투명"
            } else if style.bg_opacity < 0.3 {
                "거의 투명"
            } else if style.bg_opacity < 0.7 {
                "반투명"
            } else if style.bg_opacity < 0.99 {
                "거의 불투명"
            } else {
                "완전 불투명"
            };
            ui.label(
                RichText::new(state_label)
                    .size(11.0)
                    .color(Color32::from_rgb(158, 158, 158)),
            );
        });
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            let presets = [
                ("🔍 완전 투명", 0.0_f32),
                ("◐ 반투명", 0.5),
                ("■ 불투명", 1.0),
            ];
            for (label, value) in presets {
                let is_active = (style.bg_opacity - value).abs() < 0.02;
                let btn = if is_active {
                    egui::Button::new(RichText::new(label).size(12.0).strong())
                        .fill(Color32::from_rgb(33, 150, 243))
                } else {
                    egui::Button::new(RichText::new(label).size(12.0))
                };
                if ui.add_sized(Vec2::new(90.0, 26.0), btn).clicked() {
                    style.bg_opacity = value;
                }
            }
        });



        // 색상 설정
        ui.horizontal(|ui| {
            ui.label("기본 색상:");
            let mut c = color32_to_rgb(style.normal_color);
            if ui.color_edit_button_rgb(&mut c).changed() {
                style.normal_color = rgb_to_color32(c);
            }
            ui.add_space(8.0);
            ui.label("주의:");
            let mut c2 = color32_to_rgb(style.caution_color);
            if ui.color_edit_button_rgb(&mut c2).changed() {
                style.caution_color = rgb_to_color32(c2);
            }
        });

        ui.horizontal(|ui| {
            ui.label("경고 색상:");
            let mut c = color32_to_rgb(style.warning_color);
            if ui.color_edit_button_rgb(&mut c).changed() {
                style.warning_color = rgb_to_color32(c);
            }
            ui.add_space(8.0);
            ui.label("위험:");
            let mut c2 = color32_to_rgb(style.critical_color);
            if ui.color_edit_button_rgb(&mut c2).changed() {
                style.critical_color = rgb_to_color32(c2);
            }
        });

        ui.horizontal(|ui| {
            ui.checkbox(&mut style.shadow_enabled, "텍스트 그림자");
        });

        ui.add_space(4.0);
        ui.separator();

        // ═══════════════════════════════════════════
        // 위치 설정 (3x3 그리드)
        // ═══════════════════════════════════════════
        ui.heading("📍  위치 설정");
        ui.add_space(2.0);

        egui::Grid::new("position_grid")
            .spacing(Vec2::new(3.0, 3.0))
            .show(ui, |ui| {
                for (i, pos) in TimerPosition::ALL.iter().enumerate() {
                    let is_selected = style.position == *pos;
                    let btn = if is_selected {
                        egui::Button::new(RichText::new(pos.label()).size(11.0).strong())
                            .fill(Color32::from_rgb(33, 150, 243))
                    } else {
                        egui::Button::new(RichText::new(pos.label()).size(11.0))
                    };

                    if ui.add_sized(Vec2::new(75.0, 26.0), btn).clicked() {
                        style.position = *pos;
                    }

                    if (i + 1) % 3 == 0 {
                        ui.end_row();
                    }
                }
            });

        ui.add_space(4.0);
        ui.separator();



        // ═══════════════════════════════════════════
        // 설정 저장
        // ═══════════════════════════════════════════
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new("💾 설정 저장").size(13.0))
                .clicked()
            {
                config.style = style.clone();
                config.thresholds = timer.thresholds.clone();
                config.save();
            }

            if ui
                .button(RichText::new("🔄 기본값 복원").size(13.0))
                .clicked()
            {
                *style = TimerStyle::default();
                timer.thresholds = crate::timer::WarningThresholds::default();
            }
        });

        ui.add_space(16.0);
    });

    // 타이머 실행 중이면 지속적으로 repaint
    if timer.is_running || timer.warning_level() != WarningLevel::None {
        ui.ctx().request_repaint();
    }
}

fn color32_to_rgb(c: Color32) -> [f32; 3] {
    [
        c.r() as f32 / 255.0,
        c.g() as f32 / 255.0,
        c.b() as f32 / 255.0,
    ]
}

fn rgb_to_color32(c: [f32; 3]) -> Color32 {
    Color32::from_rgb(
        (c[0] * 255.0) as u8,
        (c[1] * 255.0) as u8,
        (c[2] * 255.0) as u8,
    )
}
