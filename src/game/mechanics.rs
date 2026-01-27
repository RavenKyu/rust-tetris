//! 게임 메커니즘 모듈
//!
//! 테트리스 게임의 핵심 메커니즘을 구현합니다:
//! - 중력 시스템 (레벨별 낙하 속도)
//! - 락다운 시스템 (Extended Placement)
//! - 홀드 시스템
//! - 고스트 피스 계산
//! - 게임 오버 판정

use std::time::Duration;

use super::{
    CollisionResult, Playfield, Tetromino, TetrominoKind, VISIBLE_HEIGHT, can_move,
    check_collision_at,
};

// ============================================================================
// 상수 정의
// ============================================================================

/// 락다운 딜레이 (밀리초)
pub const LOCKDOWN_DELAY_MS: u64 = 500;

/// 락다운 리셋 최대 횟수 (Extended Placement)
pub const MAX_MOVE_RESETS: u32 = 15;

/// 레벨별 중력 테이블 (초당 셀 단위)
/// 인덱스 = 레벨 (0부터 시작), 값 = 초당 낙하 행 수
/// tetris-core.md 섹션 8.2 기준 (초/줄의 역수)
const GRAVITY_TABLE: [f64; 20] = [
    0.8,     // Level 0: 스폰 전용 (약 1.25초/셀)
    1.0,     // Level 1: 1.0초/셀
    1.261,   // Level 2: 0.793초/셀
    1.618,   // Level 3: 0.618초/셀
    2.114,   // Level 4: 0.473초/셀
    2.817,   // Level 5: 0.355초/셀
    3.817,   // Level 6: 0.262초/셀
    5.263,   // Level 7: 0.190초/셀
    7.407,   // Level 8: 0.135초/셀
    10.638,  // Level 9: 0.094초/셀
    15.625,  // Level 10: 0.064초/셀
    23.256,  // Level 11: 0.043초/셀
    23.256,  // Level 12: 0.043초/셀
    35.714,  // Level 13: 0.028초/셀
    35.714,  // Level 14: 0.028초/셀
    35.714,  // Level 15: 0.028초/셀
    55.556,  // Level 16: 0.018초/셀
    55.556,  // Level 17: 0.018초/셀
    55.556,  // Level 18: 0.018초/셀
    90.909,  // Level 19+: 0.011초/셀 (≈20G)
];

// ============================================================================
// 중력 시스템
// ============================================================================

/// 중력 시스템 - 레벨에 따른 자동 낙하 처리
#[derive(Debug, Clone)]
pub struct GravitySystem {
    /// 현재 레벨
    level: u32,
    /// 누적된 낙하량 (행 단위)
    accumulator: f64,
}

impl GravitySystem {
    /// 새 중력 시스템 생성
    #[must_use]
    pub const fn new(level: u32) -> Self {
        Self {
            level,
            accumulator: 0.0,
        }
    }

    /// 현재 레벨 반환
    #[must_use]
    pub const fn level(&self) -> u32 {
        self.level
    }

    /// 레벨 설정
    pub fn set_level(&mut self, level: u32) {
        self.level = level;
    }

    /// 현재 레벨의 중력 값 (초당 행 수) 반환
    #[must_use]
    pub fn gravity(&self) -> f64 {
        let idx = self.level.min(19) as usize;
        GRAVITY_TABLE[idx]
    }

    /// 시간 경과에 따른 낙하 횟수 계산
    /// delta: 경과 시간
    /// 반환: 낙하해야 할 행 수
    #[must_use]
    pub fn update(&mut self, delta: Duration) -> u32 {
        let gravity = self.gravity();
        let delta_secs = delta.as_secs_f64();
        self.accumulator += gravity * delta_secs;

        let drops = self.accumulator.floor() as u32;
        self.accumulator -= drops as f64;
        drops
    }

    /// 누적량 리셋 (새 미노 스폰 시)
    pub fn reset_accumulator(&mut self) {
        self.accumulator = 0.0;
    }
}

impl Default for GravitySystem {
    fn default() -> Self {
        Self::new(0)
    }
}

// ============================================================================
// 락다운 시스템
// ============================================================================

