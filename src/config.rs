use crate::style::TimerStyle;
use crate::timer::WarningThresholds;
use std::path::PathBuf;

/// 앱 전체 설정 (저장/로드 대상)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    /// 마지막 사용한 시간 (초)
    pub last_time_secs: u64,
    /// 타이머 스타일
    pub style: TimerStyle,
    /// 경고 임계값
    pub thresholds: WarningThresholds,
    /// 오버레이를 표시할 모니터 인덱스 (0 = 기본)
    pub target_monitor: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            last_time_secs: 300, // 5분
            style: TimerStyle::default(),
            thresholds: WarningThresholds::default(),
            target_monitor: 1, // 세컨드 모니터
        }
    }
}

impl AppConfig {
    /// 설정 파일 경로
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("PresentationTimer").join("config.json"))
    }

    /// 파일에서 설정 로드. 실패 시 기본값 반환
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| std::fs::read_to_string(&path).ok())
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    /// 설정을 파일에 저장
    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(&path, json);
            }
        }
    }
}
