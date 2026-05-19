use std::fs;

/// (폰트 표시 이름, 폰트 파일 절대 경로)
pub fn scan_system_fonts() -> Vec<(String, String)> {
    let mut fonts = Vec::new();
    
    // macOS의 기본 폰트 디렉토리 목록
    let dirs = [
        "/System/Library/Fonts",
        "/System/Library/Fonts/Supplemental",
        "/Library/Fonts",
    ];

    for dir in dirs {
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
    
    // 사용자 홈 디렉토리 폰트도 포함
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
