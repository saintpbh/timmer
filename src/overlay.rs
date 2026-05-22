use crate::style::TimerStyle;
use crate::timer::{TimerState, WarningLevel};
use egui::{Color32, CornerRadius, FontId, Pos2, Rect, Vec2};

/// 오버레이 뷰포트에 타이머를 렌더링
/// 반투명 어두운 배경 + 중앙 타이머 + 진행률 바
#[allow(deprecated)] // show(ctx) is needed in deferred viewport callback where &mut Ui is unavailable
pub fn render_overlay(ctx: &egui::Context, timer: &TimerState, style: &TimerStyle) {
    // 반투명 어두운 배경
    let bg_color = Color32::from_rgba_unmultiplied(0, 0, 0, (style.bg_opacity * 255.0) as u8);

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(bg_color))
        .show(ctx, |ui| {
            let screen_rect = ui.max_rect();
            let painter = ui.painter();

            let warning_level = timer.warning_level();
            let base_color = style.color_for_level(warning_level);

            // 깜빡임 알파 계산
            let alpha = compute_blink_alpha(ctx, warning_level, style.opacity);
            let display_color = Color32::from_rgba_unmultiplied(
                base_color.r(),
                base_color.g(),
                base_color.b(),
                (alpha * 255.0) as u8,
            );

            let time_text = timer.formatted();

            // 폰트 설정 — 사용자가 선택한 폰트 또는 기본 고정폭
            let family = if style.font_family == "기본 (고정폭)" {
                egui::FontFamily::Monospace
            } else {
                egui::FontFamily::Name(style.font_family.clone().into())
            };
            let font_id = FontId::new(style.font_size, family.clone());

            // 텍스트 크기 측정
            let galley =
                painter.layout_no_wrap(time_text.clone(), font_id.clone(), display_color);
            let text_size = galley.size();

            // 위치 계산
            let (x, y) = style.position.compute_pos(
                screen_rect.width(),
                screen_rect.height(),
                text_size.x,
                text_size.y,
            );
            let text_pos = Pos2::new(screen_rect.left() + x, screen_rect.top() + y);

            // 그림자 렌더링
            if style.shadow_enabled && alpha > 0.1 {
                let shadow_offset = (style.font_size * 0.03).max(2.0);
                let shadow_color =
                    Color32::from_rgba_unmultiplied(0, 0, 0, (alpha * 200.0) as u8);
                let shadow_galley =
                    painter.layout_no_wrap(time_text.clone(), font_id.clone(), shadow_color);
                painter.galley(
                    Pos2::new(text_pos.x + shadow_offset, text_pos.y + shadow_offset),
                    shadow_galley,
                    Color32::TRANSPARENT,
                );
            }

            // 메인 텍스트 렌더링
            painter.galley(text_pos, galley, Color32::TRANSPARENT);

            // 실시간 알림 메시지 렌더링
            if style.alert_enabled && !style.alert_message.is_empty() {
                let alert_font_id = FontId::new(style.alert_font_size, family.clone());
                let display_alert_color = Color32::from_rgba_unmultiplied(
                    style.alert_color.r(),
                    style.alert_color.g(),
                    style.alert_color.b(),
                    (alpha * 255.0) as u8,
                );

                let alert_galley = painter.layout_no_wrap(
                    style.alert_message.clone(),
                    alert_font_id.clone(),
                    display_alert_color,
                );
                let alert_size = alert_galley.size();

                // 위치 계산
                let alert_pos = match style.alert_position {
                    crate::style::AlertPosition::AboveTimer => {
                        let alert_x = text_pos.x + (text_size.x - alert_size.x) / 2.0;
                        let alert_y = text_pos.y - alert_size.y - 12.0;
                        Pos2::new(alert_x, alert_y)
                    }
                    crate::style::AlertPosition::BelowTimer => {
                        let alert_x = text_pos.x + (text_size.x - alert_size.x) / 2.0;
                        let alert_y = text_pos.y + text_size.y + 12.0;
                        Pos2::new(alert_x, alert_y)
                    }
                    crate::style::AlertPosition::ScreenTop => {
                        let alert_x = screen_rect.left() + (screen_rect.width() - alert_size.x) / 2.0;
                        let alert_y = screen_rect.top() + 40.0;
                        Pos2::new(alert_x, alert_y)
                    }
                    crate::style::AlertPosition::ScreenBottom => {
                        let alert_x = screen_rect.left() + (screen_rect.width() - alert_size.x) / 2.0;
                        let alert_y = screen_rect.bottom() - alert_size.y - 40.0;
                        Pos2::new(alert_x, alert_y)
                    }
                };

                // 그림자 렌더링
                if style.shadow_enabled && alpha > 0.1 {
                    let shadow_offset = (style.alert_font_size * 0.03).max(2.0);
                    let shadow_color =
                        Color32::from_rgba_unmultiplied(0, 0, 0, (alpha * 200.0) as u8);
                    let shadow_galley = painter.layout_no_wrap(
                        style.alert_message.clone(),
                        alert_font_id.clone(),
                        shadow_color,
                    );
                    painter.galley(
                        Pos2::new(alert_pos.x + shadow_offset, alert_pos.y + shadow_offset),
                        shadow_galley,
                        Color32::TRANSPARENT,
                    );
                }

                // 메인 알림 메시지 렌더링
                painter.galley(alert_pos, alert_galley, Color32::TRANSPARENT);
            }

            // 진행률 막대 바 렌더링 제거

            // 경고 시 깜빡임을 위해 지속적으로 repaint 요청
            if warning_level != WarningLevel::None || timer.is_running {
                ctx.request_repaint();
            }
        });
}

/// 깜빡임 알파 값 계산 (sin 함수 기반)
fn compute_blink_alpha(ctx: &egui::Context, level: WarningLevel, base_opacity: f32) -> f32 {
    match level.blink_hz() {
        None => base_opacity,
        Some(hz) => {
            let time = ctx.input(|i| i.time);
            // sin 함수로 부드러운 깜빡임 (0.3 ~ 1.0 범위)
            let wave = (time as f32 * hz * std::f32::consts::TAU).sin();
            let min_alpha = match level {
                WarningLevel::Expired => 0.15,
                WarningLevel::Critical => 0.2,
                _ => 0.3,
            };
            let normalized = (wave + 1.0) / 2.0; // 0.0 ~ 1.0
            let alpha = min_alpha + (1.0 - min_alpha) * normalized;
            alpha * base_opacity
        }
    }
}
