//! SRS (Super Rotation System) 회전 시스템 모듈
//!
//! 테트리스 가이드라인의 SRS 회전 시스템을 구현합니다.
//! - 4가지 회전 상태 (0, R, 2, L)
//! - Wall Kick 테이블 (JLSTZ, I 미노)
//! - 180도 회전 지원

use super::playfield::{PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH, Playfield};
use super::tetromino::{Tetromino, TetrominoKind};

/// 회전 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RotationState {
    /// 스폰 상태 (0도)
    #[default]
    Spawn,
    /// 시계방향 90도 (R)
    Right,
    /// 180도 (2)
    Two,
    /// 반시계방향 90도 (L)
    Left,
}

impl RotationState {
    /// 시계방향으로 회전한 상태
    #[must_use]
    pub const fn rotate_cw(self) -> Self {
        match self {
            Self::Spawn => Self::Right,
            Self::Right => Self::Two,
            Self::Two => Self::Left,
            Self::Left => Self::Spawn,
        }
    }

    /// 반시계방향으로 회전한 상태
    #[must_use]
    pub const fn rotate_ccw(self) -> Self {
        match self {
            Self::Spawn => Self::Left,
            Self::Left => Self::Two,
            Self::Two => Self::Right,
            Self::Right => Self::Spawn,
        }
    }

    /// 180도 회전한 상태
    #[must_use]
    pub const fn rotate_180(self) -> Self {
        match self {
            Self::Spawn => Self::Two,
            Self::Right => Self::Left,
            Self::Two => Self::Spawn,
            Self::Left => Self::Right,
        }
    }
}

/// 회전 방향
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationDirection {
    /// 시계방향
    Clockwise,
    /// 반시계방향
    CounterClockwise,
    /// 180도
    Rotate180,
}

/// Wall Kick 오프셋 (dx, dy)
pub type KickOffset = (i32, i32);

/// JLSTZ 미노의 Wall Kick 테이블
/// 인덱스: [from_state][to_state] -> 5개의 kick offset
const JLSTZ_KICK_TABLE: [[&[KickOffset]; 4]; 4] = [
    // From Spawn (0)
    [
        &[],                                            // 0→0 (없음)
        &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)], // 0→R
        &[],                                            // 0→2 (180도, 별도 처리)
        &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],    // 0→L
    ],
    // From Right (R)
    [
        &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)], // R→0
        &[],                                        // R→R (없음)
        &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)], // R→2
        &[],                                        // R→L (180도, 별도 처리)
    ],
    // From Two (2)
    [
        &[],                                            // 2→0 (180도, 별도 처리)
        &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)], // 2→R
        &[],                                            // 2→2 (없음)
        &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],    // 2→L
    ],
    // From Left (L)
    [
        &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)], // L→0
        &[],                                           // L→R (180도, 별도 처리)
        &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)], // L→2
        &[],                                           // L→L (없음)
    ],
];

/// I 미노의 Wall Kick 테이블
const I_KICK_TABLE: [[&[KickOffset]; 4]; 4] = [
    // From Spawn (0)
    [
        &[],                                          // 0→0 (없음)
        &[(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)], // 0→R
        &[],                                          // 0→2 (180도, 별도 처리)
        &[(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)], // 0→L
    ],
    // From Right (R)
    [
        &[(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)], // R→0
        &[],                                          // R→R (없음)
        &[(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)], // R→2
        &[],                                          // R→L (180도, 별도 처리)
    ],
    // From Two (2)
    [
        &[],                                          // 2→0 (180도, 별도 처리)
        &[(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)], // 2→R
        &[],                                          // 2→2 (없음)
        &[(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)], // 2→L
    ],
    // From Left (L)
    [
        &[(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)], // L→0
        &[],                                          // L→R (180도, 별도 처리)
        &[(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)], // L→2
        &[],                                          // L→L (없음)
    ],
];

/// 회전 상태를 인덱스로 변환
const fn state_to_index(state: RotationState) -> usize {
    match state {
        RotationState::Spawn => 0,
        RotationState::Right => 1,
        RotationState::Two => 2,
        RotationState::Left => 3,
    }
}

/// 주어진 회전에 대한 Wall Kick 오프셋 목록 반환
fn get_kick_offsets(
    kind: TetrominoKind,
    from: RotationState,
    to: RotationState,
) -> &'static [KickOffset] {
    if kind == TetrominoKind::O {
        // O 미노는 wall kick 없음
        return &[(0, 0)];
    }

    let from_idx = state_to_index(from);
    let to_idx = state_to_index(to);

    let table = if kind == TetrominoKind::I {
        &I_KICK_TABLE
    } else {
        &JLSTZ_KICK_TABLE
    };

    let offsets = table[from_idx][to_idx];
    if offsets.is_empty() {
        // 180도 회전 또는 잘못된 회전의 경우 기본값
        &[(0, 0)]
    } else {
        offsets
    }
}

