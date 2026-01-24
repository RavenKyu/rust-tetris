//! 렌더링 모듈
//!
//! SDL3 기반 렌더링 시스템을 구현합니다.
//! - 색상 팔레트
//! - 레이아웃 계산
//! - 렌더링 상태
//! - SDL3 렌더러 (sdl3 feature 필요)

pub mod color;
pub mod layout;
pub mod state;

#[cfg(feature = "sdl3")]
pub mod renderer;

pub use color::{Color, Palette};
pub use layout::{
    DEFAULT_CELL_SIZE, DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, FIELD_HEIGHT, FIELD_WIDTH,
    Layout, MAX_CELL_SIZE, MIN_CELL_SIZE, NEXT_PREVIEW_COUNT, Rect,
};
pub use state::RenderState;

#[cfg(feature = "sdl3")]
pub use renderer::Renderer;
