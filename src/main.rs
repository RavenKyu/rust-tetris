//! 테트리스 마라톤 모드 실행 파일
//!
//! SDL3 기반 테트리스 게임을 실행합니다.

#[cfg(feature = "sdl3")]
mod app {
    use std::time::{Duration, Instant};

    use sdl3::event::Event;
    use sdl3::keyboard::Keycode;

    use tetris::game::{GameState, Marathon};
    use tetris::input::{GameAction, InputEvent, InputHandler, VirtualKey};
    use tetris::render::{RenderState, Renderer};

    /// SDL Keycode를 VirtualKey로 변환
    fn keycode_to_virtual(keycode: Keycode) -> Option<VirtualKey> {
        match keycode {
            Keycode::Left => Some(VirtualKey::Left),
            Keycode::Right => Some(VirtualKey::Right),
            Keycode::Up => Some(VirtualKey::Up),
            Keycode::Down => Some(VirtualKey::Down),
            Keycode::Space => Some(VirtualKey::Space),
            Keycode::Z => Some(VirtualKey::Z),
            Keycode::X => Some(VirtualKey::X),
            Keycode::A => Some(VirtualKey::A),
            Keycode::C => Some(VirtualKey::C),
            Keycode::R => Some(VirtualKey::R),
            Keycode::Escape => Some(VirtualKey::Escape),
            Keycode::LShift => Some(VirtualKey::LShift),
            Keycode::RShift => Some(VirtualKey::RShift),
            _ => None,
        }
    }

    /// Marathon 게임에서 RenderState 생성
    fn create_render_state(game: &Marathon) -> RenderState {
        let state = game.state();
        RenderState {
            playfield: game.playfield().clone(),
            current_piece: game.current_piece().cloned(),
            hold_piece: game.held_piece(),
            can_hold: game.can_hold(),
            next_pieces: game.preview_next(5),
            score: game.score(),
            level: game.level(),
            lines: game.lines(),
            game_over: matches!(state, GameState::GameOver(_)),
            paused: state == GameState::Paused,
        }
    }

    /// 입력 이벤트를 게임 액션으로 처리
    fn handle_input_event(game: &mut Marathon, event: &InputEvent) {
        match event {
            InputEvent::Action(action) => {
                handle_action(game, *action, 1);
            }
            InputEvent::Repeat(action, count) => {
                handle_action(game, *action, *count);
            }
        }
    }

    /// 게임 액션 실행
    fn handle_action(game: &mut Marathon, action: GameAction, count: u32) {
        let state = game.state();

        match action {
            GameAction::Pause => {
                game.toggle_pause();
            }
            GameAction::Restart => {
                game.restart();
                game.start();
            }
            _ => {}
        }

        // 게임 진행 중일 때만 처리하는 액션
        if state != GameState::Playing {
            return;
        }

        match action {
            GameAction::MoveLeft => {
                let moves = if count == u32::MAX { 10 } else { count };
                for _ in 0..moves {
                    if !game.move_left() {
                        break;
                    }
                }
            }
            GameAction::MoveRight => {
                let moves = if count == u32::MAX { 10 } else { count };
                for _ in 0..moves {
                    if !game.move_right() {
                        break;
                    }
                }
            }
            GameAction::SoftDrop => {
                let drops = if count == u32::MAX { 40 } else { count };
                for _ in 0..drops {
                    if !game.soft_drop() {
                        break;
                    }
                }
            }
            GameAction::HardDrop => {
                game.hard_drop();
            }
            GameAction::RotateCW => {
                game.rotate_cw();
            }
            GameAction::RotateCCW => {
                game.rotate_ccw();
            }
            GameAction::Rotate180 => {
                game.rotate_180();
            }
            GameAction::Hold => {
                game.hold();
            }
            GameAction::Pause | GameAction::Restart => {}
        }
    }

    /// 메인 게임 루프 실행
    pub fn run() -> Result<(), String> {
        // SDL 초기화
        let sdl_context = sdl3::init().map_err(|e| e.to_string())?;

        // 렌더러 생성
        let mut renderer = Renderer::new(&sdl_context)?;

        // 입력 핸들러 생성
        let mut input = InputHandler::new();

        // 게임 생성 및 시작
        let mut game = Marathon::with_default_config();
        game.start();

        // 이벤트 펌프
        let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string())?;

        // 타이밍
        let mut last_update = Instant::now();
        let target_frame_time = Duration::from_secs_f64(1.0 / 60.0); // 60 FPS

        'game_loop: loop {
            let frame_start = Instant::now();

            // 이벤트 처리
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'game_loop,

                    Event::KeyDown {
                        keycode: Some(keycode),
                        repeat: false,
                        ..
                    } => {
                        // 메뉴 상태에서 아무 키나 누르면 게임 시작
                        if game.state() == GameState::Menu {
                            game.start();
                        }

                        if let Some(vkey) = keycode_to_virtual(keycode) {
                            input.key_down(vkey);
                        }
                    }

                    Event::KeyUp {
                        keycode: Some(keycode),
                        ..
                    } => {
                        if let Some(vkey) = keycode_to_virtual(keycode) {
                            input.key_up(vkey);
                        }
                    }

                    Event::Window {
                        win_event: sdl3::event::WindowEvent::Resized(w, h),
                        ..
                    } => {
                        renderer.handle_resize(w as u32, h as u32);
                    }

                    _ => {}
                }
            }

            // 델타 시간 계산
            let now = Instant::now();
            let delta = now.duration_since(last_update);
            last_update = now;

            // 입력 업데이트
            let input_events = input.update(delta);
            for event in &input_events {
                handle_input_event(&mut game, event);
            }

            // 게임 업데이트
            game.update(delta);

            // 렌더링
            let render_state = create_render_state(&game);
            renderer.render(&render_state)?;

            // 프레임 레이트 제어
            let frame_time = frame_start.elapsed();
            if frame_time < target_frame_time {
                std::thread::sleep(target_frame_time - frame_time);
            }
        }

        Ok(())
    }
}

#[cfg(feature = "sdl3")]
fn main() {
    if let Err(e) = app::run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "sdl3"))]
fn main() {
    eprintln!("SDL3 feature is not enabled.");
    eprintln!("Run with: cargo run --features sdl3");
    std::process::exit(1);
}