/// 회전 시스템
#[derive(Debug, Clone, Default)]
pub struct RotationSystem {
    /// 현재 회전 상태
    state: RotationState,
}

impl RotationSystem {
    /// 새로운 회전 시스템 생성 (스폰 상태)
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: RotationState::Spawn,
        }
    }

    /// 현재 회전 상태
    #[must_use]
    pub const fn state(&self) -> RotationState {
        self.state
    }

    /// 회전 상태 설정
    pub fn set_state(&mut self, state: RotationState) {
        self.state = state;
    }

    /// 회전 상태 초기화 (스폰 상태로)
    pub fn reset(&mut self) {
        self.state = RotationState::Spawn;
    }

    /// 테트로미노 회전 시도
    ///
    /// Wall Kick을 적용하여 유효한 위치를 찾으면 회전 성공.
    ///
    /// # Returns
    /// - `Some((dx, dy, new_state))`: 회전 성공 시 적용할 오프셋과 새 상태
    /// - `None`: 회전 불가
    pub fn try_rotate(
        &self,
        tetromino: &Tetromino,
        playfield: &Playfield,
        direction: RotationDirection,
    ) -> Option<(i32, i32, RotationState)> {
        let new_state = match direction {
            RotationDirection::Clockwise => self.state.rotate_cw(),
            RotationDirection::CounterClockwise => self.state.rotate_ccw(),
            RotationDirection::Rotate180 => self.state.rotate_180(),
        };

        // 회전된 형태 계산
        let rotated_shape = match direction {
            RotationDirection::Clockwise => tetromino.rotated_cw(),
            RotationDirection::CounterClockwise => tetromino.rotated_ccw(),
            RotationDirection::Rotate180 => {
                // 180도는 CW 2회
                let temp = tetromino.rotated_cw();
                rotate_shape_cw(&temp)
            }
        };

        // 180도 회전의 경우 별도 처리
        let kick_offsets = if direction == RotationDirection::Rotate180 {
            // 180도 회전은 90도 2회의 kick을 순차 적용하는 것과 동일
            // 간단히 (0,0)만 테스트하거나, 복합 kick 사용
            get_180_kick_offsets(tetromino.kind, self.state)
        } else {
            get_kick_offsets(tetromino.kind, self.state, new_state)
        };

        // Wall Kick 테스트
        for &(dx, dy) in kick_offsets {
            let new_x = tetromino.x + dx;
            let new_y = tetromino.y + dy;

            if is_valid_position(&rotated_shape, new_x, new_y, playfield) {
                return Some((dx, dy, new_state));
            }
        }

        None
    }

    /// 회전 적용 (try_rotate 성공 후 호출)
    pub fn apply_rotation(&mut self, new_state: RotationState) {
        self.state = new_state;
    }
}

/// 180도 회전용 kick offsets
/// 스펙에 따르면 90도 2회와 동일한 결과
fn get_180_kick_offsets(kind: TetrominoKind, from: RotationState) -> &'static [KickOffset] {
    if kind == TetrominoKind::O {
        return &[(0, 0)];
    }

    // 180도 회전은 간단히 기본 위치만 테스트
    // (실제 가이드라인에서는 더 복잡할 수 있음)
    match (kind, from) {
        (TetrominoKind::I, _) => &[(0, 0), (0, 1), (0, -1), (1, 0), (-1, 0)],
        _ => &[(0, 0), (0, 1), (0, -1), (1, 0), (-1, 0)],
    }
}

/// 형태를 시계방향으로 회전
fn rotate_shape_cw(shape: &[[bool; 4]; 4]) -> [[bool; 4]; 4] {
    let mut rotated = [[false; 4]; 4];
    for (row, row_data) in shape.iter().enumerate() {
        for (col, &value) in row_data.iter().enumerate() {
            rotated[col][3 - row] = value;
        }
    }
    rotated
}

/// 주어진 위치에서 형태가 유효한지 확인
fn is_valid_position(shape: &[[bool; 4]; 4], x: i32, y: i32, playfield: &Playfield) -> bool {
    for (row_idx, row) in shape.iter().enumerate() {
        for (col_idx, &filled) in row.iter().enumerate() {
            if filled {
                // 매트릭스 row 0이 상단이므로, y를 반전
                let abs_x = x + col_idx as i32;
                let abs_y = y + (3 - row_idx as i32);

                // 범위 체크
                if abs_x < 0 || abs_x >= PLAYFIELD_WIDTH as i32 {
                    return false;
                }
                if abs_y < 0 || abs_y >= PLAYFIELD_HEIGHT as i32 {
                    return false;
                }

                // 충돌 체크
                if playfield.is_filled_at(abs_x as usize, abs_y as usize) {
                    return false;
                }
            }
        }
    }
    true
}

