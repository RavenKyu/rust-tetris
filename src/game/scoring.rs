//! 점수 시스템 모듈
//!
//! 테트리스 점수 계산을 담당합니다:
//! - 기본 점수 (Single, Double, Triple, Tetris)
//! - T-Spin 판정 및 점수
//! - Back-to-Back 보너스
//! - 콤보 보너스
//! - Perfect Clear 보너스
//! - Soft/Hard Drop 점수

use super::{Playfield, RotationState, Tetromino, TetrominoKind};

// ============================================================================
// 점수 상수
// ============================================================================

/// 기본 라인 클리어 점수
const SINGLE_SCORE: u32 = 100;
const DOUBLE_SCORE: u32 = 300;
const TRIPLE_SCORE: u32 = 500;
const TETRIS_SCORE: u32 = 800;

/// T-Spin 점수
const TSPIN_MINI_SCORE: u32 = 100;
const TSPIN_SCORE: u32 = 400;
const TSPIN_MINI_SINGLE_SCORE: u32 = 200;
const TSPIN_SINGLE_SCORE: u32 = 800;
const TSPIN_DOUBLE_SCORE: u32 = 1200;
const TSPIN_TRIPLE_SCORE: u32 = 1600;

/// 보너스
const COMBO_BONUS: u32 = 50;
const PERFECT_CLEAR_BONUS: u32 = 3000;

/// Back-to-Back 배수 (1.5x = 3/2)
const B2B_NUMERATOR: u32 = 3;
const B2B_DENOMINATOR: u32 = 2;

// ============================================================================
// 열거형 정의
// ============================================================================

/// 라인 클리어 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineClearType {
    /// 1줄 클리어
    Single,
    /// 2줄 클리어
    Double,
    /// 3줄 클리어
    Triple,
    /// 4줄 클리어
    Tetris,
}

impl LineClearType {
    /// 클리어한 줄 수로부터 생성
    #[must_use]
    pub const fn from_lines(lines: u32) -> Option<Self> {
        match lines {
            1 => Some(Self::Single),
            2 => Some(Self::Double),
            3 => Some(Self::Triple),
            4 => Some(Self::Tetris),
            _ => None,
        }
    }

    /// 기본 점수 반환
    #[must_use]
    pub const fn base_score(self) -> u32 {
        match self {
            Self::Single => SINGLE_SCORE,
            Self::Double => DOUBLE_SCORE,
            Self::Triple => TRIPLE_SCORE,
            Self::Tetris => TETRIS_SCORE,
        }
    }

    /// 줄 수 반환
    #[must_use]
    pub const fn lines(self) -> u32 {
        match self {
            Self::Single => 1,
            Self::Double => 2,
            Self::Triple => 3,
            Self::Tetris => 4,
        }
    }

    /// Back-to-Back 대상인지 여부
    #[must_use]
    pub const fn is_difficult(self) -> bool {
        matches!(self, Self::Tetris)
    }
}

/// T-Spin 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TSpinType {
    /// T-Spin 아님
    #[default]
    None,
    /// T-Spin Mini
    Mini,
    /// T-Spin (Full)
    Full,
}

impl TSpinType {
    /// T-Spin인지 여부 (Mini 포함)
    #[must_use]
    pub const fn is_tspin(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// 점수 이벤트
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreEvent {
    /// 라인 클리어
    LineClear {
        clear_type: LineClearType,
        tspin: TSpinType,
    },
    /// Soft Drop (셀 수)
    SoftDrop(u32),
    /// Hard Drop (셀 수)
    HardDrop(u32),
    /// Perfect Clear
    PerfectClear,
}

/// 점수 계산 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreResult {
    /// 획득 점수
    pub score: u32,
    /// 클리어한 줄 수
    pub lines: u32,
    /// Back-to-Back 적용 여부
    pub is_back_to_back: bool,
    /// 콤보 수
    pub combo: u32,
}

// ============================================================================
// 점수 시스템
// ============================================================================

/// 점수 시스템
#[derive(Debug, Clone)]
pub struct ScoringSystem {
    /// 현재 점수
    score: u32,
    /// 현재 레벨
    level: u32,
    /// 클리어한 총 라인 수
    lines: u32,
    /// 현재 콤보 수 (연속 라인 클리어)
    combo: u32,
    /// Back-to-Back 활성 여부
    back_to_back: bool,
}

impl ScoringSystem {
    /// 새 점수 시스템 생성
    #[must_use]
    pub const fn new(starting_level: u32) -> Self {
        Self {
            score: 0,
            level: starting_level,
            lines: 0,
            combo: 0,
            back_to_back: false,
        }
    }

