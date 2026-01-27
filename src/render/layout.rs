//! 레이아웃 계산 모듈
//!
//! 화면 레이아웃 및 위치 계산을 담당합니다.
//! - 영역 정의 (HOLD, PLAYFIELD, NEXT, 점수 표시 등)
//! - 동적 크기 조절
//! - 좌표 변환

use crate::game::{PLAYFIELD_WIDTH, VISIBLE_HEIGHT};

// ============================================================================
// 상수 정의
// ============================================================================

/// 기본 윈도우 너비
pub const DEFAULT_WINDOW_WIDTH: u32 = 1280;

/// 기본 윈도우 높이
pub const DEFAULT_WINDOW_HEIGHT: u32 = 720;

/// 기본 셀 크기 (픽셀)
pub const DEFAULT_CELL_SIZE: u32 = 30;

/// 최소 셀 크기 (픽셀)
pub const MIN_CELL_SIZE: u32 = 10;

/// 최대 셀 크기 (픽셀)
pub const MAX_CELL_SIZE: u32 = 50;

/// 플레이필드 셀 너비
pub const FIELD_WIDTH: u32 = PLAYFIELD_WIDTH as u32;

/// 플레이필드 셀 높이 (가시 영역)
pub const FIELD_HEIGHT: u32 = VISIBLE_HEIGHT as u32;

/// Next 프리뷰 개수
pub const NEXT_PREVIEW_COUNT: u32 = 5;

/// 미노 프리뷰 크기 (셀 단위)
pub const PREVIEW_SIZE: u32 = 4;

/// 패딩 (셀 단위)
pub const PADDING: u32 = 1;

// ============================================================================
// 사각형 영역
// ============================================================================

/// 사각형 영역 (픽셀 단위)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    /// 새 사각형 생성
    #[must_use]
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// 우측 x 좌표
    #[must_use]
    pub const fn right(&self) -> i32 {
        self.x + self.width as i32
    }

    /// 하단 y 좌표
    #[must_use]
    pub const fn bottom(&self) -> i32 {
        self.y + self.height as i32
    }

    /// 중심 x 좌표
    #[must_use]
    pub const fn center_x(&self) -> i32 {
        self.x + (self.width / 2) as i32
    }

    /// 중심 y 좌표
    #[must_use]
    pub const fn center_y(&self) -> i32 {
        self.y + (self.height / 2) as i32
    }

    /// 점이 사각형 내부에 있는지 확인
    #[must_use]
    pub const fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }
}

// ============================================================================
// 레이아웃 계산
// ============================================================================

/// 화면 레이아웃
#[derive(Debug, Clone)]
pub struct Layout {
    /// 윈도우 크기
    pub window_width: u32,
    pub window_height: u32,

    /// 현재 셀 크기
    pub cell_size: u32,

    /// 플레이필드 영역
    pub playfield: Rect,

    /// 홀드 영역
    pub hold: Rect,

    /// Next 프리뷰 영역 (5개)
    pub next: [Rect; NEXT_PREVIEW_COUNT as usize],

    /// 점수 표시 영역
    pub score: Rect,

    /// 레벨 표시 영역
    pub level: Rect,

    /// 라인 표시 영역
    pub lines: Rect,
}

