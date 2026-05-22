use egui::Color32;

/// 타이머 화면 위치 프리셋
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TimerPosition {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    Center,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl TimerPosition {
    pub const ALL: [Self; 9] = [
        Self::TopLeft,
        Self::TopCenter,
        Self::TopRight,
        Self::MiddleLeft,
        Self::Center,
        Self::MiddleRight,
        Self::BottomLeft,
        Self::BottomCenter,
        Self::BottomRight,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::TopLeft => "↖ 좌상단",
            Self::TopCenter => "↑ 상단",
            Self::TopRight => "↗ 우상단",
            Self::MiddleLeft => "← 좌측",
            Self::Center => "● 중앙",
            Self::MiddleRight => "→ 우측",
            Self::BottomLeft => "↙ 좌하단",
            Self::BottomCenter => "↓ 하단",
            Self::BottomRight => "↘ 우하단",
        }
    }

    /// 주어진 화면 크기와 텍스트 크기에 대해 좌표 계산
    pub fn compute_pos(
        self,
        screen_w: f32,
        screen_h: f32,
        text_w: f32,
        text_h: f32,
    ) -> (f32, f32) {
        let margin = 40.0;
        let x = match self {
            Self::TopLeft | Self::MiddleLeft | Self::BottomLeft => margin,
            Self::TopCenter | Self::Center | Self::BottomCenter => (screen_w - text_w) / 2.0,
            Self::TopRight | Self::MiddleRight | Self::BottomRight => screen_w - text_w - margin,
        };
        let y = match self {
            Self::TopLeft | Self::TopCenter | Self::TopRight => margin,
            Self::MiddleLeft | Self::Center | Self::MiddleRight => (screen_h - text_h) / 2.0,
            Self::BottomLeft | Self::BottomCenter | Self::BottomRight => screen_h - text_h - margin,
        };
        (x, y)
    }
}

/// 실시간 알림 메시지 위치 프리셋
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AlertPosition {
    AboveTimer,
    BelowTimer,
    ScreenTop,
    ScreenBottom,
}

impl AlertPosition {
    pub const ALL: [Self; 4] = [
        Self::AboveTimer,
        Self::BelowTimer,
        Self::ScreenTop,
        Self::ScreenBottom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::AboveTimer => "⏱ 타이머 바로 위",
            Self::BelowTimer => "⏱ 타이머 바로 아래",
            Self::ScreenTop => "🖥 화면 최상단",
            Self::ScreenBottom => "🖥 화면 최하단",
        }
    }
}

impl Default for AlertPosition {
    fn default() -> Self {
        Self::BelowTimer
    }
}


/// 색상을 [u8; 4] (RGBA)로 직렬화하기 위한 헬퍼
mod color_serde {
    use egui::Color32;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct Rgba(u8, u8, u8, u8);

    pub fn serialize<S: Serializer>(color: &Color32, serializer: S) -> Result<S::Ok, S::Error> {
        Rgba(color.r(), color.g(), color.b(), color.a()).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Color32, D::Error> {
        let Rgba(r, g, b, a) = Rgba::deserialize(deserializer)?;
        Ok(Color32::from_rgba_premultiplied(r, g, b, a))
    }
}

fn default_font_family() -> String {
    "DIN Alternate Bold".to_string()
}

/// 타이머 표시 스타일 설정
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimerStyle {
    /// 선택된 폰트 이름
    #[serde(default = "default_font_family")]
    pub font_family: String,
    /// 폰트 크기 (px)
    pub font_size: f32,

    /// 기본 텍스트 색상
    #[serde(with = "color_serde")]
    pub normal_color: Color32,

    /// 주의 색상 (2분 이하)
    #[serde(with = "color_serde")]
    pub caution_color: Color32,

    /// 경고 색상 (1분 이하)
    #[serde(with = "color_serde")]
    pub warning_color: Color32,

    /// 위험 색상 (30초 이하)
    #[serde(with = "color_serde")]
    pub critical_color: Color32,

    /// 텍스트 불투명도 (0.0 ~ 1.0)
    pub opacity: f32,

    /// 배경 불투명도 (0.0 ~ 1.0)
    pub bg_opacity: f32,

    /// 화면 위치
    pub position: TimerPosition,

    /// 텍스트 그림자 활성화
    pub shadow_enabled: bool,

    /// 볼드체 사용
    pub bold: bool,

    /// 소리 알림 켜기/끄기
    #[serde(default = "default_true")]
    pub sound_enabled: bool,

    /// 소리 알림이 울릴 남은 시간(초) (기본 0초)
    #[serde(default)]
    pub sound_alert_secs: u64,

    /// 알림 볼륨 (0.0 ~ 1.0)
    #[serde(default = "default_volume")]
    pub sound_volume: f32,

    /// 실시간 알림 메시지 켜기/끄기
    #[serde(default)]
    pub alert_enabled: bool,

    /// 실시간 알림 메시지 텍스트
    #[serde(default)]
    pub alert_message: String,

    /// 알림 메시지 위치 설정
    #[serde(default)]
    pub alert_position: AlertPosition,

    /// 알림 메시지 폰트 크기
    #[serde(default = "default_alert_font_size")]
    pub alert_font_size: f32,

    /// 알림 메시지 색상
    #[serde(default = "default_alert_color", with = "color_serde")]
    pub alert_color: Color32,
}

fn default_true() -> bool {
    true
}

fn default_volume() -> f32 {
    1.0
}

fn default_alert_font_size() -> f32 {
    36.0
}

fn default_alert_color() -> Color32 {
    Color32::from_rgb(255, 235, 59)
}

impl Default for TimerStyle {
    fn default() -> Self {
        Self {
            font_family: default_font_family(),
            font_size: 160.0,
            normal_color: Color32::WHITE,
            caution_color: Color32::from_rgb(255, 214, 0),  // 밝은 노랑
            warning_color: Color32::from_rgb(255, 152, 0),  // 주황
            critical_color: Color32::from_rgb(244, 67, 54), // 빨강
            opacity: 0.95,
            bg_opacity: 0.55,
            position: TimerPosition::Center, // 기본: 화면 중앙
            shadow_enabled: true,
            bold: true,
            sound_enabled: true,
            sound_alert_secs: 0,
            sound_volume: 1.0,
            alert_enabled: false,
            alert_message: String::new(),
            alert_position: AlertPosition::default(),
            alert_font_size: default_alert_font_size(),
            alert_color: default_alert_color(),
        }
    }
}


impl TimerStyle {
    /// 경고 레벨에 따른 색상 반환
    pub fn color_for_level(&self, level: crate::timer::WarningLevel) -> Color32 {
        match level {
            crate::timer::WarningLevel::None => self.normal_color,
            crate::timer::WarningLevel::Caution => self.caution_color,
            crate::timer::WarningLevel::Warning => self.warning_color,
            crate::timer::WarningLevel::Critical | crate::timer::WarningLevel::Expired => {
                self.critical_color
            }
        }
    }
}