    /// 현재 점수 반환
    #[must_use]
    pub const fn score(&self) -> u32 {
        self.score
    }

    /// 현재 레벨 반환
    #[must_use]
    pub const fn level(&self) -> u32 {
        self.level
    }

    /// 클리어한 총 라인 수 반환
    #[must_use]
    pub const fn lines(&self) -> u32 {
        self.lines
    }

    /// 현재 콤보 수 반환
    #[must_use]
    pub const fn combo(&self) -> u32 {
        self.combo
    }

    /// Back-to-Back 활성 여부 반환
    #[must_use]
    pub const fn is_back_to_back(&self) -> bool {
        self.back_to_back
    }

    /// 점수 이벤트 처리
    pub fn process_event(&mut self, event: ScoreEvent) -> ScoreResult {
        match event {
            ScoreEvent::LineClear { clear_type, tspin } => {
                self.process_line_clear(clear_type, tspin)
            }
            ScoreEvent::SoftDrop(cells) => {
                let score = cells;
                self.score += score;
                ScoreResult {
                    score,
                    lines: 0,
                    is_back_to_back: false,
                    combo: 0,
                }
            }
            ScoreEvent::HardDrop(cells) => {
                let score = cells * 2;
                self.score += score;
                ScoreResult {
                    score,
                    lines: 0,
                    is_back_to_back: false,
                    combo: 0,
                }
            }
            ScoreEvent::PerfectClear => {
                self.score += PERFECT_CLEAR_BONUS;
                ScoreResult {
                    score: PERFECT_CLEAR_BONUS,
                    lines: 0,
                    is_back_to_back: false,
                    combo: 0,
                }
            }
        }
    }

    /// 라인 클리어 처리
    fn process_line_clear(&mut self, clear_type: LineClearType, tspin: TSpinType) -> ScoreResult {
        let lines_cleared = clear_type.lines();

        // 기본 점수 계산
        let base_score = Self::calculate_base_score(clear_type, tspin);

        // 이 클리어가 "어려운" 클리어인지 확인 (Tetris 또는 T-Spin)
        let is_difficult = clear_type.is_difficult() || tspin.is_tspin();

        // Back-to-Back 적용 여부
        let apply_b2b = self.back_to_back && is_difficult;

        // 점수에 레벨 배수 적용
        let mut total_score = base_score * self.level;

        // Back-to-Back 보너스 적용 (1.5x)
        if apply_b2b {
            total_score = total_score * B2B_NUMERATOR / B2B_DENOMINATOR;
        }

        // 콤보 보너스 추가
        let combo_bonus = self.combo * COMBO_BONUS * self.level;
        total_score += combo_bonus;

        // 상태 업데이트
        self.score += total_score;
        self.lines += lines_cleared;
        self.combo += 1;

        // Back-to-Back 상태 업데이트
        // 어려운 클리어면 B2B 활성화, 아니면 해제
        self.back_to_back = is_difficult;

        // 레벨업 체크 (10줄당 1레벨)
        self.update_level();

        ScoreResult {
            score: total_score,
            lines: lines_cleared,
            is_back_to_back: apply_b2b,
            combo: self.combo,
        }
    }

