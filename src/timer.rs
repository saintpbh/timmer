use std::time::Instant;

/// 경고 단계별 상태
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WarningLevel {
    /// 정상 — 경고 없음
    None,
    /// 2분 이하 — 노란색, 느린 깜빡임
    Caution,
    /// 1분 이하 — 주황색, 중간 깜빡임
    Warning,
    /// 30초 이하 — 빨간색, 빠른 깜빡임
    Critical,
    /// 시간 초과 — 빨간색 고정
    Expired,
}

impl WarningLevel {
    /// 깜빡임 주파수 (Hz). None이면 깜빡임 없음.
    /// 주의/경고는 색상만 변경, 위험/초과일 때만 깜빡임
    pub fn blink_hz(self) -> Option<f32> {
        match self {
            Self::None => None,
            Self::Caution => None,
            Self::Warning => None,
            Self::Critical => Some(4.0),
            Self::Expired => Some(6.0),
        }
    }
}

/// 경고 임계값 설정
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WarningThresholds {
    /// 주의 (초 단위, 기본 120초 = 2분)
    pub caution_secs: u64,
    /// 경고 (초 단위, 기본 60초 = 1분)
    pub warning_secs: u64,
    /// 위험 (초 단위, 기본 30초)
    pub critical_secs: u64,
    /// 각 임계값 활성화 여부
    pub caution_enabled: bool,
    pub warning_enabled: bool,
    pub critical_enabled: bool,
}

impl Default for WarningThresholds {
    fn default() -> Self {
        Self {
            caution_secs: 120,
            warning_secs: 60,
            critical_secs: 30,
            caution_enabled: true,
            warning_enabled: true,
            critical_enabled: true,
        }
    }
}

/// 타이머 핵심 상태
pub struct TimerState {
    /// 설정된 총 시간 (초)
    pub total_secs: u64,
    /// 남은 시간 (초, 소수점 포함)
    remaining: f64,
    /// 실행 중 여부
    pub is_running: bool,
    /// 오버레이 표시 여부 (시작 → true, 리셋 → false)
    pub overlay_active: bool,
    /// 마지막 tick 시각 (frame-rate 독립적 타이밍)
    last_tick: Option<Instant>,
    /// 경고 임계값
    pub thresholds: WarningThresholds,
    /// 이번 타이머 세션에서 알람이 이미 울렸는지 여부
    pub alarm_fired: bool,
}

impl TimerState {
    pub fn new(total_secs: u64) -> Self {
        Self {
            total_secs,
            remaining: total_secs as f64,
            is_running: false,
            overlay_active: false,
            last_tick: None,
            thresholds: WarningThresholds::default(),
            alarm_fired: false,
        }
    }

    /// 매 프레임 호출 — Instant 기반으로 frame-rate 독립적 감소
    pub fn tick(&mut self) {
        if !self.is_running {
            return;
        }

        let now = Instant::now();
        if let Some(last) = self.last_tick {
            let elapsed = now.duration_since(last).as_secs_f64();
            self.remaining = (self.remaining - elapsed).max(0.0);

            // 시간 초과 시 자동 정지 (00:00 유지)
            if self.remaining <= 0.0 {
                self.remaining = 0.0;
            }
        }
        self.last_tick = Some(now);
    }

    pub fn start(&mut self) {
        self.is_running = true;
        self.overlay_active = true;
        self.last_tick = Some(Instant::now());
        self.alarm_fired = false;
    }

    pub fn pause(&mut self) {
        self.is_running = false;
        self.last_tick = None;
    }

    pub fn reset(&mut self) {
        self.remaining = self.total_secs as f64;
        self.is_running = false;
        self.overlay_active = false;
        self.last_tick = None;
        self.alarm_fired = false;
    }

    pub fn set_time(&mut self, secs: u64) {
        self.total_secs = secs;
        self.remaining = secs as f64;
        self.is_running = false;
        self.last_tick = None;
        self.alarm_fired = false;
    }

    /// 남은 시간 (초, 소수점 포함)
    pub fn remaining_secs(&self) -> f64 {
        self.remaining
    }

    /// 남은 시간의 분 부분
    pub fn minutes(&self) -> u64 {
        (self.remaining.ceil() as u64) / 60
    }

    /// 남은 시간의 초 부분
    pub fn seconds(&self) -> u64 {
        (self.remaining.ceil() as u64) % 60
    }

    /// "MM:SS" 형식 문자열
    pub fn formatted(&self) -> String {
        format!("{:02}:{:02}", self.minutes(), self.seconds())
    }

    /// 현재 경고 레벨
    pub fn warning_level(&self) -> WarningLevel {
        let secs = self.remaining.ceil() as u64;

        if secs == 0 && self.total_secs > 0 {
            return WarningLevel::Expired;
        }

        if self.thresholds.critical_enabled && secs <= self.thresholds.critical_secs && secs > 0 {
            return WarningLevel::Critical;
        }

        if self.thresholds.warning_enabled && secs <= self.thresholds.warning_secs {
            return WarningLevel::Warning;
        }

        if self.thresholds.caution_enabled && secs <= self.thresholds.caution_secs {
            return WarningLevel::Caution;
        }

        WarningLevel::None
    }
}
