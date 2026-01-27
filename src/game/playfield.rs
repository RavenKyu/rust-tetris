//! 플레이필드 모듈
//!
//! 10×40 테트리스 플레이 그리드를 관리합니다.
//! - 가시 영역: 0-19행 (20행은 부분 표시)
//! - 버퍼 존: 20-39행 (숨김)
//! - 좌표계: 원점 (0, 0)은 좌측 하단

/// 플레이필드 너비 (열 수)
pub const PLAYFIELD_WIDTH: usize = 10;

/// 플레이필드 총 높이 (행 수)
pub const PLAYFIELD_HEIGHT: usize = 40;

/// 가시 영역 높이
pub const VISIBLE_HEIGHT: usize = 20;

/// 버퍼 존 시작 행
pub const BUFFER_ZONE_START: usize = 20;

/// 셀의 색상 (테트로미노 종류에 대응)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellColor {
    /// I 미노 - 시안
    Cyan,
    /// O 미노 - 노랑
    Yellow,
    /// T 미노 - 보라
    Purple,
    /// S 미노 - 초록
    Green,
    /// Z 미노 - 빨강
    Red,
    /// J 미노 - 파랑
    Blue,
    /// L 미노 - 주황
    Orange,
}

/// 플레이필드의 셀 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cell {
    /// 비어있는 셀
    #[default]
    Empty,
    /// 블록으로 채워진 셀
    Filled(CellColor),
}

impl Cell {
    /// 셀이 비어있는지 확인
    #[inline]
    pub const fn is_empty(self) -> bool {
        matches!(self, Cell::Empty)
    }

    /// 셀이 채워져 있는지 확인
    #[inline]
    pub const fn is_filled(self) -> bool {
        matches!(self, Cell::Filled(_))
    }
}

/// 테트리스 플레이필드 (10×40 그리드)
#[derive(Debug, Clone)]
pub struct Playfield {
    /// 셀 데이터 (행 우선, 인덱스 0이 최하단)
    cells: [[Cell; PLAYFIELD_WIDTH]; PLAYFIELD_HEIGHT],
}

impl Default for Playfield {
    fn default() -> Self {
        Self::new()
    }
}