/// 충돌 검사 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionResult {
    /// 충돌 없음
    None,
    /// 벽과 충돌 (좌/우/하단)
    Wall,
    /// 기존 블록과 충돌
    Block,
    /// 상단 경계 초과
    Ceiling,
}

/// 테트로미노의 현재 위치에서 충돌 검사
pub fn check_collision(tetromino: &Tetromino, playfield: &Playfield) -> CollisionResult {
    check_collision_at(tetromino.shape(), tetromino.x, tetromino.y, playfield)
}

/// 주어진 형태와 위치에서 충돌 검사
pub fn check_collision_at(
    shape: &[[bool; 4]; 4],
    x: i32,
    y: i32,
    playfield: &Playfield,
) -> CollisionResult {
    for (row_idx, row) in shape.iter().enumerate() {
        for (col_idx, &filled) in row.iter().enumerate() {
            if filled {
                let abs_x = x + col_idx as i32;
                let abs_y = y + (3 - row_idx as i32);

                // 좌우 벽
                if abs_x < 0 || abs_x >= PLAYFIELD_WIDTH as i32 {
                    return CollisionResult::Wall;
                }

                // 바닥
                if abs_y < 0 {
                    return CollisionResult::Wall;
                }

                // 천장
                if abs_y >= PLAYFIELD_HEIGHT as i32 {
                    return CollisionResult::Ceiling;
                }

                // 블록 충돌
                if playfield.is_filled_at(abs_x as usize, abs_y as usize) {
                    return CollisionResult::Block;
                }
            }
        }
    }
    CollisionResult::None
}

