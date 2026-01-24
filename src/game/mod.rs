//! 게임 핵심 로직 모듈
//!
//! 이 모듈은 테트리스 게임의 핵심 로직을 포함합니다:
//! - 플레이필드 (그리드 관리)
//! - 테트로미노 (블록 정의 및 생성)
//! - 회전 시스템 (SRS)
//! - 게임 메커니즘 (락다운, 홀드 등)
//! - 점수 시스템

pub mod playfield;
pub mod rotation;
pub mod tetromino;
// pub mod mechanics;   // 게임 메커니즘 (플레이필드, 테트로미노 구현 후)
// pub mod scoring;     // 점수 시스템 (메커니즘 구현 후)

pub use playfield::*;
pub use rotation::*;
pub use tetromino::*;