    /// 기본 점수 계산 (T-Spin 포함)
    const fn calculate_base_score(clear_type: LineClearType, tspin: TSpinType) -> u32 {
        match tspin {
            TSpinType::None => clear_type.base_score(),
            TSpinType::Mini => match clear_type.lines() {
                0 => TSPIN_MINI_SCORE,
                1 => TSPIN_MINI_SINGLE_SCORE,
                _ => clear_type.base_score(), // Mini는 Single까지만
            },
            TSpinType::Full => match clear_type.lines() {
                0 => TSPIN_SCORE,
                1 => TSPIN_SINGLE_SCORE,
                2 => TSPIN_DOUBLE_SCORE,
                3 => TSPIN_TRIPLE_SCORE,
                _ => clear_type.base_score(),
            },
        }
    }

    /// 레벨 업데이트 (10줄당 1레벨)
    fn update_level(&mut self) {
        let target_level = 1 + self.lines / 10;
        if target_level > self.level {
            self.level = target_level;
        }
    }

    /// 콤보 리셋 (라인 클리어 없이 미노 잠금 시)
    pub fn reset_combo(&mut self) {
        self.combo = 0;
    }

    /// 전체 리셋 (게임 재시작)
    pub fn reset(&mut self, starting_level: u32) {
        self.score = 0;
        self.level = starting_level;
        self.lines = 0;
        self.combo = 0;
        self.back_to_back = false;
    }
}

impl Default for ScoringSystem {
    fn default() -> Self {
        Self::new(1)
    }
}

// ============================================================================
// T-Spin 판정
// ============================================================================

/// T-Spin 판정 (3-corner 방식)
///
/// T 미노의 4 코너 중 3개 이상이 벽이나 블록으로 막혀있으면 T-Spin
/// - 마지막 동작이 회전이어야 함
/// - Mini: 마지막 킥이 (0,0)인 경우 (킥 없이 회전)
/// - Full: 마지막 킥이 (0,0)이 아닌 경우 또는 특정 조건
#[must_use]
pub fn detect_tspin(
    tetromino: &Tetromino,
    playfield: &Playfield,
    rotation_state: RotationState,
    last_kick: (i32, i32),
    was_last_move_rotation: bool,
) -> TSpinType {
    // T 미노가 아니면 T-Spin 아님
    if tetromino.kind != TetrominoKind::T {
        return TSpinType::None;
    }

    // 마지막 동작이 회전이 아니면 T-Spin 아님
    if !was_last_move_rotation {
        return TSpinType::None;
    }

    // T 미노 중심 좌표 계산
    // T 미노는 3x3 영역에서 중앙이 (1,1)
    // 4x4 매트릭스에서 T 미노 중심은 (x+1, y+2)
    let center_x = tetromino.x + 1;
    let center_y = tetromino.y + 2;

    // 4 코너 좌표 (상대적)
    let corners = [
        (center_x - 1, center_y + 1), // 좌상
        (center_x + 1, center_y + 1), // 우상
        (center_x - 1, center_y - 1), // 좌하
        (center_x + 1, center_y - 1), // 우하
    ];

    // 코너가 막혀있는지 확인
    let blocked_corners: Vec<bool> = corners
        .iter()
        .map(|&(x, y)| is_blocked(x, y, playfield))
        .collect();

    let blocked_count = blocked_corners.iter().filter(|&&b| b).count();

    // 3개 이상 막혀있으면 T-Spin
    if blocked_count < 3 {
        return TSpinType::None;
    }

    // T-Spin Mini vs Full 판정
    // 회전 상태에 따른 "앞쪽" 코너 결정
    let front_corners_blocked = match rotation_state {
        RotationState::Spawn => blocked_corners[0] && blocked_corners[1], // 상단
        RotationState::Right => blocked_corners[1] && blocked_corners[3], // 우측
        RotationState::Two => blocked_corners[2] && blocked_corners[3],   // 하단
        RotationState::Left => blocked_corners[0] && blocked_corners[2],  // 좌측
    };

    // 앞쪽 2개 코너가 모두 막혀있거나 킥을 사용한 경우 Full T-Spin
    if front_corners_blocked || last_kick != (0, 0) {
        TSpinType::Full
    } else {
        TSpinType::Mini
    }
}

/// 좌표가 막혀있는지 확인 (벽 또는 블록)
fn is_blocked(x: i32, y: i32, playfield: &Playfield) -> bool {
    // 범위 밖은 막힌 것으로 처리
    if !(0..10).contains(&x) || !(0..40).contains(&y) {
        return true;
    }

    playfield.is_filled_at(x as usize, y as usize)
}

/// Perfect Clear 판정
/// 플레이필드가 완전히 비어있으면 Perfect Clear
#[must_use]
pub fn is_perfect_clear(playfield: &Playfield) -> bool {
    playfield.is_empty()
}

// ============================================================================
// 테스트
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::playfield::{Cell, CellColor};

