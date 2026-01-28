//! 테트로미노 모듈
//!
//! 7종류의 테트로미노(I, O, T, S, Z, J, L)와 7-bag 랜덤 생성기를 정의합니다.

use super::playfield::CellColor;
use rand::SeedableRng;
use rand::seq::SliceRandom;

/// 테트로미노 종류 (7종)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TetrominoKind {
    /// I 미노 - 시안, 가로 4칸
    I,
    /// O 미노 - 노랑, 2×2 정사각형
    O,
    /// T 미노 - 보라, T자 모양
    T,
    /// S 미노 - 초록, S자 모양
    S,
    /// Z 미노 - 빨강, Z자 모양
    Z,
    /// J 미노 - 파랑, J자 모양
    J,
    /// L 미노 - 주황, L자 모양
    L,
}

impl TetrominoKind {
    /// 모든 테트로미노 종류 (7-bag용)
    pub const ALL: [TetrominoKind; 7] = [
        TetrominoKind::I,
        TetrominoKind::O,
        TetrominoKind::T,
        TetrominoKind::S,
        TetrominoKind::Z,
        TetrominoKind::J,
        TetrominoKind::L,
    ];

    /// 테트로미노의 색상 반환
    #[must_use]
    pub const fn color(self) -> CellColor {
        match self {
            TetrominoKind::I => CellColor::Cyan,
            TetrominoKind::O => CellColor::Yellow,
            TetrominoKind::T => CellColor::Purple,
            TetrominoKind::S => CellColor::Green,
            TetrominoKind::Z => CellColor::Red,
            TetrominoKind::J => CellColor::Blue,
            TetrominoKind::L => CellColor::Orange,
        }
    }

