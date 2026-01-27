//! 렌더링 상태 모듈
//!
//! 게임 상태를 렌더링에 전달하기 위한 구조체

use crate::game::{Playfield, Tetromino, TetrominoKind};

/// 설정 화면 상태
#[derive(Debug, Clone, Default)]
pub struct SettingsState {
    /// 설정 화면 활성화 여부
    pub active: bool,
    /// 현재 선택된 항목 (0: DAS, 1: ARR)
    pub selected_item: u8,
    /// 현재 DAS 값 (ms)
    pub das_ms: u64,
    /// 현재 ARR 값 (ms)
    pub arr_ms: u64,
}

/// 렌더링에 필요한 게임 상태
#[derive(Debug, Clone)]
pub struct RenderState {
    /// 플레이필드
    pub playfield: Playfield,
    /// 현재 테트로미노 (없으면 None)
    pub current_piece: Option<Tetromino>,
    /// 홀드된 미노 종류
    pub hold_piece: Option<TetrominoKind>,
    /// 홀드 사용 가능 여부
    pub can_hold: bool,
    /// Next 프리뷰 (최대 5개)
    pub next_pieces: Vec<TetrominoKind>,
    /// 현재 점수
    pub score: u32,
    /// 현재 레벨
    pub level: u32,
    /// 클리어한 라인 수
    pub lines: u32,
    /// 게임 오버 여부
    pub game_over: bool,
    /// 일시정지 여부
    pub paused: bool,
    /// 설정 화면 상태
    pub settings: SettingsState,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            playfield: Playfield::new(),
            current_piece: None,
            hold_piece: None,
            can_hold: true,
            next_pieces: Vec::new(),
            score: 0,
            level: 1,
            lines: 0,
            game_over: false,
            paused: false,
            settings: SettingsState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_state_default() {
        let state = RenderState::default();
        assert!(state.current_piece.is_none());
        assert!(state.hold_piece.is_none());
        assert!(state.can_hold);
        assert!(state.next_pieces.is_empty());
        assert_eq!(state.score, 0);
        assert_eq!(state.level, 1);
        assert_eq!(state.lines, 0);
        assert!(!state.game_over);
        assert!(!state.paused);
        assert!(!state.settings.active);
    }
}