/// 락다운 시스템 - Extended Placement 방식
#[derive(Debug, Clone)]
pub struct LockdownSystem {
    /// 락다운 타이머
    timer: Duration,
    /// 무브 리셋 횟수
    move_resets: u32,
    /// 바닥에 닿아있는지 여부
    is_grounded: bool,
}

impl LockdownSystem {
    /// 새 락다운 시스템 생성
    #[must_use]
    pub const fn new() -> Self {
        Self {
            timer: Duration::ZERO,
            move_resets: 0,
            is_grounded: false,
        }
    }

    /// 현재 타이머 값 반환
    #[must_use]
    pub const fn timer(&self) -> Duration {
        self.timer
    }

    /// 무브 리셋 횟수 반환
    #[must_use]
    pub const fn move_resets(&self) -> u32 {
        self.move_resets
    }

    /// 바닥 접촉 여부 반환
    #[must_use]
    pub const fn is_grounded(&self) -> bool {
        self.is_grounded
    }

    /// 락다운 딜레이 상수 반환
    #[must_use]
    pub const fn lockdown_delay() -> Duration {
        Duration::from_millis(LOCKDOWN_DELAY_MS)
    }

    /// 바닥 접촉 상태 업데이트
    /// 테트로미노가 바닥(또는 블록)에 닿아있는지 확인
    pub fn update_grounded(&mut self, tetromino: &Tetromino, playfield: &Playfield) {
        let was_grounded = self.is_grounded;
        self.is_grounded = !can_move(tetromino, playfield, 0, -1);

        // 처음 바닥에 닿았을 때 타이머 시작
        if self.is_grounded && !was_grounded {
            self.timer = Duration::ZERO;
        }
    }

    /// 시간 경과 업데이트
    /// 반환: 락다운이 발생해야 하면 true
    #[must_use]
    pub fn update_timer(&mut self, delta: Duration) -> bool {
        if self.is_grounded {
            self.timer += delta;
            self.timer >= Self::lockdown_delay()
        } else {
            false
        }
    }

    /// 무브 리셋 시도
    /// 성공하면 true, 리셋 한도 초과시 false
    pub fn try_reset(&mut self) -> bool {
        if self.move_resets < MAX_MOVE_RESETS {
            self.move_resets += 1;
            self.timer = Duration::ZERO;
            true
        } else {
            false
        }
    }

    /// 새 미노를 위해 리셋
    pub fn reset(&mut self) {
        self.timer = Duration::ZERO;
        self.move_resets = 0;
        self.is_grounded = false;
    }
}

impl Default for LockdownSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 홀드 시스템
// ============================================================================

/// 홀드 시스템 - 현재 미노 저장/교환
#[derive(Debug, Clone, Default)]
pub struct HoldSystem {
    /// 홀드된 미노 종류
    held_piece: Option<TetrominoKind>,
    /// 홀드 가능 여부 (한 미노당 1회 제한)
    can_hold: bool,
}

/// 홀드 작업 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldResult {
    /// 홀드 성공, 기존 홀드 미노 반환
    Swapped(TetrominoKind),
    /// 홀드 성공, 홀드함이 비어있었음 (다음 미노 사용 필요)
    Stored,
    /// 홀드 불가 (이미 이번 미노에서 홀드함)
    NotAllowed,
}

impl HoldSystem {
    /// 새 홀드 시스템 생성
    #[must_use]
    pub const fn new() -> Self {
        Self {
            held_piece: None,
            can_hold: true,
        }
    }

    /// 홀드된 미노 종류 반환
    #[must_use]
    pub const fn held_piece(&self) -> Option<TetrominoKind> {
        self.held_piece
    }

    /// 홀드 가능 여부 반환
    #[must_use]
    pub const fn can_hold(&self) -> bool {
        self.can_hold
    }

    /// 홀드 시도
    /// current: 현재 테트로미노의 종류
    /// 반환: 홀드 결과
    pub fn hold(&mut self, current: TetrominoKind) -> HoldResult {
        if !self.can_hold {
            return HoldResult::NotAllowed;
        }

        self.can_hold = false;
        match self.held_piece {
            Some(held) => {
                self.held_piece = Some(current);
                HoldResult::Swapped(held)
            }
            None => {
                self.held_piece = Some(current);
                HoldResult::Stored
            }
        }
    }