    /// 테트로미노의 초기 형태 (회전 상태 0)
    /// 4×4 매트릭스로 표현, true = 블록 존재
    /// 인덱스: [row][col], row 0이 상단
    #[must_use]
    pub const fn shape(self) -> [[bool; 4]; 4] {
        match self {
            // I 미노: 가로 4칸 (2행에 위치)
            // ....
            // ████
            // ....
            // ....
            TetrominoKind::I => [
                [false, false, false, false],
                [true, true, true, true],
                [false, false, false, false],
                [false, false, false, false],
            ],
            // O 미노: 2×2 정사각형 (중앙 상단)
            // .██.
            // .██.
            // ....
            // ....
            TetrominoKind::O => [
                [false, true, true, false],
                [false, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            // T 미노: T자 모양
            // .█.
            // ███
            // ...
            // ...
            TetrominoKind::T => [
                [false, true, false, false],
                [true, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            // S 미노: S자 모양
            // .██
            // ██.
            // ...
            // ...
            TetrominoKind::S => [
                [false, true, true, false],
                [true, true, false, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            // Z 미노: Z자 모양
            // ██.
            // .██
            // ...
            // ...
            TetrominoKind::Z => [
                [true, true, false, false],
                [false, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            // J 미노: J자 모양
            // █..
            // ███
            // ...
            // ...
            TetrominoKind::J => [
                [true, false, false, false],
                [true, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            // L 미노: L자 모양
            // ..█
            // ███
            // ...
            // ...
            TetrominoKind::L => [
                [false, false, true, false],
                [true, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
        }
    }

    /// 스폰 X 오프셋 (플레이필드 기준)
    /// I, O는 정중앙, 나머지는 좌측 정렬
    #[must_use]
    pub const fn spawn_x_offset(self) -> i32 {
        match self {
            // 10칸 너비 기준, 중앙 정렬
            TetrominoKind::I | TetrominoKind::O => 3,
            // 좌측 정렬
            _ => 3,
        }
    }

    /// 스폰 Y 오프셋 (플레이필드 기준, 바닥이 0)
    /// 20-21행에서 스폰 (가시 영역 상단에 부분적으로 보임)
    #[must_use]
    pub const fn spawn_y_offset(self) -> i32 {
        // 4×4 매트릭스 기준, 상단 2행(row 0-1)에 미노가 있으므로
        // row 0 → y+3 = 21, row 1 → y+2 = 20
        // 미노가 20-21행에 위치하여 스폰 시 즉시 보임
        18
    }
}

/// 활성 테트로미노 (플레이필드 위의 움직이는 미노)
#[derive(Debug, Clone)]
pub struct Tetromino {
    /// 테트로미노 종류
    pub kind: TetrominoKind,
    /// X 위치 (플레이필드 좌표, 매트릭스 좌측 기준)
    pub x: i32,
    /// Y 위치 (플레이필드 좌표, 매트릭스 하단 기준)
    pub y: i32,
    /// 현재 형태 (회전 적용됨)
    shape: [[bool; 4]; 4],
}

impl Tetromino {
    /// 새 테트로미노 생성 (스폰 위치)
    #[must_use]
    pub fn new(kind: TetrominoKind) -> Self {
        Self {
            kind,
            x: kind.spawn_x_offset(),
            y: kind.spawn_y_offset(),
            shape: kind.shape(),
        }
    }

    /// 현재 형태 반환
    #[must_use]
    pub const fn shape(&self) -> &[[bool; 4]; 4] {
        &self.shape
    }

    /// 테트로미노의 색상
    #[must_use]
    pub const fn color(&self) -> CellColor {
        self.kind.color()
    }

    /// 블록이 차지하는 절대 좌표 목록 반환
    /// (플레이필드 좌표계)
    #[must_use]
    pub fn blocks(&self) -> Vec<(i32, i32)> {
        let mut result = Vec::with_capacity(4);
        for (row_idx, row) in self.shape.iter().enumerate() {
            for (col_idx, &filled) in row.iter().enumerate() {
                if filled {
                    // 매트릭스 row 0이 상단이므로, y를 반전
                    let abs_x = self.x + col_idx as i32;
                    let abs_y = self.y + (3 - row_idx as i32);
                    result.push((abs_x, abs_y));
                }
            }
        }
        result
    }

    /// 시계방향 90도 회전한 형태 반환 (실제 적용 안함)
    #[must_use]
    pub fn rotated_cw(&self) -> [[bool; 4]; 4] {
        let mut rotated = [[false; 4]; 4];
        for (row, row_data) in self.shape.iter().enumerate() {
            for (col, &value) in row_data.iter().enumerate() {
                rotated[col][3 - row] = value;
            }
        }
        rotated
    }

    /// 반시계방향 90도 회전한 형태 반환 (실제 적용 안함)
    #[must_use]
    pub fn rotated_ccw(&self) -> [[bool; 4]; 4] {
        let mut rotated = [[false; 4]; 4];
        for (row, row_data) in self.shape.iter().enumerate() {
            for (col, &value) in row_data.iter().enumerate() {
                rotated[3 - col][row] = value;
            }
        }
        rotated
    }

    /// 시계방향 90도 회전 적용
    pub fn rotate_cw(&mut self) {
        // O 미노는 회전해도 형태가 동일하므로 무시
        if self.kind != TetrominoKind::O {
            self.shape = self.rotated_cw();
        }
    }

    /// 반시계방향 90도 회전 적용
    pub fn rotate_ccw(&mut self) {
        // O 미노는 회전해도 형태가 동일하므로 무시
        if self.kind != TetrominoKind::O {
            self.shape = self.rotated_ccw();
        }
    }

    /// 형태를 직접 설정 (Wall Kick 테스트용)
    pub fn set_shape(&mut self, shape: [[bool; 4]; 4]) {
        self.shape = shape;
    }

    /// 위치 이동
    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }

    /// 위치 설정
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}

/// 7-bag 랜덤 생성기
///
/// 7개의 테트로미노를 한 세트로 섞어서 순차적으로 배출합니다.
/// 세트가 비면 새로 섞어서 다음 세트를 준비합니다.
#[derive(Debug, Clone)]
pub struct SevenBag {
    /// 현재 bag의 남은 피스들
    current_bag: Vec<TetrominoKind>,
    /// 다음 bag (미리 준비)
    next_bag: Vec<TetrominoKind>,
    /// 난수 생성기
    rng: rand::rngs::StdRng,
}

impl SevenBag {
    /// 새로운 7-bag 생성기 (시스템 시드)
    #[must_use]
    pub fn new() -> Self {
        Self::with_seed(rand::random())
    }

    /// 지정된 시드로 7-bag 생성기 생성 (테스트용)
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let current_bag = Self::generate_bag(&mut rng);
        let next_bag = Self::generate_bag(&mut rng);
        Self {
            current_bag,
            next_bag,
            rng,
        }
    }

    /// 섞인 새 bag 생성
    fn generate_bag(rng: &mut rand::rngs::StdRng) -> Vec<TetrominoKind> {
        let mut bag: Vec<TetrominoKind> = TetrominoKind::ALL.to_vec();
        bag.shuffle(rng);
        bag
    }

    /// 다음 테트로미노 꺼내기
    pub fn pop_next(&mut self) -> TetrominoKind {
        // 현재 bag에서 꺼냄 (뒤에서부터)
        if let Some(kind) = self.current_bag.pop() {
            // 현재 bag이 비면 next_bag을 current로, 새 bag 생성
            if self.current_bag.is_empty() {
                std::mem::swap(&mut self.current_bag, &mut self.next_bag);
                self.next_bag = Self::generate_bag(&mut self.rng);
            }
            kind
        } else {
            // 이론상 도달 불가 (항상 bag이 채워져 있음)
            unreachable!("Bag should never be empty")
        }
    }

    /// 다음에 나올 N개의 테트로미노 미리보기 (꺼내지 않음)
    #[must_use]
    pub fn preview(&self, count: usize) -> Vec<TetrominoKind> {
        let mut result = Vec::with_capacity(count);

        // current_bag의 뒤에서부터 (next() 순서와 동일)
        let current_len = self.current_bag.len();
        for i in 0..count.min(current_len) {
            result.push(self.current_bag[current_len - 1 - i]);
        }

        // 부족하면 next_bag에서
        let remaining = count.saturating_sub(current_len);
        let next_len = self.next_bag.len();
        for i in 0..remaining.min(next_len) {
            result.push(self.next_bag[next_len - 1 - i]);
        }

        result
    }

    /// 현재 bag에 남은 피스 수
    #[must_use]
    pub fn remaining_in_bag(&self) -> usize {
        self.current_bag.len()
    }
}

impl Default for SevenBag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== TetrominoKind 테스트 ==========

    #[test]
    fn test_all_kinds() {
        assert_eq!(TetrominoKind::ALL.len(), 7);
    }

    #[test]
    fn test_kind_colors() {
        assert_eq!(TetrominoKind::I.color(), CellColor::Cyan);
        assert_eq!(TetrominoKind::O.color(), CellColor::Yellow);
        assert_eq!(TetrominoKind::T.color(), CellColor::Purple);
        assert_eq!(TetrominoKind::S.color(), CellColor::Green);
        assert_eq!(TetrominoKind::Z.color(), CellColor::Red);
        assert_eq!(TetrominoKind::J.color(), CellColor::Blue);
        assert_eq!(TetrominoKind::L.color(), CellColor::Orange);
    }

    #[test]
    fn test_i_shape() {
        let shape = TetrominoKind::I.shape();
        // I 미노는 2행에 4칸이 있어야 함
        assert!(!shape[0].iter().any(|&x| x)); // 0행 비어있음
        assert!(shape[1].iter().all(|&x| x)); // 1행 모두 채워짐
        assert!(!shape[2].iter().any(|&x| x)); // 2행 비어있음
        assert!(!shape[3].iter().any(|&x| x)); // 3행 비어있음
    }

    #[test]
    fn test_o_shape() {
        let shape = TetrominoKind::O.shape();
        // O 미노는 2×2 정사각형 (중앙)
        assert_eq!(shape[0], [false, true, true, false]);
        assert_eq!(shape[1], [false, true, true, false]);
        assert!(!shape[2].iter().any(|&x| x));
        assert!(!shape[3].iter().any(|&x| x));
    }

    #[test]
    fn test_t_shape() {
        let shape = TetrominoKind::T.shape();
        // T 미노: 0행에 중앙 1개, 1행에 3개
        assert_eq!(shape[0], [false, true, false, false]);
        assert_eq!(shape[1], [true, true, true, false]);
    }

    #[test]
    fn test_shape_has_4_blocks() {
        // 모든 테트로미노는 정확히 4개의 블록을 가짐
        for kind in TetrominoKind::ALL {
            let shape = kind.shape();
            let block_count: usize = shape
                .iter()
                .flat_map(|row| row.iter())
                .filter(|&&b| b)
                .count();
            assert_eq!(block_count, 4, "{:?} should have 4 blocks", kind);
        }
    }

    // ========== Tetromino 테스트 ==========

    #[test]
    fn test_tetromino_new() {
        let t = Tetromino::new(TetrominoKind::T);
        assert_eq!(t.kind, TetrominoKind::T);
        assert_eq!(t.x, 3);
        assert_eq!(t.y, 18);
    }

    #[test]
    fn test_tetromino_blocks() {
        let t = Tetromino::new(TetrominoKind::I);
        let blocks = t.blocks();
        assert_eq!(blocks.len(), 4);
        // I 미노는 가로 4칸 (y=20 높이에 위치)
        // 매트릭스 row 1에 블록이 있으므로 y = 18 + (3 - 1) = 20
        for (x, y) in &blocks {
            assert_eq!(*y, 20);
            assert!((3..=6).contains(x));
        }
    }

    #[test]
    fn test_tetromino_move() {
        let mut t = Tetromino::new(TetrominoKind::T);
        let initial_x = t.x;
        let initial_y = t.y;

        t.move_by(-1, 0);
        assert_eq!(t.x, initial_x - 1);
        assert_eq!(t.y, initial_y);

        t.move_by(0, -1);
        assert_eq!(t.x, initial_x - 1);
        assert_eq!(t.y, initial_y - 1);
    }

    #[test]
    fn test_tetromino_rotate_cw() {
        let mut t = Tetromino::new(TetrominoKind::T);
        // T 미노 초기:
        // .█.
        // ███
        let initial_shape = *t.shape();

        t.rotate_cw();
        // 시계방향 회전 후:
        // █.
        // ██
        // █.
        let rotated = t.shape();

        // 초기와 달라야 함
        assert_ne!(&initial_shape, rotated);

        // 4번 회전하면 원래대로
        t.rotate_cw();
        t.rotate_cw();
        t.rotate_cw();
        assert_eq!(&initial_shape, t.shape());
    }

    #[test]
    fn test_tetromino_rotate_ccw() {
        let mut t = Tetromino::new(TetrominoKind::T);
        let initial_shape = *t.shape();

        t.rotate_ccw();
        assert_ne!(&initial_shape, t.shape());

        // 4번 회전하면 원래대로
        t.rotate_ccw();
        t.rotate_ccw();
        t.rotate_ccw();
        assert_eq!(&initial_shape, t.shape());
    }

    #[test]
    fn test_rotate_cw_ccw_inverse() {
        // CW와 CCW는 서로 역연산
        let mut t = Tetromino::new(TetrominoKind::J);
        let initial = *t.shape();

        t.rotate_cw();
        t.rotate_ccw();
        assert_eq!(&initial, t.shape());

        t.rotate_ccw();
        t.rotate_cw();
        assert_eq!(&initial, t.shape());
    }

    #[test]
    fn test_o_rotation_unchanged() {
        // O 미노는 회전해도 형태가 동일
        let mut t = Tetromino::new(TetrominoKind::O);
        let initial = *t.shape();

        t.rotate_cw();
        assert_eq!(&initial, t.shape());

        t.rotate_ccw();
        assert_eq!(&initial, t.shape());
    }

    // ========== SevenBag 테스트 ==========

    #[test]
    fn test_seven_bag_deterministic() {
        // 같은 시드면 같은 순서
        let mut bag1 = SevenBag::with_seed(42);
        let mut bag2 = SevenBag::with_seed(42);

        for _ in 0..14 {
            assert_eq!(bag1.pop_next(), bag2.pop_next());
        }
    }

    #[test]
    fn test_seven_bag_no_duplicates_in_bag() {
        // 7개를 뽑으면 중복 없이 모든 종류가 나옴
        let mut bag = SevenBag::with_seed(12345);

        let mut first_seven: Vec<TetrominoKind> = (0..7).map(|_| bag.pop_next()).collect();
        first_seven.sort_by_key(|k| *k as u8);
        first_seven.dedup();
        assert_eq!(first_seven.len(), 7);

        // 다음 7개도 마찬가지
        let mut second_seven: Vec<TetrominoKind> = (0..7).map(|_| bag.pop_next()).collect();
        second_seven.sort_by_key(|k| *k as u8);
        second_seven.dedup();
        assert_eq!(second_seven.len(), 7);
    }

    #[test]
    fn test_seven_bag_remaining() {
        let mut bag = SevenBag::with_seed(99);
        assert_eq!(bag.remaining_in_bag(), 7);

        bag.pop_next();
        assert_eq!(bag.remaining_in_bag(), 6);

        // 6개 더 뽑으면 다음 bag으로
        for _ in 0..6 {
            bag.pop_next();
        }
        assert_eq!(bag.remaining_in_bag(), 7);
    }

    #[test]
    fn test_seven_bag_preview() {
        let mut bag = SevenBag::with_seed(777);

        // 5개 미리보기
        let preview = bag.preview(5);
        assert_eq!(preview.len(), 5);

        // 실제로 꺼내면 미리보기와 같아야 함
        for expected in preview {
            assert_eq!(bag.pop_next(), expected);
        }
    }

    #[test]
    fn test_seven_bag_preview_across_bags() {
        let mut bag = SevenBag::with_seed(555);

        // 5개 뽑아서 남은 개수를 2개로 만듦
        for _ in 0..5 {
            bag.pop_next();
        }
        assert_eq!(bag.remaining_in_bag(), 2);

        // 5개 미리보기 (현재 bag 2개 + 다음 bag 3개)
        let preview = bag.preview(5);
        assert_eq!(preview.len(), 5);

        // 실제로 꺼내서 확인
        for expected in preview {
            assert_eq!(bag.pop_next(), expected);
        }
    }

    #[test]
    fn test_seven_bag_long_run() {
        // 많이 뽑아도 문제없이 동작
        let mut bag = SevenBag::with_seed(1);
        for _ in 0..100 {
            let _ = bag.pop_next();
        }
    }
}
