use std::fs;

/// (폰트 표시 이름, 폰트 파일 절대 경로)
pub fn scan_system_fonts() -> Vec<(String, String)> {
    let mut fonts = Vec::new();
    
    // macOS 및 Windows 폰트 디렉토리 목록 생성
    let mut dirs = vec![
        "/System/Library/Fonts".to_string(),
        "/System/Library/Fonts/Supplemental".to_string(),
        "/Library/Fonts".to_string(),
    ];

    // Windows 환경인 경우 시스템 폰트 폴더 추가
    #[cfg(target_os = "windows")]
    {
        if let Ok(sys_root) = std::env::var("SystemRoot") {
            dirs.push(format!("{}\\Fonts", sys_root));
        } else if let Ok(win_dir) = std::env::var("windir") {
            dirs.push(format!("{}\\Fonts", win_dir));
        } else {
            dirs.push("C:\\Windows\\Fonts".to_string());
        }
    }

    for dir in &dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ext_str == "ttf" || ext_str == "ttc" || ext_str == "otf" {
                        if let Some(stem) = path.file_stem() {
                            let name = stem.to_string_lossy().to_string();
                            fonts.push((name, path.to_string_lossy().to_string()));
                        }
                    }
                }
            }
        }
    }
    
    // 사용자 홈 디렉토리 폰트도 포함 (macOS 전용)
    #[cfg(target_os = "macos")]
    if let Ok(home) = std::env::var("HOME") {
        let user_font_dir = format!("{}/Library/Fonts", home);
        if let Ok(entries) = fs::read_dir(user_font_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ext_str == "ttf" || ext_str == "ttc" || ext_str == "otf" {
                        if let Some(stem) = path.file_stem() {
                            let name = stem.to_string_lossy().to_string();
                            fonts.push((name, path.to_string_lossy().to_string()));
                        }
                    }
                }
            }
        }
    }

    // 이름 순으로 정렬 및 중복 제거
    fonts.sort_by(|a, b| a.0.cmp(&b.0));
    fonts.dedup_by(|a, b| a.0 == b.0);

    // 기본 폰트들을 상단에 추가
    let mut final_fonts = vec![
        ("기본 (고정폭)".to_string(), "".to_string()),
    ];
    
    final_fonts.extend(fonts);
    final_fonts
}

