//! 게임 설정 관리 모듈
//!
//! DAS/ARR 등 플레이어 설정을 JSON 파일로 저장/로드합니다.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::input::{DEFAULT_ARR_MS, DEFAULT_DAS_MS};

/// 설정 파일 이름
const SETTINGS_FILE_NAME: &str = "settings.json";

/// 게임 설정
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameSettings {
    /// DAS (Delayed Auto Shift) 밀리초
    pub das_ms: u64,
    /// ARR (Auto Repeat Rate) 밀리초
    pub arr_ms: u64,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            das_ms: DEFAULT_DAS_MS,
            arr_ms: DEFAULT_ARR_MS,
        }
    }
}

impl GameSettings {
    /// DAS 유효 범위 (50-300ms)
    pub const DAS_MIN: u64 = 50;
    pub const DAS_MAX: u64 = 300;

    /// ARR 유효 범위 (0-100ms)
    pub const ARR_MIN: u64 = 0;
    pub const ARR_MAX: u64 = 100;

    /// DAS 값을 유효 범위로 클램핑하여 설정
    #[must_use]
    pub fn with_das(mut self, das_ms: u64) -> Self {
        self.das_ms = das_ms.clamp(Self::DAS_MIN, Self::DAS_MAX);
        self
    }

    /// ARR 값을 유효 범위로 클램핑하여 설정
    #[must_use]
    pub fn with_arr(mut self, arr_ms: u64) -> Self {
        self.arr_ms = arr_ms.clamp(Self::ARR_MIN, Self::ARR_MAX);
        self
    }

    /// 설정 파일 경로 결정
    ///
    /// 실행 파일과 같은 디렉토리에 `settings.json`을 저장합니다.
    #[must_use]
    pub fn settings_path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join(SETTINGS_FILE_NAME)))
            .unwrap_or_else(|| PathBuf::from(SETTINGS_FILE_NAME))
    }

    /// 파일에서 설정 로드 (없거나 파싱 실패 시 기본값 반환)
    #[must_use]
    pub fn load_from(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str::<Self>(&content).ok())
            .map(|s| s.clamped())
            .unwrap_or_default()
    }

    /// 기본 경로에서 설정 로드
    #[must_use]
    pub fn load() -> Self {
        Self::load_from(&Self::settings_path())
    }

    /// 파일에 설정 저장
    pub fn save_to(&self, path: &Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)
    }

    /// 기본 경로에 설정 저장
    pub fn save(&self) -> Result<(), std::io::Error> {
        self.save_to(&Self::settings_path())
    }

    /// 값을 유효 범위로 클램핑
    #[must_use]
    fn clamped(self) -> Self {
        Self {
            das_ms: self.das_ms.clamp(Self::DAS_MIN, Self::DAS_MAX),
            arr_ms: self.arr_ms.clamp(Self::ARR_MIN, Self::ARR_MAX),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_default_settings() {
        let settings = GameSettings::default();
        assert_eq!(settings.das_ms, DEFAULT_DAS_MS);
        assert_eq!(settings.arr_ms, DEFAULT_ARR_MS);
    }

    #[test]
    fn test_with_das_clamped() {
        let settings = GameSettings::default().with_das(10);
        assert_eq!(settings.das_ms, GameSettings::DAS_MIN);

        let settings = GameSettings::default().with_das(500);
        assert_eq!(settings.das_ms, GameSettings::DAS_MAX);

        let settings = GameSettings::default().with_das(150);
        assert_eq!(settings.das_ms, 150);
    }

    #[test]
    fn test_with_arr_clamped() {
        let settings = GameSettings::default().with_arr(200);
        assert_eq!(settings.arr_ms, GameSettings::ARR_MAX);

        let settings = GameSettings::default().with_arr(0);
        assert_eq!(settings.arr_ms, 0);

        let settings = GameSettings::default().with_arr(75);
        assert_eq!(settings.arr_ms, 75);
    }

    #[test]
    fn test_save_and_load() {
        let dir = std::env::temp_dir().join("tetris_test_settings");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test_settings.json");

        let settings = GameSettings::default().with_das(200).with_arr(30);
        settings.save_to(&path).expect("save failed");

        let loaded = GameSettings::load_from(&path);
        assert_eq!(loaded, settings);

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_load_missing_file() {
        let path = PathBuf::from("/tmp/nonexistent_settings_12345.json");
        let settings = GameSettings::load_from(&path);
        assert_eq!(settings, GameSettings::default());
    }

    #[test]
    fn test_load_invalid_json() {
        let dir = std::env::temp_dir().join("tetris_test_invalid");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("bad_settings.json");

        let mut f = fs::File::create(&path).unwrap();
        f.write_all(b"not valid json").unwrap();

        let settings = GameSettings::load_from(&path);
        assert_eq!(settings, GameSettings::default());

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_load_out_of_range_clamped() {
        let dir = std::env::temp_dir().join("tetris_test_clamp");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("clamp_settings.json");

        let json = r#"{"das_ms": 999, "arr_ms": 999}"#;
        fs::write(&path, json).unwrap();

        let settings = GameSettings::load_from(&path);
        assert_eq!(settings.das_ms, GameSettings::DAS_MAX);
        assert_eq!(settings.arr_ms, GameSettings::ARR_MAX);

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let settings = GameSettings {
            das_ms: 150,
            arr_ms: 40,
        };
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: GameSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, settings);
    }
}