impl Layout {
    /// 윈도우 크기에 맞는 레이아웃 계산
    #[must_use]
    pub fn calculate(window_width: u32, window_height: u32) -> Self {
        // 셀 크기 계산 (플레이필드가 화면에 맞도록)
        let cell_size = Self::calculate_cell_size(window_width, window_height);

        // 플레이필드 크기 (픽셀)
        let field_pixel_width = FIELD_WIDTH * cell_size;
        let field_pixel_height = FIELD_HEIGHT * cell_size;

        // 사이드 패널 너비 (홀드, Next 영역)
        let side_panel_width = (PREVIEW_SIZE + PADDING * 2) * cell_size;

        // 전체 콘텐츠 너비
        let total_width = side_panel_width + field_pixel_width + side_panel_width;

        // 플레이필드 위치 (화면 중앙)
        let playfield_x = ((window_width - total_width) / 2 + side_panel_width) as i32;
        let playfield_y = ((window_height - field_pixel_height) / 2) as i32;

        let playfield = Rect::new(
            playfield_x,
            playfield_y,
            field_pixel_width,
            field_pixel_height,
        );

        // 홀드 영역 (플레이필드 왼쪽)
        let hold_size = PREVIEW_SIZE * cell_size;
        let hold = Rect::new(
            playfield_x - (PADDING * cell_size) as i32 - hold_size as i32,
            playfield_y,
            hold_size,
            hold_size,
        );

        // 점수/레벨/라인 영역 (홀드 아래)
        let info_y = hold.bottom() + (PADDING * cell_size) as i32;
        let info_width = hold_size;
        let info_height = cell_size * 2;

        let score = Rect::new(hold.x, info_y, info_width, info_height);
        let level = Rect::new(
            hold.x,
            score.bottom() + (PADDING * cell_size / 2) as i32,
            info_width,
            info_height,
        );
        let lines = Rect::new(
            hold.x,
            level.bottom() + (PADDING * cell_size / 2) as i32,
            info_width,
            info_height,
        );

        // Next 프리뷰 영역들 (플레이필드 오른쪽)
        let next_x = playfield.right() + (PADDING * cell_size) as i32;
        let next_size = PREVIEW_SIZE * cell_size;
        let next_gap = cell_size / 2;

        let next = std::array::from_fn(|i| {
            Rect::new(
                next_x,
                playfield_y + (i as u32 * (next_size + next_gap)) as i32,
                next_size,
                next_size,
            )
        });

        Self {
            window_width,
            window_height,
            cell_size,
            playfield,
            hold,
            next,
            score,
            level,
            lines,
        }
    }

    /// 적절한 셀 크기 계산
    fn calculate_cell_size(window_width: u32, window_height: u32) -> u32 {
        // 수평 기준: 플레이필드 + 양쪽 사이드 패널이 화면의 80%
        let side_panels = (PREVIEW_SIZE + PADDING * 2) * 2;
        let total_cells_width = FIELD_WIDTH + side_panels;
        let max_cell_by_width = (window_width * 80 / 100) / total_cells_width;

        // 수직 기준: 플레이필드가 화면의 90%
        let max_cell_by_height = (window_height * 90 / 100) / FIELD_HEIGHT;

        // 더 작은 값을 선택하고 범위 내로 클램핑
        let cell_size = max_cell_by_width.min(max_cell_by_height);
        cell_size.clamp(MIN_CELL_SIZE, MAX_CELL_SIZE)
    }

    /// 플레이필드 셀 좌표를 픽셀 좌표로 변환
    /// (0, 0)은 플레이필드 좌측 하단
    #[must_use]
    pub fn cell_to_pixel(&self, cell_x: i32, cell_y: i32) -> (i32, i32) {
        let px = self.playfield.x + cell_x * self.cell_size as i32;
        // Y축 반전 (플레이필드 좌표계는 하단이 0)
        let py = self.playfield.bottom() - (cell_y + 1) * self.cell_size as i32;
        (px, py)
    }

    /// 플레이필드 셀의 사각형 영역 반환
    #[must_use]
    pub fn cell_rect(&self, cell_x: i32, cell_y: i32) -> Rect {
        let (px, py) = self.cell_to_pixel(cell_x, cell_y);
        Rect::new(px, py, self.cell_size, self.cell_size)
    }