impl Playfield {
    /// 새로운 빈 플레이필드 생성
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cells: [[Cell::Empty; PLAYFIELD_WIDTH]; PLAYFIELD_HEIGHT],
        }
    }

    /// 주어진 좌표의 셀 참조 (읽기 전용)
    ///
    /// # Arguments
    /// * `x` - X 좌표 (0-9)
    /// * `y` - Y 좌표 (0-39, 0이 최하단)
    ///
    /// # Returns
    /// 유효한 좌표면 `Some(&Cell)`, 범위 밖이면 `None`
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        self.cells.get(y).and_then(|row| row.get(x))
    }

    /// 주어진 좌표의 셀 설정
    ///
    /// # Arguments
    /// * `x` - X 좌표 (0-9)
    /// * `y` - Y 좌표 (0-39)
    /// * `cell` - 설정할 셀 상태
    ///
    /// # Returns
    /// 유효한 좌표면 `true`, 범위 밖이면 `false`
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) -> bool {
        if let Some(target) = self.cells.get_mut(y).and_then(|row| row.get_mut(x)) {
            *target = cell;
            true
        } else {
            false
        }
    }

    /// 주어진 좌표가 유효한지 확인
    #[inline]
    pub const fn is_valid_position(x: usize, y: usize) -> bool {
        x < PLAYFIELD_WIDTH && y < PLAYFIELD_HEIGHT
    }

    /// 주어진 좌표가 비어있는지 확인 (범위 밖이면 false)
    #[inline]
    pub fn is_empty_at(&self, x: usize, y: usize) -> bool {
        self.get(x, y).is_some_and(|cell| cell.is_empty())
    }

    /// 주어진 좌표가 채워져 있는지 확인 (범위 밖이면 false)
    #[inline]
    pub fn is_filled_at(&self, x: usize, y: usize) -> bool {
        self.get(x, y).is_some_and(|cell| cell.is_filled())
    }

    /// 특정 행이 완전히 채워졌는지 확인
    pub fn is_row_full(&self, y: usize) -> bool {
        if y >= PLAYFIELD_HEIGHT {
            return false;
        }
        self.cells[y].iter().all(|cell| cell.is_filled())
    }

    /// 특정 행이 완전히 비어있는지 확인
    pub fn is_row_empty(&self, y: usize) -> bool {
        if y >= PLAYFIELD_HEIGHT {
            return false;
        }
        self.cells[y].iter().all(|cell| cell.is_empty())
    }

    /// 완전히 채워진 행들을 찾아 반환
    pub fn find_full_rows(&self) -> Vec<usize> {
        (0..PLAYFIELD_HEIGHT)
            .filter(|&y| self.is_row_full(y))
            .collect()
    }

    /// 지정된 행들을 제거하고 상단 블록을 아래로 이동
    ///
    /// # Arguments
    /// * `rows` - 제거할 행 인덱스들 (정렬되지 않아도 됨)
    ///
    /// # Returns
    /// 실제로 제거된 행 수
    pub fn clear_rows(&mut self, rows: &[usize]) -> usize {
        if rows.is_empty() {
            return 0;
        }

        // 내림차순 정렬 (위에서 아래로 처리)
        let mut sorted_rows: Vec<usize> = rows.to_vec();
        sorted_rows.sort_unstable_by(|a, b| b.cmp(a));
        sorted_rows.dedup();

        let mut cleared = 0;
        for y in sorted_rows {
            if y < PLAYFIELD_HEIGHT {
                self.remove_row(y);
                cleared += 1;
            }
        }
        cleared
    }

    /// 채워진 모든 행을 찾아서 제거
    ///
    /// # Returns
    /// 제거된 행 수 (0-4)
    pub fn clear_full_rows(&mut self) -> usize {
        let full_rows = self.find_full_rows();
        self.clear_rows(&full_rows)
    }

    /// 단일 행 제거 (위의 행들을 한 칸씩 아래로 이동)
    fn remove_row(&mut self, y: usize) {
        // y+1부터 끝까지 한 칸씩 아래로 복사
        for row_idx in y..(PLAYFIELD_HEIGHT - 1) {
            self.cells[row_idx] = self.cells[row_idx + 1];
        }
        // 최상단 행은 빈 행으로 설정
        self.cells[PLAYFIELD_HEIGHT - 1] = [Cell::Empty; PLAYFIELD_WIDTH];
    }

    /// 플레이필드가 완전히 비어있는지 확인 (Perfect Clear 판정용)
    pub fn is_empty(&self) -> bool {
        self.cells
            .iter()
            .all(|row| row.iter().all(|cell| cell.is_empty()))
    }

    /// 가시 영역 내에 블록이 있는지 확인
    pub fn has_blocks_in_visible_area(&self) -> bool {
        (0..VISIBLE_HEIGHT).any(|y| !self.is_row_empty(y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playfield_constants() {
        assert_eq!(PLAYFIELD_WIDTH, 10);
        assert_eq!(PLAYFIELD_HEIGHT, 40);
        assert_eq!(VISIBLE_HEIGHT, 20);
        assert_eq!(BUFFER_ZONE_START, 20);
    }

    #[test]
    fn test_cell_default() {
        let cell: Cell = Cell::default();
        assert!(cell.is_empty());
        assert!(!cell.is_filled());
    }

    #[test]
    fn test_cell_states() {
        let empty = Cell::Empty;
        let filled = Cell::Filled(CellColor::Cyan);

        assert!(empty.is_empty());
        assert!(!empty.is_filled());

        assert!(!filled.is_empty());
        assert!(filled.is_filled());
    }

    #[test]
    fn test_playfield_new() {
        let pf = Playfield::new();

        // 모든 셀이 비어있어야 함
        for y in 0..PLAYFIELD_HEIGHT {
            for x in 0..PLAYFIELD_WIDTH {
                assert!(
                    pf.is_empty_at(x, y),
                    "Cell at ({}, {}) should be empty",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn test_playfield_get_set() {
        let mut pf = Playfield::new();

        // 유효한 좌표에 셀 설정
        assert!(pf.set(5, 10, Cell::Filled(CellColor::Red)));
        assert_eq!(pf.get(5, 10), Some(&Cell::Filled(CellColor::Red)));

        // 비어있는 셀 확인
        assert!(pf.is_empty_at(0, 0));
        assert!(!pf.is_filled_at(0, 0));

        // 채워진 셀 확인
        assert!(!pf.is_empty_at(5, 10));
        assert!(pf.is_filled_at(5, 10));
    }

    #[test]
    fn test_playfield_boundary() {
        let mut pf = Playfield::new();

        // 경계 값 테스트
        assert!(pf.set(0, 0, Cell::Filled(CellColor::Blue))); // 좌측 하단
        assert!(pf.set(9, 0, Cell::Filled(CellColor::Blue))); // 우측 하단
        assert!(pf.set(0, 39, Cell::Filled(CellColor::Blue))); // 좌측 상단
        assert!(pf.set(9, 39, Cell::Filled(CellColor::Blue))); // 우측 상단

        // 범위 밖 테스트
        assert!(!pf.set(10, 0, Cell::Filled(CellColor::Blue)));
        assert!(!pf.set(0, 40, Cell::Filled(CellColor::Blue)));
        assert_eq!(pf.get(10, 0), None);
        assert_eq!(pf.get(0, 40), None);
    }

    #[test]
    fn test_is_valid_position() {
        assert!(Playfield::is_valid_position(0, 0));
        assert!(Playfield::is_valid_position(9, 39));
        assert!(!Playfield::is_valid_position(10, 0));
        assert!(!Playfield::is_valid_position(0, 40));
    }

    #[test]
    fn test_row_full_check() {
        let mut pf = Playfield::new();

        // 빈 행
        assert!(!pf.is_row_full(0));
        assert!(pf.is_row_empty(0));

        // 부분적으로 채워진 행
        pf.set(0, 0, Cell::Filled(CellColor::Cyan));
        assert!(!pf.is_row_full(0));
        assert!(!pf.is_row_empty(0));

        // 완전히 채워진 행
        for x in 0..PLAYFIELD_WIDTH {
            pf.set(x, 1, Cell::Filled(CellColor::Cyan));
        }
        assert!(pf.is_row_full(1));
        assert!(!pf.is_row_empty(1));
    }

    #[test]
    fn test_find_full_rows() {
        let mut pf = Playfield::new();

        // 0, 2, 4행을 채움
        for y in [0, 2, 4] {
            for x in 0..PLAYFIELD_WIDTH {
                pf.set(x, y, Cell::Filled(CellColor::Yellow));
            }
        }

        let full_rows = pf.find_full_rows();
        assert_eq!(full_rows, vec![0, 2, 4]);
    }

    #[test]
    fn test_clear_single_row() {
        let mut pf = Playfield::new();

        // 0행 채우기
        for x in 0..PLAYFIELD_WIDTH {
            pf.set(x, 0, Cell::Filled(CellColor::Purple));
        }
        // 1행에 표시용 블록
        pf.set(5, 1, Cell::Filled(CellColor::Green));

        // 0행 클리어
        let cleared = pf.clear_rows(&[0]);
        assert_eq!(cleared, 1);

        // 1행이 0행으로 이동했어야 함
        assert!(pf.is_filled_at(5, 0));
        assert!(pf.is_empty_at(5, 1));
    }

    #[test]
    fn test_clear_multiple_rows() {
        let mut pf = Playfield::new();

        // 0, 1, 2행 채우기
        for y in 0..3 {
            for x in 0..PLAYFIELD_WIDTH {
                pf.set(x, y, Cell::Filled(CellColor::Orange));
            }
        }
        // 3행에 표시용 블록
        pf.set(3, 3, Cell::Filled(CellColor::Blue));

        // 0, 1, 2행 클리어
        let cleared = pf.clear_rows(&[0, 1, 2]);
        assert_eq!(cleared, 3);

        // 3행이 0행으로 이동했어야 함
        assert!(pf.is_filled_at(3, 0));
        for y in 1..4 {
            assert!(pf.is_empty_at(3, y), "Row {} should be empty", y);
        }
    }

    #[test]
    fn test_clear_full_rows() {
        let mut pf = Playfield::new();

        // 0행과 2행 채우기 (1행은 부분)
        for x in 0..PLAYFIELD_WIDTH {
            pf.set(x, 0, Cell::Filled(CellColor::Red));
            pf.set(x, 2, Cell::Filled(CellColor::Red));
        }
        pf.set(0, 1, Cell::Filled(CellColor::Blue)); // 1행 부분 채움
        pf.set(5, 3, Cell::Filled(CellColor::Green)); // 3행 표시용

        let cleared = pf.clear_full_rows();
        assert_eq!(cleared, 2);

        // 1행의 블록이 0행으로, 3행의 블록이 1행으로 이동
        assert!(pf.is_filled_at(0, 0));
        assert!(pf.is_filled_at(5, 1));
    }

    #[test]
    fn test_is_empty() {
        let mut pf = Playfield::new();
        assert!(pf.is_empty());

        pf.set(0, 0, Cell::Filled(CellColor::Cyan));
        assert!(!pf.is_empty());
    }

    #[test]
    fn test_has_blocks_in_visible_area() {
        let mut pf = Playfield::new();
        assert!(!pf.has_blocks_in_visible_area());

        // 버퍼 존에만 블록 (가시 영역에는 없음)
        pf.set(0, BUFFER_ZONE_START, Cell::Filled(CellColor::Cyan));
        assert!(!pf.has_blocks_in_visible_area());

        // 가시 영역에 블록
        pf.set(5, 10, Cell::Filled(CellColor::Yellow));
        assert!(pf.has_blocks_in_visible_area());
    }

    #[test]
    fn test_tetris_clear() {
        // 테트리스 (4줄 동시 클리어) 시나리오
        let mut pf = Playfield::new();

        // 0-3행 채우기
        for y in 0..4 {
            for x in 0..PLAYFIELD_WIDTH {
                pf.set(x, y, Cell::Filled(CellColor::Cyan));
            }
        }
        // 4행에 표시용 블록
        pf.set(7, 4, Cell::Filled(CellColor::Red));

        let cleared = pf.clear_full_rows();
        assert_eq!(cleared, 4, "Should clear 4 rows (Tetris)");

        // 4행이 0행으로 이동
        assert!(pf.is_filled_at(7, 0));
        assert!(pf.is_empty_at(7, 1));
    }
}