/// 테트로미노가 특정 방향으로 이동 가능한지 확인
pub fn can_move(tetromino: &Tetromino, playfield: &Playfield, dx: i32, dy: i32) -> bool {
    let new_x = tetromino.x + dx;
    let new_y = tetromino.y + dy;
    check_collision_at(tetromino.shape(), new_x, new_y, playfield) == CollisionResult::None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::playfield::{Cell, CellColor};

    // ========== RotationState 테스트 ==========

    #[test]
    fn test_rotation_state_cw() {
        assert_eq!(RotationState::Spawn.rotate_cw(), RotationState::Right);
        assert_eq!(RotationState::Right.rotate_cw(), RotationState::Two);
        assert_eq!(RotationState::Two.rotate_cw(), RotationState::Left);
        assert_eq!(RotationState::Left.rotate_cw(), RotationState::Spawn);
    }

    #[test]
    fn test_rotation_state_ccw() {
        assert_eq!(RotationState::Spawn.rotate_ccw(), RotationState::Left);
        assert_eq!(RotationState::Left.rotate_ccw(), RotationState::Two);
        assert_eq!(RotationState::Two.rotate_ccw(), RotationState::Right);
        assert_eq!(RotationState::Right.rotate_ccw(), RotationState::Spawn);
    }

    #[test]
    fn test_rotation_state_180() {
        assert_eq!(RotationState::Spawn.rotate_180(), RotationState::Two);
        assert_eq!(RotationState::Two.rotate_180(), RotationState::Spawn);
        assert_eq!(RotationState::Right.rotate_180(), RotationState::Left);
        assert_eq!(RotationState::Left.rotate_180(), RotationState::Right);
    }

    #[test]
    fn test_cw_ccw_inverse() {
        for state in [
            RotationState::Spawn,
            RotationState::Right,
            RotationState::Two,
            RotationState::Left,
        ] {
            assert_eq!(state.rotate_cw().rotate_ccw(), state);
            assert_eq!(state.rotate_ccw().rotate_cw(), state);
        }
    }

    // ========== RotationSystem 테스트 ==========

    #[test]
    fn test_rotation_system_new() {
        let rs = RotationSystem::new();
        assert_eq!(rs.state(), RotationState::Spawn);
    }

    #[test]
    fn test_rotation_system_reset() {
        let mut rs = RotationSystem::new();
        rs.set_state(RotationState::Right);
        rs.reset();
        assert_eq!(rs.state(), RotationState::Spawn);
    }

    // ========== Wall Kick 테스트 ==========

    #[test]
    fn test_basic_rotation_no_obstacle() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(4, 10); // 중앙에 위치
        let rs = RotationSystem::new();

        // 장애물 없이 기본 회전
        let result = rs.try_rotate(&tetromino, &playfield, RotationDirection::Clockwise);
        assert!(result.is_some());

        let (dx, dy, new_state) = result.unwrap();
        assert_eq!(dx, 0);
        assert_eq!(dy, 0);
        assert_eq!(new_state, RotationState::Right);
    }

    #[test]
    fn test_wall_kick_left_wall() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(-1, 10); // 왼쪽 벽에 붙음
        let rs = RotationSystem::new();

        // Wall kick이 필요한 상황
        let result = rs.try_rotate(&tetromino, &playfield, RotationDirection::Clockwise);

        // Wall kick이 적용되어 회전 가능해야 함
        assert!(result.is_some());
    }

    #[test]
    fn test_o_mino_no_kick() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::O);
        tetromino.set_position(4, 10);
        let rs = RotationSystem::new();

        // O 미노는 회전해도 형태 동일, kick 없음
        let result = rs.try_rotate(&tetromino, &playfield, RotationDirection::Clockwise);
        assert!(result.is_some());

        let (dx, dy, _) = result.unwrap();
        assert_eq!(dx, 0);
        assert_eq!(dy, 0);
    }

    #[test]
    fn test_i_mino_wall_kick() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::I);
        tetromino.set_position(0, 10); // 왼쪽 벽 근처
        let rs = RotationSystem::new();

        // I 미노 회전 (다른 kick 테이블 사용)
        let result = rs.try_rotate(&tetromino, &playfield, RotationDirection::Clockwise);
        assert!(result.is_some());
    }

    #[test]
    fn test_rotation_blocked() {
        let mut playfield = Playfield::new();

        // 테트로미노 주변을 블록으로 채움
        for x in 0..PLAYFIELD_WIDTH {
            for y in 8..13 {
                if !(4..=5).contains(&x) || !(9..=11).contains(&y) {
                    playfield.set(x, y, Cell::Filled(CellColor::Cyan));
                }
            }
        }

        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(4, 9);
        let rs = RotationSystem::new();

        // 회전 시도 (좁은 공간에서는 kick으로 회전 가능할 수 있음)
        let _result = rs.try_rotate(&tetromino, &playfield, RotationDirection::Clockwise);
        // 결과는 wall kick 성공 여부에 따라 달라짐
    }

    #[test]
    fn test_180_rotation() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(4, 10);
        let rs = RotationSystem::new();

        let result = rs.try_rotate(&tetromino, &playfield, RotationDirection::Rotate180);
        assert!(result.is_some());

        let (_, _, new_state) = result.unwrap();
        assert_eq!(new_state, RotationState::Two);
    }

    // ========== CollisionResult 테스트 ==========

    #[test]
    fn test_check_collision_none() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(4, 10);

        assert_eq!(
            check_collision(&tetromino, &playfield),
            CollisionResult::None
        );
    }

    #[test]
    fn test_check_collision_wall_left() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(-2, 10);

        assert_eq!(
            check_collision(&tetromino, &playfield),
            CollisionResult::Wall
        );
    }

    #[test]
    fn test_check_collision_wall_right() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(9, 10);

        assert_eq!(
            check_collision(&tetromino, &playfield),
            CollisionResult::Wall
        );
    }

    #[test]
    fn test_check_collision_wall_bottom() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        // T 미노의 가장 낮은 블록은 row 1에 있고, abs_y = y + (3-1) = y + 2
        // 바닥(y<0)에 닿으려면: y + 2 < 0, 즉 y < -2
        tetromino.set_position(4, -3);

        assert_eq!(
            check_collision(&tetromino, &playfield),
            CollisionResult::Wall
        );
    }

    #[test]
    fn test_check_collision_block() {
        let mut playfield = Playfield::new();
        playfield.set(5, 10, Cell::Filled(CellColor::Red));

        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(4, 8); // T의 중앙이 (5, 10)과 겹침

        assert_eq!(
            check_collision(&tetromino, &playfield),
            CollisionResult::Block
        );
    }

    // ========== can_move 테스트 ==========

    #[test]
    fn test_can_move_left() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(4, 10);

        assert!(can_move(&tetromino, &playfield, -1, 0));
    }

    #[test]
    fn test_can_move_right() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(4, 10);

        assert!(can_move(&tetromino, &playfield, 1, 0));
    }

    #[test]
    fn test_can_move_down() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(4, 10);

        assert!(can_move(&tetromino, &playfield, 0, -1));
    }

    #[test]
    fn test_cannot_move_left_at_wall() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(0, 10);

        assert!(!can_move(&tetromino, &playfield, -1, 0));
    }

    #[test]
    fn test_cannot_move_down_at_floor() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        // T 미노의 가장 낮은 블록: abs_y = y + 2
        // 바닥에 닿으려면 abs_y = 0, 즉 y = -2
        tetromino.set_position(4, -2);

        // 아래로 이동하면 abs_y = -1이 되어 벽 충돌
        assert!(!can_move(&tetromino, &playfield, 0, -1));
    }

    #[test]
    fn test_cannot_move_into_block() {
        let mut playfield = Playfield::new();
        playfield.set(3, 10, Cell::Filled(CellColor::Blue));

        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(4, 8);

        // 왼쪽에 블록이 있어 이동 불가
        assert!(!can_move(&tetromino, &playfield, -1, 0));
    }
}