    /// 새 미노 스폰 시 홀드 가능 상태로 리셋
    pub fn allow_hold(&mut self) {
        self.can_hold = true;
    }

    /// 전체 리셋 (게임 시작 시)
    pub fn reset(&mut self) {
        self.held_piece = None;
        self.can_hold = true;
    }
}

// ============================================================================
// 고스트 피스
// ============================================================================

/// 고스트 피스 Y 위치 계산
/// 현재 테트로미노가 하드 드롭했을 때의 Y 위치 반환
#[must_use]
pub fn calculate_ghost_position(tetromino: &Tetromino, playfield: &Playfield) -> i32 {
    let mut ghost_y = tetromino.y;

    // 아래로 한 칸씩 이동하면서 충돌 검사
    // 음수 y도 허용 (테트로미노 기준점이 음수여도 블록은 양수 위치에 있을 수 있음)
    loop {
        let test_y = ghost_y - 1;
        if check_collision_at(tetromino.shape(), tetromino.x, test_y, playfield)
            != CollisionResult::None
        {
            break;
        }
        ghost_y = test_y;
    }

    ghost_y
}

// ============================================================================
// 게임 오버 판정
// ============================================================================

/// 게임 오버 조건
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverReason {
    /// 스폰 시 겹침
    SpawnOverlap,
    /// 가시 영역 밖에서 잠김
    LockOutAboveVisible,
}

/// 스폰 시 게임 오버 판정
/// 새 테트로미노가 스폰될 때 기존 블록과 겹치면 게임 오버
#[must_use]
pub fn check_spawn_collision(tetromino: &Tetromino, playfield: &Playfield) -> bool {
    check_collision_at(tetromino.shape(), tetromino.x, tetromino.y, playfield)
        != CollisionResult::None
}

/// 락다운 시 게임 오버 판정
/// 테트로미노가 완전히 가시 영역 밖에서 잠기면 게임 오버
#[must_use]
pub fn check_lock_out(tetromino: &Tetromino) -> bool {
    let blocks = tetromino.blocks();
    blocks.iter().all(|&(_, y)| y >= VISIBLE_HEIGHT as i32)
}

/// 통합 게임 오버 판정
/// 반환: 게임 오버 사유 (없으면 None)
#[must_use]
pub fn check_game_over(
    tetromino: &Tetromino,
    playfield: &Playfield,
    is_spawn: bool,
) -> Option<GameOverReason> {
    if is_spawn && check_spawn_collision(tetromino, playfield) {
        return Some(GameOverReason::SpawnOverlap);
    }

    if !is_spawn && check_lock_out(tetromino) {
        return Some(GameOverReason::LockOutAboveVisible);
    }

    None
}

