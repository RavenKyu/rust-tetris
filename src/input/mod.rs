//! 입력 처리 모듈
//!
//! 키보드 입력 처리와 DAS/ARR 시스템을 구현합니다.
//!
//! ## DAS/ARR 설명
//! - **DAS (Delayed Auto Shift)**: 키를 누르고 있을 때 자동 반복 시작까지의 지연
//! - **ARR (Auto Repeat Rate)**: 자동 반복 간격 (0 = 즉시 끝까지 이동)

use std::collections::HashMap;
use std::time::Duration;

/// 기본 DAS 값 (밀리초)
pub const DEFAULT_DAS_MS: u64 = 167;

/// 기본 ARR 값 (밀리초)
pub const DEFAULT_ARR_MS: u64 = 50;

/// 게임 액션 (입력에 의해 트리거되는 동작)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameAction {
    /// 좌측 이동
    MoveLeft,
    /// 우측 이동
    MoveRight,
    /// Soft Drop (아래로 빠르게)
    SoftDrop,
    /// Hard Drop (즉시 착지)
    HardDrop,
    /// 시계방향 회전
    RotateCW,
    /// 반시계방향 회전
    RotateCCW,
    /// 180도 회전
    Rotate180,
    /// 홀드
    Hold,
    /// 일시정지
    Pause,
    /// 재시작
    Restart,
}

/// 가상 키코드 (SDL3 키코드와 매핑)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtualKey {
    Left,
    Right,
    Up,
    Down,
    Space,
    Z,
    X,
    A,
    C,
    R,
    Escape,
    LShift,
    RShift,
}

/// 키 바인딩 설정
#[derive(Debug, Clone)]
pub struct KeyBindings {
    bindings: HashMap<VirtualKey, GameAction>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyBindings {
    /// 기본 키 바인딩으로 생성
    #[must_use]
    pub fn new() -> Self {
        let mut bindings = HashMap::new();

        // 이동
        bindings.insert(VirtualKey::Left, GameAction::MoveLeft);
        bindings.insert(VirtualKey::Right, GameAction::MoveRight);
        bindings.insert(VirtualKey::Down, GameAction::SoftDrop);
        bindings.insert(VirtualKey::Space, GameAction::HardDrop);

        // 회전
        bindings.insert(VirtualKey::Up, GameAction::RotateCW);
        bindings.insert(VirtualKey::X, GameAction::RotateCW);
        bindings.insert(VirtualKey::Z, GameAction::RotateCCW);
        bindings.insert(VirtualKey::A, GameAction::Rotate180);

        // 홀드
        bindings.insert(VirtualKey::C, GameAction::Hold);
        bindings.insert(VirtualKey::LShift, GameAction::Hold);
        bindings.insert(VirtualKey::RShift, GameAction::Hold);

        // 시스템
        bindings.insert(VirtualKey::Escape, GameAction::Pause);
        bindings.insert(VirtualKey::R, GameAction::Restart);

        Self { bindings }
    }

    /// 키에 대한 액션 조회
    #[must_use]
    pub fn get_action(&self, key: VirtualKey) -> Option<GameAction> {
        self.bindings.get(&key).copied()
    }

    /// 키 바인딩 설정
    pub fn set_binding(&mut self, key: VirtualKey, action: GameAction) {
        self.bindings.insert(key, action);
    }

    /// 키 바인딩 제거
    pub fn remove_binding(&mut self, key: VirtualKey) {
        self.bindings.remove(&key);
    }
}

/// 단일 키의 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyState {
    /// 눌리지 않음
    Released,
    /// 막 눌림 (이번 프레임)
    JustPressed,
    /// 누르고 있음
    Held,
}

/// DAS/ARR 설정
#[derive(Debug, Clone, Copy)]
pub struct DasArrConfig {
    /// DAS (Delayed Auto Shift) - 자동 반복 시작까지 지연
    pub das: Duration,
    /// ARR (Auto Repeat Rate) - 자동 반복 간격
    pub arr: Duration,
}

impl Default for DasArrConfig {
    fn default() -> Self {
        Self {
            das: Duration::from_millis(DEFAULT_DAS_MS),
            arr: Duration::from_millis(DEFAULT_ARR_MS),
        }
    }
}

impl DasArrConfig {
    /// 새로운 DAS/ARR 설정 생성
    #[must_use]
    pub fn new(das_ms: u64, arr_ms: u64) -> Self {
        Self {
            das: Duration::from_millis(das_ms),
            arr: Duration::from_millis(arr_ms),
        }
    }
}