    // ------------------------------------------------------------------------
    // LineClearType 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_line_clear_type_from_lines() {
        assert_eq!(LineClearType::from_lines(1), Some(LineClearType::Single));
        assert_eq!(LineClearType::from_lines(2), Some(LineClearType::Double));
        assert_eq!(LineClearType::from_lines(3), Some(LineClearType::Triple));
        assert_eq!(LineClearType::from_lines(4), Some(LineClearType::Tetris));
        assert_eq!(LineClearType::from_lines(0), None);
        assert_eq!(LineClearType::from_lines(5), None);
    }

    #[test]
    fn test_line_clear_base_scores() {
        assert_eq!(LineClearType::Single.base_score(), 100);
        assert_eq!(LineClearType::Double.base_score(), 300);
        assert_eq!(LineClearType::Triple.base_score(), 500);
        assert_eq!(LineClearType::Tetris.base_score(), 800);
    }

    #[test]
    fn test_line_clear_is_difficult() {
        assert!(!LineClearType::Single.is_difficult());
        assert!(!LineClearType::Double.is_difficult());
        assert!(!LineClearType::Triple.is_difficult());
        assert!(LineClearType::Tetris.is_difficult());
    }

    // ------------------------------------------------------------------------
    // TSpinType 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_tspin_type_is_tspin() {
        assert!(!TSpinType::None.is_tspin());
        assert!(TSpinType::Mini.is_tspin());
        assert!(TSpinType::Full.is_tspin());
    }

    // ------------------------------------------------------------------------
    // ScoringSystem 기본 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_scoring_system_new() {
        let ss = ScoringSystem::new(5);
        assert_eq!(ss.score(), 0);
        assert_eq!(ss.level(), 5);
        assert_eq!(ss.lines(), 0);
        assert_eq!(ss.combo(), 0);
        assert!(!ss.is_back_to_back());
    }

    #[test]
    fn test_scoring_system_default() {
        let ss = ScoringSystem::default();
        assert_eq!(ss.level(), 1);
    }

    // ------------------------------------------------------------------------
    // 기본 점수 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_single_score() {
        let mut ss = ScoringSystem::new(1);
        let result = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Single,
            tspin: TSpinType::None,
        });
        assert_eq!(result.score, 100);
        assert_eq!(result.lines, 1);
    }

    #[test]
    fn test_double_score() {
        let mut ss = ScoringSystem::new(1);
        let result = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Double,
            tspin: TSpinType::None,
        });
        assert_eq!(result.score, 300);
        assert_eq!(result.lines, 2);
    }

    #[test]
    fn test_triple_score() {
        let mut ss = ScoringSystem::new(1);
        let result = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Triple,
            tspin: TSpinType::None,
        });
        assert_eq!(result.score, 500);
        assert_eq!(result.lines, 3);
    }

    #[test]
    fn test_tetris_score() {
        let mut ss = ScoringSystem::new(1);
        let result = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Tetris,
            tspin: TSpinType::None,
        });
        assert_eq!(result.score, 800);
        assert_eq!(result.lines, 4);
    }

    #[test]
    fn test_level_multiplier() {
        let mut ss = ScoringSystem::new(5);
        let result = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Single,
            tspin: TSpinType::None,
        });
        // 100 * 5 = 500
        assert_eq!(result.score, 500);
    }

    // ------------------------------------------------------------------------
    // T-Spin 점수 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_tspin_mini_single_score() {
        let mut ss = ScoringSystem::new(1);
        let result = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Single,
            tspin: TSpinType::Mini,
        });
        assert_eq!(result.score, 200);
    }

    #[test]
    fn test_tspin_single_score() {
        let mut ss = ScoringSystem::new(1);
        let result = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Single,
            tspin: TSpinType::Full,
        });
        assert_eq!(result.score, 800);
    }

    #[test]
    fn test_tspin_double_score() {
        let mut ss = ScoringSystem::new(1);
        let result = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Double,
            tspin: TSpinType::Full,
        });
        assert_eq!(result.score, 1200);
    }

    #[test]
    fn test_tspin_triple_score() {
        let mut ss = ScoringSystem::new(1);
        let result = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Triple,
            tspin: TSpinType::Full,
        });
        assert_eq!(result.score, 1600);
    }

    // ------------------------------------------------------------------------
    // Back-to-Back 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_back_to_back_tetris() {
        let mut ss = ScoringSystem::new(1);

        // 첫 번째 Tetris
        let result1 = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Tetris,
            tspin: TSpinType::None,
        });
        assert_eq!(result1.score, 800);
        assert!(!result1.is_back_to_back);
        assert!(ss.is_back_to_back());

        // 두 번째 Tetris (B2B)
        let result2 = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Tetris,
            tspin: TSpinType::None,
        });
        // 800 * 1.5 = 1200
        assert_eq!(result2.score, 1200 + 50); // + combo bonus
        assert!(result2.is_back_to_back);
    }

    #[test]
    fn test_back_to_back_tspin() {
        let mut ss = ScoringSystem::new(1);

        // T-Spin Single
        ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Single,
            tspin: TSpinType::Full,
        });
        assert!(ss.is_back_to_back());

        // T-Spin Double (B2B)
        let result = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Double,
            tspin: TSpinType::Full,
        });
        // 1200 * 1.5 = 1800 + combo
        assert_eq!(result.score, 1800 + 50);
        assert!(result.is_back_to_back);
    }

    #[test]
    fn test_back_to_back_broken_by_single() {
        let mut ss = ScoringSystem::new(1);

        // Tetris
        ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Tetris,
            tspin: TSpinType::None,
        });
        assert!(ss.is_back_to_back());

        // Single (B2B 해제)
        ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Single,
            tspin: TSpinType::None,
        });
        assert!(!ss.is_back_to_back());
    }

    // ------------------------------------------------------------------------
    // 콤보 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_combo_bonus() {
        let mut ss = ScoringSystem::new(1);

        // 첫 번째 클리어: 콤보 0
        let result1 = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Single,
            tspin: TSpinType::None,
        });
        assert_eq!(result1.combo, 1);
        assert_eq!(result1.score, 100); // 콤보 보너스 없음

        // 두 번째 클리어: 콤보 1
        let result2 = ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Single,
            tspin: TSpinType::None,
        });
        assert_eq!(result2.combo, 2);
        assert_eq!(result2.score, 100 + 50); // +50 콤보 보너스
    }

    #[test]
    fn test_combo_reset() {
        let mut ss = ScoringSystem::new(1);

        ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Single,
            tspin: TSpinType::None,
        });
        assert_eq!(ss.combo(), 1);

        ss.reset_combo();
        assert_eq!(ss.combo(), 0);
    }

    // ------------------------------------------------------------------------
    // Drop 점수 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_soft_drop_score() {
        let mut ss = ScoringSystem::new(1);
        let result = ss.process_event(ScoreEvent::SoftDrop(10));
        assert_eq!(result.score, 10);
        assert_eq!(ss.score(), 10);
    }

    #[test]
    fn test_hard_drop_score() {
        let mut ss = ScoringSystem::new(1);
        let result = ss.process_event(ScoreEvent::HardDrop(10));
        assert_eq!(result.score, 20);
        assert_eq!(ss.score(), 20);
    }

    // ------------------------------------------------------------------------
    // Perfect Clear 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_perfect_clear_bonus() {
        let mut ss = ScoringSystem::new(1);
        let result = ss.process_event(ScoreEvent::PerfectClear);
        assert_eq!(result.score, 3000);
        assert_eq!(ss.score(), 3000);
    }

    #[test]
    fn test_is_perfect_clear() {
        let playfield = Playfield::new();
        assert!(is_perfect_clear(&playfield));
    }

    #[test]
    fn test_is_not_perfect_clear() {
        let mut playfield = Playfield::new();
        playfield.set(0, 0, Cell::Filled(CellColor::Cyan));
        assert!(!is_perfect_clear(&playfield));
    }

    // ------------------------------------------------------------------------
    // 레벨업 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_level_up() {
        let mut ss = ScoringSystem::new(1);

        // 10줄 클리어하면 레벨 2
        for _ in 0..10 {
            ss.process_event(ScoreEvent::LineClear {
                clear_type: LineClearType::Single,
                tspin: TSpinType::None,
            });
        }
        assert_eq!(ss.level(), 2);
        assert_eq!(ss.lines(), 10);
    }

    #[test]
    fn test_level_up_with_tetris() {
        let mut ss = ScoringSystem::new(1);

        // 테트리스 3번 = 12줄 → 레벨 2
        for _ in 0..3 {
            ss.process_event(ScoreEvent::LineClear {
                clear_type: LineClearType::Tetris,
                tspin: TSpinType::None,
            });
        }
        assert_eq!(ss.lines(), 12);
        assert_eq!(ss.level(), 2);
    }

    // ------------------------------------------------------------------------
    // 리셋 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_reset() {
        let mut ss = ScoringSystem::new(1);
        ss.process_event(ScoreEvent::LineClear {
            clear_type: LineClearType::Tetris,
            tspin: TSpinType::None,
        });

        ss.reset(3);
        assert_eq!(ss.score(), 0);
        assert_eq!(ss.level(), 3);
        assert_eq!(ss.lines(), 0);
        assert_eq!(ss.combo(), 0);
        assert!(!ss.is_back_to_back());
    }

    // ------------------------------------------------------------------------
    // T-Spin 판정 테스트
    // ------------------------------------------------------------------------

    #[test]
    fn test_detect_tspin_not_t_mino() {
        let playfield = Playfield::new();
        let tetromino = Tetromino::new(TetrominoKind::I);
        let result = detect_tspin(&tetromino, &playfield, RotationState::Spawn, (0, 0), true);
        assert_eq!(result, TSpinType::None);
    }

    #[test]
    fn test_detect_tspin_not_rotation() {
        let playfield = Playfield::new();
        let tetromino = Tetromino::new(TetrominoKind::T);
        let result = detect_tspin(
            &tetromino,
            &playfield,
            RotationState::Spawn,
            (0, 0),
            false, // 마지막 동작이 회전 아님
        );
        assert_eq!(result, TSpinType::None);
    }

    #[test]
    fn test_detect_tspin_in_corner() {
        let mut playfield = Playfield::new();

        // T 미노를 코너에 끼워넣는 상황 시뮬레이션
        // T 미노 중심 좌표 = (x+1, y+2)
        // 위치 (0, -1)일 때 중심 = (1, 1)
        // 코너: (0,2), (2,2), (0,0), (2,0)

        // 3개 이상 코너 막기 위해 블록 배치
        playfield.set(0, 2, Cell::Filled(CellColor::Cyan)); // 좌상
        playfield.set(2, 2, Cell::Filled(CellColor::Cyan)); // 우상
        playfield.set(2, 0, Cell::Filled(CellColor::Cyan)); // 우하

        let mut tetromino = Tetromino::new(TetrominoKind::T);
        tetromino.set_position(0, -1);

        let result = detect_tspin(
            &tetromino,
            &playfield,
            RotationState::Spawn,
            (1, 0), // 킥 사용
            true,
        );

        // 3개 코너가 막혀있고 킥을 사용했으므로 Full T-Spin
        assert_eq!(result, TSpinType::Full);
    }
}
