//! 색상 팔레트 모듈
//!
//! 테트리스 게임의 색상 정의 및 유틸리티

use crate::game::CellColor;

/// RGBA 색상 (0-255 범위)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// 새 RGBA 색상 생성
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// RGB 색상 생성 (알파 255)
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    /// 16진수 문자열로부터 생성 (예: "#1a1a2e")
    #[must_use]
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Self::rgb(r, g, b))
    }

    /// 알파값 적용한 새 색상 반환
    #[must_use]
    pub const fn with_alpha(self, a: u8) -> Self {
        Self::new(self.r, self.g, self.b, a)
    }

    /// 밝기 조절 (0.0 ~ 1.0+)
    #[must_use]
    pub fn with_brightness(self, factor: f32) -> Self {
        let r = ((self.r as f32 * factor).min(255.0)) as u8;
        let g = ((self.g as f32 * factor).min(255.0)) as u8;
        let b = ((self.b as f32 * factor).min(255.0)) as u8;
        Self::new(r, g, b, self.a)
    }

    /// 50% 투명도 적용 (고스트 피스용)
    #[must_use]
    pub const fn ghost(self) -> Self {
        self.with_alpha(128)
    }

    /// 80% 밝기 적용 (잠긴 블록용)
    #[must_use]
    pub fn locked(self) -> Self {
        self.with_brightness(0.8)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::rgb(0, 0, 0)
    }
}

// ============================================================================
// 색상 팔레트
// ============================================================================

/// 게임 색상 팔레트
pub struct Palette;

impl Palette {
    /// 배경색 (#1a1a2e)
    pub const BACKGROUND: Color = Color::rgb(0x1a, 0x1a, 0x2e);

    /// 그리드 색상 (#16213e)
    pub const GRID: Color = Color::rgb(0x16, 0x21, 0x3e);

    /// 그리드 라인 색상 (배경보다 약간 밝게)
    pub const GRID_LINE: Color = Color::rgb(0x25, 0x30, 0x50);

    /// 테두리 색상
    pub const BORDER: Color = Color::rgb(0x3a, 0x4a, 0x6e);

    /// 텍스트 색상
    pub const TEXT: Color = Color::rgb(0xff, 0xff, 0xff);

    /// 텍스트 보조 색상 (레이블 등)
    pub const TEXT_SECONDARY: Color = Color::rgb(0xaa, 0xaa, 0xaa);

    /// I 미노 - 시안 (#00FFFF)
    pub const CYAN: Color = Color::rgb(0x00, 0xff, 0xff);

    /// O 미노 - 노랑 (#FFFF00)
    pub const YELLOW: Color = Color::rgb(0xff, 0xff, 0x00);

    /// T 미노 - 보라 (#800080)
    pub const PURPLE: Color = Color::rgb(0x80, 0x00, 0x80);

    /// S 미노 - 초록 (#00FF00)
    pub const GREEN: Color = Color::rgb(0x00, 0xff, 0x00);

    /// Z 미노 - 빨강 (#FF0000)
    pub const RED: Color = Color::rgb(0xff, 0x00, 0x00);

    /// J 미노 - 파랑 (#0000FF)
    pub const BLUE: Color = Color::rgb(0x00, 0x00, 0xff);

    /// L 미노 - 주황 (#FFA500)
    pub const ORANGE: Color = Color::rgb(0xff, 0xa5, 0x00);

    /// CellColor를 Color로 변환
    #[must_use]
    pub const fn from_cell_color(cell_color: CellColor) -> Color {
        match cell_color {
            CellColor::Cyan => Self::CYAN,
            CellColor::Yellow => Self::YELLOW,
            CellColor::Purple => Self::PURPLE,
            CellColor::Green => Self::GREEN,
            CellColor::Red => Self::RED,
            CellColor::Blue => Self::BLUE,
            CellColor::Orange => Self::ORANGE,
        }
    }
}

// ============================================================================
// 테스트
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        let c = Color::new(100, 150, 200, 255);
        assert_eq!(c.r, 100);
        assert_eq!(c.g, 150);
        assert_eq!(c.b, 200);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_color_rgb() {
        let c = Color::rgb(100, 150, 200);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_color_from_hex() {
        let c = Color::from_hex("#1a1a2e").unwrap();
        assert_eq!(c.r, 0x1a);
        assert_eq!(c.g, 0x1a);
        assert_eq!(c.b, 0x2e);
    }

    #[test]
    fn test_color_from_hex_no_hash() {
        let c = Color::from_hex("1a1a2e").unwrap();
        assert_eq!(c.r, 0x1a);
    }

    #[test]
    fn test_color_from_hex_invalid() {
        assert!(Color::from_hex("invalid").is_none());
        assert!(Color::from_hex("#abc").is_none());
    }

    #[test]
    fn test_color_with_alpha() {
        let c = Color::rgb(100, 150, 200).with_alpha(128);
        assert_eq!(c.a, 128);
        assert_eq!(c.r, 100); // RGB 유지
    }

    #[test]
    fn test_color_ghost() {
        let c = Color::rgb(255, 0, 0).ghost();
        assert_eq!(c.a, 128);
    }

    #[test]
    fn test_color_with_brightness() {
        let c = Color::rgb(100, 100, 100).with_brightness(0.5);
        assert_eq!(c.r, 50);
        assert_eq!(c.g, 50);
        assert_eq!(c.b, 50);
    }

    #[test]
    fn test_color_with_brightness_clamped() {
        let c = Color::rgb(200, 200, 200).with_brightness(2.0);
        assert_eq!(c.r, 255); // 클램핑됨
    }

    #[test]
    fn test_color_locked() {
        let c = Color::rgb(100, 100, 100).locked();
        assert_eq!(c.r, 80);
    }

    #[test]
    fn test_palette_background() {
        assert_eq!(Palette::BACKGROUND.r, 0x1a);
        assert_eq!(Palette::BACKGROUND.g, 0x1a);
        assert_eq!(Palette::BACKGROUND.b, 0x2e);
    }

    #[test]
    fn test_palette_from_cell_color() {
        assert_eq!(Palette::from_cell_color(CellColor::Cyan), Palette::CYAN);
        assert_eq!(Palette::from_cell_color(CellColor::Yellow), Palette::YELLOW);
        assert_eq!(Palette::from_cell_color(CellColor::Purple), Palette::PURPLE);
        assert_eq!(Palette::from_cell_color(CellColor::Green), Palette::GREEN);
        assert_eq!(Palette::from_cell_color(CellColor::Red), Palette::RED);
        assert_eq!(Palette::from_cell_color(CellColor::Blue), Palette::BLUE);
        assert_eq!(Palette::from_cell_color(CellColor::Orange), Palette::ORANGE);
    }

    #[test]
    fn test_mino_colors_match_spec() {
        // 스펙에 정의된 색상 확인
        assert_eq!(Palette::CYAN, Color::rgb(0x00, 0xff, 0xff)); // I
        assert_eq!(Palette::YELLOW, Color::rgb(0xff, 0xff, 0x00)); // O
        assert_eq!(Palette::PURPLE, Color::rgb(0x80, 0x00, 0x80)); // T
        assert_eq!(Palette::GREEN, Color::rgb(0x00, 0xff, 0x00)); // S
        assert_eq!(Palette::RED, Color::rgb(0xff, 0x00, 0x00)); // Z
        assert_eq!(Palette::BLUE, Color::rgb(0x00, 0x00, 0xff)); // J
        assert_eq!(Palette::ORANGE, Color::rgb(0xff, 0xa5, 0x00)); // L
    }
}