/// 방향 키 DAS 상태
#[derive(Debug, Clone)]
struct DasState {
    /// 키가 눌린 상태인지
    pressed: bool,
    /// 키가 눌린 후 경과 시간
    hold_time: Duration,
    /// DAS가 발동되었는지 (초기 지연 완료)
    das_charged: bool,
    /// ARR 누적 시간 (DAS 발동 후)
    arr_accumulator: Duration,
}

impl Default for DasState {
    fn default() -> Self {
        Self {
            pressed: false,
            hold_time: Duration::ZERO,
            das_charged: false,
            arr_accumulator: Duration::ZERO,
        }
    }
}

impl DasState {
    fn press(&mut self) {
        if !self.pressed {
            self.pressed = true;
            self.hold_time = Duration::ZERO;
            self.das_charged = false;
            self.arr_accumulator = Duration::ZERO;
        }
    }

    fn release(&mut self) {
        self.pressed = false;
        self.hold_time = Duration::ZERO;
        self.das_charged = false;
        self.arr_accumulator = Duration::ZERO;
    }

    /// DAS/ARR 업데이트하고 이동 횟수 반환
    fn update(&mut self, delta: Duration, config: &DasArrConfig) -> u32 {
        if !self.pressed {
            return 0;
        }

        self.hold_time += delta;

        // DAS가 아직 충전되지 않은 경우
        if !self.das_charged {
            if self.hold_time >= config.das {
                self.das_charged = true;
                self.arr_accumulator = self.hold_time - config.das;
                // DAS 충전 완료 시 1회 이동
                if config.arr.is_zero() {
                    // ARR이 0이면 무한대 이동 (실제로는 큰 값)
                    return u32::MAX;
                }
                return 1 + self.consume_arr(config);
            }
            return 0;
        }

        // DAS 충전 완료 후 ARR 처리
        self.arr_accumulator += delta;
        self.consume_arr(config)
    }

    fn consume_arr(&mut self, config: &DasArrConfig) -> u32 {
        if config.arr.is_zero() {
            // ARR이 0이면 즉시 끝까지 (무한대)
            return u32::MAX;
        }

        let moves = (self.arr_accumulator.as_nanos() / config.arr.as_nanos()) as u32;
        self.arr_accumulator =
            Duration::from_nanos((self.arr_accumulator.as_nanos() % config.arr.as_nanos()) as u64);
        moves
    }
}