    /// 프리뷰 영역 내 미노 위치 계산
    /// 미노를 영역 중앙에 배치
    #[must_use]
    pub fn preview_mino_offset(&self, region: &Rect) -> (i32, i32) {
        // 4x4 미노 매트릭스를 영역 중앙에 배치
        let mino_size = 4 * self.cell_size;
        let offset_x = region.x + ((region.width - mino_size) / 2) as i32;
        let offset_y = region.y + ((region.height - mino_size) / 2) as i32;
        (offset_x, offset_y)
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::calculate(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
    }
}

// ============================================================================
// 테스트
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_new() {
        let r = Rect::new(10, 20, 100, 50);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.width, 100);
        assert_eq!(r.height, 50);
    }

    #[test]
    fn test_rect_right_bottom() {
        let r = Rect::new(10, 20, 100, 50);
        assert_eq!(r.right(), 110);
        assert_eq!(r.bottom(), 70);
    }

    #[test]
    fn test_rect_center() {
        let r = Rect::new(0, 0, 100, 50);
        assert_eq!(r.center_x(), 50);
        assert_eq!(r.center_y(), 25);
    }

    #[test]
    fn test_rect_contains() {
        let r = Rect::new(10, 10, 20, 20);
        assert!(r.contains(10, 10)); // 좌상단 모서리
        assert!(r.contains(15, 15)); // 내부
        assert!(r.contains(29, 29)); // 우하단 경계 직전
        assert!(!r.contains(30, 30)); // 경계 밖
        assert!(!r.contains(9, 15)); // 왼쪽 밖
    }

    #[test]
    fn test_layout_default() {
        let layout = Layout::default();
        assert_eq!(layout.window_width, DEFAULT_WINDOW_WIDTH);
        assert_eq!(layout.window_height, DEFAULT_WINDOW_HEIGHT);
    }

    #[test]
    fn test_layout_cell_size_default() {
        let layout = Layout::default();
        // 기본 윈도우 크기에서 셀 크기가 적절한 범위 내
        assert!(layout.cell_size >= MIN_CELL_SIZE);
        assert!(layout.cell_size <= MAX_CELL_SIZE);
    }

    #[test]
    fn test_layout_cell_size_small_window() {
        let layout = Layout::calculate(400, 300);
        // 작은 윈도우에서도 최소 셀 크기 보장
        assert!(layout.cell_size >= MIN_CELL_SIZE);
    }

    #[test]
    fn test_layout_cell_size_large_window() {
        let layout = Layout::calculate(3840, 2160); // 4K
        // 큰 윈도우에서도 최대 셀 크기 제한
        assert!(layout.cell_size <= MAX_CELL_SIZE);
    }

    #[test]
    fn test_layout_playfield_dimensions() {
        let layout = Layout::default();
        let expected_width = FIELD_WIDTH * layout.cell_size;
        let expected_height = FIELD_HEIGHT * layout.cell_size;
        assert_eq!(layout.playfield.width, expected_width);
        assert_eq!(layout.playfield.height, expected_height);
    }

    #[test]
    fn test_layout_playfield_centered() {
        let layout = Layout::default();
        // 플레이필드가 화면 중앙 근처에 있는지 확인
        let center_x = layout.window_width as i32 / 2;
        let playfield_center = layout.playfield.center_x();
        // 정확한 중앙은 아닐 수 있지만 근처에 있어야 함
        assert!((center_x - playfield_center).abs() < layout.window_width as i32 / 4);
    }

    #[test]
    fn test_layout_hold_left_of_playfield() {
        let layout = Layout::default();
        assert!(layout.hold.right() < layout.playfield.x);
    }

    #[test]
    fn test_layout_next_right_of_playfield() {
        let layout = Layout::default();
        assert!(layout.next[0].x > layout.playfield.right());
    }

    #[test]
    fn test_layout_next_count() {
        let layout = Layout::default();
        assert_eq!(layout.next.len(), NEXT_PREVIEW_COUNT as usize);
    }

    #[test]
    fn test_layout_next_vertical_order() {
        let layout = Layout::default();
        for i in 1..layout.next.len() {
            assert!(layout.next[i].y > layout.next[i - 1].y);
        }
    }

    #[test]
    fn test_layout_cell_to_pixel_origin() {
        let layout = Layout::default();
        let (px, py) = layout.cell_to_pixel(0, 0);
        // (0, 0)은 플레이필드 좌측 하단
        assert_eq!(px, layout.playfield.x);
        assert_eq!(py, layout.playfield.bottom() - layout.cell_size as i32);
    }

    #[test]
    fn test_layout_cell_to_pixel_top_right() {
        let layout = Layout::default();
        let (px, py) = layout.cell_to_pixel(9, 19);
        // (9, 19)는 플레이필드 우측 상단
        assert_eq!(px, layout.playfield.x + 9 * layout.cell_size as i32);
        assert_eq!(py, layout.playfield.y);
    }

    #[test]
    fn test_layout_cell_rect() {
        let layout = Layout::default();
        let rect = layout.cell_rect(5, 10);
        assert_eq!(rect.width, layout.cell_size);
        assert_eq!(rect.height, layout.cell_size);
    }

    #[test]
    fn test_layout_info_below_hold() {
        let layout = Layout::default();
        assert!(layout.score.y >= layout.hold.bottom());
        assert!(layout.level.y >= layout.score.bottom());
        assert!(layout.lines.y >= layout.level.bottom());
    }
}
