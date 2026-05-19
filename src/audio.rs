use std::sync::atomic::{AtomicBool, Ordering};

/// 앱에 내장된 알림 소리 파일
const ALARM_SOUND: &[u8] = include_bytes!("../ding.wav");

/// 재생 중복 방지 플래그
static IS_PLAYING: AtomicBool = AtomicBool::new(false);

/// macOS의 afplay 커맨드를 사용하여 안정적으로 오디오 재생.
/// rodio/cpal이 eframe GUI 앱 내에서 오디오 디바이스를 열지 못하는 문제를 우회.
pub struct AudioPlayer;

impl AudioPlayer {
    pub fn new() -> Option<Self> {
        // afplay는 macOS 기본 내장 명령어이므로 항상 사용 가능
        log::info!("오디오: macOS afplay 기반 플레이어 초기화");
        Some(Self)
    }

    /// 내장된 알림 소리를 지정된 볼륨으로 재생
    pub fn play_alarm(&self, volume: f32) {
        // 이미 재생 중이면 중복 실행 방지
        if IS_PLAYING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            log::info!("알림 이미 재생 중, 스킵");
            return;
        }

        std::thread::spawn(move || {
            // 내장된 wav 데이터를 임시 파일에 쓰기
            let tmp_path = std::env::temp_dir().join("presentation_timer_alarm.wav");
            if let Err(e) = std::fs::write(&tmp_path, ALARM_SOUND) {
                log::error!("알림 파일 쓰기 실패: {:?}", e);
                IS_PLAYING.store(false, Ordering::SeqCst);
                return;
            }

            // afplay로 재생 (macOS 기본 명령어)
            // -v 옵션: 볼륨 (0.0 ~ 1.0 → afplay는 0 ~ 255)
            let afplay_volume = (volume * 255.0).clamp(0.0, 255.0) as u32;
            log::info!("알림 재생 시작: afplay (볼륨: {})", afplay_volume);

            match std::process::Command::new("afplay")
                .arg(tmp_path.to_str().unwrap_or(""))
                .arg("-v")
                .arg(format!("{}", volume))
                .spawn()
            {
                Ok(mut child) => {
                    // 재생이 끝날 때까지 대기 (별도 스레드에서)
                    let _ = child.wait();
                    log::info!("알림 재생 완료");
                }
                Err(e) => {
                    log::error!("afplay 실행 실패: {:?}", e);
                }
            }

            IS_PLAYING.store(false, Ordering::SeqCst);
        });
    }
}