/// 입력 핸들러
///
/// 키 입력을 받아 게임 액션으로 변환하고,
/// DAS/ARR을 적용하여 이동 명령을 생성합니다.
#[derive(Debug, Clone)]
pub struct InputHandler {
    /// 키 바인딩
    bindings: KeyBindings,
    /// DAS/ARR 설정
    das_arr: DasArrConfig,
    /// 각 키의 현재 상태
    key_states: HashMap<VirtualKey, KeyState>,
    /// 좌측 이동 DAS 상태
    das_left: DasState,
    /// 우측 이동 DAS 상태
    das_right: DasState,
    /// Soft Drop DAS 상태
    das_down: DasState,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHandler {
    /// 새로운 입력 핸들러 생성
    #[must_use]
    pub fn new() -> Self {
        Self {
            bindings: KeyBindings::new(),
            das_arr: DasArrConfig::default(),
            key_states: HashMap::new(),
            das_left: DasState::default(),
            das_right: DasState::default(),
            das_down: DasState::default(),
        }
    }

    /// DAS/ARR 설정으로 생성
    #[must_use]
    pub fn with_das_arr(das_arr: DasArrConfig) -> Self {
        Self {
            das_arr,
            ..Self::new()
        }
    }

    /// 키 바인딩 참조
    #[must_use]
    pub fn bindings(&self) -> &KeyBindings {
        &self.bindings
    }

    /// 키 바인딩 가변 참조
    pub fn bindings_mut(&mut self) -> &mut KeyBindings {
        &mut self.bindings
    }

    /// DAS/ARR 설정
    pub fn set_das_arr(&mut self, config: DasArrConfig) {
        self.das_arr = config;
    }

    /// DAS/ARR 설정 조회
    #[must_use]
    pub fn das_arr(&self) -> &DasArrConfig {
        &self.das_arr
    }

    /// 키 누름 이벤트 처리
    pub fn key_down(&mut self, key: VirtualKey) {
        let state = self.key_states.entry(key).or_insert(KeyState::Released);
        if *state == KeyState::Released {
            *state = KeyState::JustPressed;
        }

        // DAS 상태 업데이트
        if let Some(action) = self.bindings.get_action(key) {
            match action {
                GameAction::MoveLeft => self.das_left.press(),
                GameAction::MoveRight => self.das_right.press(),
                GameAction::SoftDrop => self.das_down.press(),
                _ => {}
            }
        }
    }

    /// 키 뗌 이벤트 처리
    pub fn key_up(&mut self, key: VirtualKey) {
        self.key_states.insert(key, KeyState::Released);

        // DAS 상태 업데이트
        if let Some(action) = self.bindings.get_action(key) {
            match action {
                GameAction::MoveLeft => self.das_left.release(),
                GameAction::MoveRight => self.das_right.release(),
                GameAction::SoftDrop => self.das_down.release(),
                _ => {}
            }
        }
    }

    /// 프레임 업데이트 (delta 시간 경과)
    /// 반환: 이번 프레임에 발생한 액션 목록
    pub fn update(&mut self, delta: Duration) -> Vec<InputEvent> {
        let mut events = Vec::new();

        // 즉시 트리거 액션 (JustPressed 상태인 키)
        for (&key, &state) in &self.key_states {
            if state == KeyState::JustPressed
                && let Some(action) = self.bindings.get_action(key)
            {
                // 모든 액션은 첫 입력 시 즉시 1회 발생
                events.push(InputEvent::Action(action));
            }
        }

        // JustPressed -> Held 상태 전환
        for state in self.key_states.values_mut() {
            if *state == KeyState::JustPressed {
                *state = KeyState::Held;
            }
        }

        // DAS/ARR 업데이트
        let left_moves = self.das_left.update(delta, &self.das_arr);
        let right_moves = self.das_right.update(delta, &self.das_arr);
        let down_moves = self.das_down.update(delta, &self.das_arr);

        // 좌우 동시 입력 시 나중에 누른 쪽 우선 (현재는 왼쪽 우선)
        if left_moves > 0 && right_moves > 0 {
            // 둘 다 눌렸을 때는 서로 상쇄
        } else if left_moves > 0 {
            events.push(InputEvent::Repeat(GameAction::MoveLeft, left_moves));
        } else if right_moves > 0 {
            events.push(InputEvent::Repeat(GameAction::MoveRight, right_moves));
        }

        if down_moves > 0 {
            events.push(InputEvent::Repeat(GameAction::SoftDrop, down_moves));
        }

        events
    }

    /// 특정 키가 현재 눌려있는지 확인
    #[must_use]
    pub fn is_key_pressed(&self, key: VirtualKey) -> bool {
        matches!(
            self.key_states.get(&key),
            Some(KeyState::JustPressed | KeyState::Held)
        )
    }

    /// 특정 액션이 이번 프레임에 트리거되었는지 확인
    #[must_use]
    pub fn is_action_just_triggered(&self, action: GameAction) -> bool {
        self.key_states.iter().any(|(&key, &state)| {
            state == KeyState::JustPressed && self.bindings.get_action(key) == Some(action)
        })
    }

    /// 모든 키 상태 초기화
    pub fn reset(&mut self) {
        self.key_states.clear();
        self.das_left = DasState::default();
        self.das_right = DasState::default();
        self.das_down = DasState::default();
    }
}

/// 입력 이벤트
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    /// 단일 액션 발생
    Action(GameAction),
    /// 반복 액션 발생 (DAS/ARR에 의한)
    /// u32::MAX는 "끝까지" (ARR=0일 때)
    Repeat(GameAction, u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== KeyBindings 테스트 ==========

    #[test]
    fn test_default_bindings() {
        let bindings = KeyBindings::new();

        assert_eq!(
            bindings.get_action(VirtualKey::Left),
            Some(GameAction::MoveLeft)
        );
        assert_eq!(
            bindings.get_action(VirtualKey::Right),
            Some(GameAction::MoveRight)
        );
        assert_eq!(
            bindings.get_action(VirtualKey::Down),
            Some(GameAction::SoftDrop)
        );
        assert_eq!(
            bindings.get_action(VirtualKey::Space),
            Some(GameAction::HardDrop)
        );
        assert_eq!(
            bindings.get_action(VirtualKey::Up),
            Some(GameAction::RotateCW)
        );
        assert_eq!(
            bindings.get_action(VirtualKey::Z),
            Some(GameAction::RotateCCW)
        );
        assert_eq!(
            bindings.get_action(VirtualKey::A),
            Some(GameAction::Rotate180)
        );
        assert_eq!(bindings.get_action(VirtualKey::C), Some(GameAction::Hold));
        assert_eq!(
            bindings.get_action(VirtualKey::Escape),
            Some(GameAction::Pause)
        );
        assert_eq!(
            bindings.get_action(VirtualKey::R),
            Some(GameAction::Restart)
        );
    }

    #[test]
    fn test_custom_binding() {
        let mut bindings = KeyBindings::new();
        bindings.set_binding(VirtualKey::X, GameAction::Hold);

        assert_eq!(bindings.get_action(VirtualKey::X), Some(GameAction::Hold));
    }

    #[test]
    fn test_remove_binding() {
        let mut bindings = KeyBindings::new();
        bindings.remove_binding(VirtualKey::Z);

        assert_eq!(bindings.get_action(VirtualKey::Z), None);
    }

    // ========== DasArrConfig 테스트 ==========

    #[test]
    fn test_default_das_arr() {
        let config = DasArrConfig::default();

        assert_eq!(config.das, Duration::from_millis(DEFAULT_DAS_MS));
        assert_eq!(config.arr, Duration::from_millis(DEFAULT_ARR_MS));
    }

    #[test]
    fn test_custom_das_arr() {
        let config = DasArrConfig::new(150, 50);

        assert_eq!(config.das, Duration::from_millis(150));
        assert_eq!(config.arr, Duration::from_millis(50));
    }

    // ========== InputHandler 테스트 ==========

    #[test]
    fn test_input_handler_new() {
        let handler = InputHandler::new();
        assert!(!handler.is_key_pressed(VirtualKey::Left));
    }

    #[test]
    fn test_key_press_release() {
        let mut handler = InputHandler::new();

        handler.key_down(VirtualKey::Left);
        assert!(handler.is_key_pressed(VirtualKey::Left));

        handler.key_up(VirtualKey::Left);
        assert!(!handler.is_key_pressed(VirtualKey::Left));
    }

    #[test]
    fn test_just_pressed_action() {
        let mut handler = InputHandler::new();

        handler.key_down(VirtualKey::Space);
        assert!(handler.is_action_just_triggered(GameAction::HardDrop));

        // update 후에는 JustPressed -> Held
        handler.update(Duration::from_millis(16));
        assert!(!handler.is_action_just_triggered(GameAction::HardDrop));
    }

    #[test]
    fn test_immediate_action_on_press() {
        let mut handler = InputHandler::new();

        handler.key_down(VirtualKey::Left);
        let events = handler.update(Duration::ZERO);

        // 첫 프레스 시 즉시 액션 발생
        assert!(events.contains(&InputEvent::Action(GameAction::MoveLeft)));
    }

    #[test]
    fn test_das_no_repeat_before_threshold() {
        // DAS 100ms, ARR 50ms 설정
        let config = DasArrConfig::new(100, 50);
        let mut handler = InputHandler::with_das_arr(config);

        handler.key_down(VirtualKey::Left);
        handler.update(Duration::ZERO); // 첫 프레스 처리

        // 50ms 후 - DAS 미충전, 반복 없음
        let events = handler.update(Duration::from_millis(50));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, InputEvent::Repeat(GameAction::MoveLeft, _))),
            "Should not repeat before DAS threshold"
        );
    }

    #[test]
    fn test_das_repeat_after_threshold() {
        // DAS 100ms, ARR 50ms 설정
        let config = DasArrConfig::new(100, 50);
        let mut handler = InputHandler::with_das_arr(config);

        handler.key_down(VirtualKey::Left);
        handler.update(Duration::ZERO); // 첫 프레스 처리

        // 110ms 후 - DAS 충전 완료, 반복 시작
        let events = handler.update(Duration::from_millis(110));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, InputEvent::Repeat(GameAction::MoveLeft, n) if *n >= 1)),
            "Should repeat after DAS threshold"
        );
    }

    #[test]
    fn test_arr_zero_instant_move() {
        // DAS 100ms, ARR 0ms (즉시 끝까지)
        let config = DasArrConfig::new(100, 0);
        let mut handler = InputHandler::with_das_arr(config);

        handler.key_down(VirtualKey::Left);
        handler.update(Duration::ZERO);

        // DAS 충전 후
        let events = handler.update(Duration::from_millis(110));

        // ARR=0이면 u32::MAX (무한대) 반환
        assert!(
            events.iter().any(
                |e| matches!(e, InputEvent::Repeat(GameAction::MoveLeft, n) if *n == u32::MAX)
            ),
            "ARR=0 should return MAX moves"
        );
    }

    #[test]
    fn test_soft_drop_das() {
        let config = DasArrConfig::new(100, 0);
        let mut handler = InputHandler::with_das_arr(config);

        handler.key_down(VirtualKey::Down);
        let events = handler.update(Duration::ZERO);

        assert!(events.contains(&InputEvent::Action(GameAction::SoftDrop)));
    }

    #[test]
    fn test_rotation_no_das() {
        let mut handler = InputHandler::new();

        // 회전은 DAS 적용 안됨 - 한 번만 발생
        handler.key_down(VirtualKey::Up);
        let events = handler.update(Duration::ZERO);

        assert!(events.contains(&InputEvent::Action(GameAction::RotateCW)));

        // 계속 누르고 있어도 반복 안됨
        let events = handler.update(Duration::from_millis(200));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, InputEvent::Repeat(GameAction::RotateCW, _))),
            "Rotation should not repeat"
        );
    }

    #[test]
    fn test_left_right_cancel() {
        let mut handler = InputHandler::new();

        // 좌우 동시 입력
        handler.key_down(VirtualKey::Left);
        handler.key_down(VirtualKey::Right);
        handler.update(Duration::ZERO);

        // DAS 충전 후에도 좌우 상쇄
        let events = handler.update(Duration::from_millis(200));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, InputEvent::Repeat(GameAction::MoveLeft, _))),
            "Left should be cancelled"
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, InputEvent::Repeat(GameAction::MoveRight, _))),
            "Right should be cancelled"
        );
    }

    #[test]
    fn test_reset() {
        let mut handler = InputHandler::new();

        handler.key_down(VirtualKey::Left);
        handler.update(Duration::from_millis(50));

        handler.reset();

        assert!(!handler.is_key_pressed(VirtualKey::Left));
    }

    #[test]
    fn test_hold_multiple_keys() {
        let mut handler = InputHandler::new();

        // C와 Shift 모두 Hold에 바인딩됨
        handler.key_down(VirtualKey::C);
        assert!(handler.is_action_just_triggered(GameAction::Hold));

        handler.update(Duration::ZERO);
        handler.key_up(VirtualKey::C);

        handler.key_down(VirtualKey::LShift);
        assert!(handler.is_action_just_triggered(GameAction::Hold));
    }

    #[test]
    fn test_key_up_stops_das() {
        let config = DasArrConfig::new(100, 50);
        let mut handler = InputHandler::with_das_arr(config);

        handler.key_down(VirtualKey::Left);
        handler.update(Duration::from_millis(50));

        // 키를 뗌
        handler.key_up(VirtualKey::Left);

        // DAS가 초기화되어야 함
        let events = handler.update(Duration::from_millis(100));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, InputEvent::Repeat(GameAction::MoveLeft, _))),
            "DAS should be reset after key up"
        );
    }

    #[test]
    fn test_runtime_das_arr_update() {
        // 초기 DAS=100, ARR=50으로 시작
        let mut handler = InputHandler::with_das_arr(DasArrConfig::new(100, 50));

        handler.key_down(VirtualKey::Left);
        handler.update(Duration::ZERO); // 첫 프레스

        // 110ms 후 DAS 충전 완료 (DAS=100ms)
        let events = handler.update(Duration::from_millis(110));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, InputEvent::Repeat(GameAction::MoveLeft, _))),
            "Should repeat with DAS=100ms after 110ms"
        );

        // 런타임에 DAS를 200ms로 변경
        handler.set_das_arr(DasArrConfig::new(200, 50));

        // 키를 뗐다가 다시 눌러 DAS 리셋
        handler.key_up(VirtualKey::Left);
        handler.key_down(VirtualKey::Left);
        handler.update(Duration::ZERO); // 첫 프레스

        // 150ms 후 — 새 DAS(200ms)에는 부족
        let events = handler.update(Duration::from_millis(150));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, InputEvent::Repeat(GameAction::MoveLeft, _))),
            "Should NOT repeat with DAS=200ms after only 150ms"
        );

        // 추가 60ms (총 210ms) — 새 DAS(200ms) 초과
        let events = handler.update(Duration::from_millis(60));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, InputEvent::Repeat(GameAction::MoveLeft, _))),
            "Should repeat with DAS=200ms after 210ms total"
        );
    }
}