// ============================================================================
// 테스트
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // 중력 시스템 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_gravity_system_new() {
        let gs = GravitySystem::new(5);
        assert_eq!(gs.level(), 5);
    }

    #[test]
    fn test_gravity_system_default() {
        let gs = GravitySystem::default();
        assert_eq!(gs.level(), 0);
    }

    #[test]
    fn test_gravity_level_0() {
        let gs = GravitySystem::new(0);
        let g = gs.gravity();
        assert!((g - 0.8).abs() < 0.0001);
    }

    #[test]
    fn test_gravity_level_capped_at_19() {
        let gs = GravitySystem::new(100);
        let g = gs.gravity();
        assert!((g - 90.909).abs() < 0.001);
    }

    #[test]
    fn test_gravity_update_single_drop() {
        let mut gs = GravitySystem::new(18); // 55.556 cells/sec
        // 55.556 cells/sec, 20ms = 0.02초 → 1.11행 → 1행 낙하
        let drops = gs.update(Duration::from_millis(20));
        assert_eq!(drops, 1);
    }

    #[test]
    fn test_gravity_update_accumulation() {
        let mut gs = GravitySystem::new(0); // 0.8 cells/sec
        // 0.8 * 2초 = 1.6행 → 1행 낙하
        let drops = gs.update(Duration::from_secs(2));
        assert!(drops >= 1);
    }

    #[test]
    fn test_gravity_reset_accumulator() {
        let mut gs = GravitySystem::new(10);
        let _ = gs.update(Duration::from_millis(100));
        gs.reset_accumulator();
        // accumulator가 0이 되어 작은 delta로는 drop 없음
        let drops = gs.update(Duration::from_millis(1));
        assert_eq!(drops, 0);
    }

    // ------------------------------------------------------------------------
    // 락다운 시스템 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_lockdown_system_new() {
        let ls = LockdownSystem::new();
        assert_eq!(ls.timer(), Duration::ZERO);
        assert_eq!(ls.move_resets(), 0);
        assert!(!ls.is_grounded());
    }

    #[test]
    fn test_lockdown_delay_constant() {
        assert_eq!(LockdownSystem::lockdown_delay(), Duration::from_millis(500));
    }

    #[test]
    fn test_lockdown_timer_not_grounded() {
        let mut ls = LockdownSystem::new();
        let should_lock = ls.update_timer(Duration::from_millis(600));
        assert!(!should_lock);
    }

    #[test]
    fn test_lockdown_timer_grounded_triggers() {
        let mut ls = LockdownSystem::new();
        ls.is_grounded = true; // 테스트용 직접 설정

        let should_lock = ls.update_timer(Duration::from_millis(500));
        assert!(should_lock);
    }

    #[test]
    fn test_lockdown_try_reset_success() {
        let mut ls = LockdownSystem::new();
        ls.is_grounded = true;
        ls.timer = Duration::from_millis(300);

        assert!(ls.try_reset());
        assert_eq!(ls.timer(), Duration::ZERO);
        assert_eq!(ls.move_resets(), 1);
    }

    #[test]
    fn test_lockdown_try_reset_limit() {
        let mut ls = LockdownSystem::new();
        for _ in 0..MAX_MOVE_RESETS {
            assert!(ls.try_reset());
        }
        // 15번째 이후로는 실패
        assert!(!ls.try_reset());
        assert_eq!(ls.move_resets(), MAX_MOVE_RESETS);
    }

    #[test]
    fn test_lockdown_reset() {
        let mut ls = LockdownSystem::new();
        ls.is_grounded = true;
        ls.timer = Duration::from_millis(200);
        ls.move_resets = 5;

        ls.reset();
        assert_eq!(ls.timer(), Duration::ZERO);
        assert_eq!(ls.move_resets(), 0);
        assert!(!ls.is_grounded());
    }

    #[test]
    fn test_lockdown_update_grounded() {
        let mut ls = LockdownSystem::new();
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        // T 미노: row 1의 abs_y = y + 2, 바닥에 닿으려면 y = -2
        tetromino.set_position(3, -2);

        ls.update_grounded(&tetromino, &playfield);
        assert!(ls.is_grounded());
    }

    // ------------------------------------------------------------------------
    // 홀드 시스템 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_hold_system_new() {
        let hs = HoldSystem::new();
        assert!(hs.held_piece().is_none());
        assert!(hs.can_hold());
    }

    #[test]
    fn test_hold_first_time() {
        let mut hs = HoldSystem::new();
        let result = hs.hold(TetrominoKind::T);

        assert_eq!(result, HoldResult::Stored);
        assert_eq!(hs.held_piece(), Some(TetrominoKind::T));
        assert!(!hs.can_hold());
    }

    #[test]
    fn test_hold_swap() {
        let mut hs = HoldSystem::new();
        hs.hold(TetrominoKind::T);
        hs.allow_hold(); // 새 미노 스폰 시뮬레이션

        let result = hs.hold(TetrominoKind::I);
        assert_eq!(result, HoldResult::Swapped(TetrominoKind::T));
        assert_eq!(hs.held_piece(), Some(TetrominoKind::I));
    }

    #[test]
    fn test_hold_not_allowed_twice() {
        let mut hs = HoldSystem::new();
        hs.hold(TetrominoKind::T);

        let result = hs.hold(TetrominoKind::I);
        assert_eq!(result, HoldResult::NotAllowed);
        assert_eq!(hs.held_piece(), Some(TetrominoKind::T)); // 변경 없음
    }

    #[test]
    fn test_hold_allow_after_spawn() {
        let mut hs = HoldSystem::new();
        hs.hold(TetrominoKind::T);
        assert!(!hs.can_hold());

        hs.allow_hold();
        assert!(hs.can_hold());
    }

    #[test]
    fn test_hold_reset() {
        let mut hs = HoldSystem::new();
        hs.hold(TetrominoKind::T);

        hs.reset();
        assert!(hs.held_piece().is_none());
        assert!(hs.can_hold());
    }

    // ------------------------------------------------------------------------
    // 고스트 피스 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_ghost_position_empty_field() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(3, 20);

        let ghost_y = calculate_ghost_position(&tetromino, &playfield);
        // T 미노: row 1의 abs_y = y + 2, 바닥(y=0)에 닿으려면 y = -2
        assert_eq!(ghost_y, -2);
    }

    #[test]
    fn test_ghost_position_with_blocks() {
        let mut playfield = Playfield::new();
        use crate::game::playfield::{Cell, CellColor};
        // y=5 행에 블록 배치
        for x in 0..10 {
            playfield.set(x, 5, Cell::Filled(CellColor::Cyan));
        }

        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(3, 20);

        let ghost_y = calculate_ghost_position(&tetromino, &playfield);
        // T 미노 하단(row 1, abs_y=y+2)이 y=6에 위치해야 함 → y=4
        assert_eq!(ghost_y, 4);
    }

    #[test]
    fn test_ghost_position_already_at_bottom() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(3, -2); // 이미 바닥에 위치

        let ghost_y = calculate_ghost_position(&tetromino, &playfield);
        assert_eq!(ghost_y, -2);
    }

    // ------------------------------------------------------------------------
    // 게임 오버 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_spawn_collision_empty_field() {
        let playfield = Playfield::new();
        let tetromino = Tetromino::new(TetrominoKind::T);

        assert!(!check_spawn_collision(&tetromino, &playfield));
    }

    #[test]
    fn test_spawn_collision_with_blocks() {
        let mut playfield = Playfield::new();
        use crate::game::playfield::{Cell, CellColor};
        // 스폰 위치(y=20~21)에 블록 배치
        playfield.set(4, 20, Cell::Filled(CellColor::Cyan));

        let tetromino = Tetromino::new(TetrominoKind::T);
        assert!(check_spawn_collision(&tetromino, &playfield));
    }

    #[test]
    fn test_lock_out_above_visible() {
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        // 가시 영역(20) 위에 완전히 위치
        tetromino.set_position(3, 21);

        // T 미노의 블록들이 y=21+2, y=21+3 등에 위치 (모두 >= 20)
        assert!(check_lock_out(&tetromino));
    }

    #[test]
    fn test_lock_out_partial_visible() {
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        // T 미노: row 1의 abs_y = y + 2, row 0의 abs_y = y + 3
        // y=17일 때: row 1은 y=19 (가시 영역 내), row 0은 y=20 (가시 영역 밖)
        tetromino.set_position(3, 17);

        assert!(!check_lock_out(&tetromino));
    }

    #[test]
    fn test_game_over_spawn() {
        let mut playfield = Playfield::new();
        use crate::game::playfield::{Cell, CellColor};
        playfield.set(4, 20, Cell::Filled(CellColor::Cyan));

        let tetromino = Tetromino::new(TetrominoKind::T);
        let result = check_game_over(&tetromino, &playfield, true);

        assert_eq!(result, Some(GameOverReason::SpawnOverlap));
    }

    #[test]
    fn test_game_over_lock_out() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(3, 21);

        let result = check_game_over(&tetromino, &playfield, false);
        assert_eq!(result, Some(GameOverReason::LockOutAboveVisible));
    }

    #[test]
    fn test_no_game_over() {
        let playfield = Playfield::new();
        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(3, 10);

        let result = check_game_over(&tetromino, &playfield, false);
        assert!(result.is_none());
    }
}
