//! 렌더러 모듈
//!
//! SDL3 기반 렌더링 구현

use sdl3::Sdl;
use sdl3::pixels::Color as SdlColor;
use sdl3::rect::Rect as SdlRect;
use sdl3::render::{Canvas, TextureCreator};
use sdl3::video::{Window, WindowContext};

use super::color::{Color, Palette};
use super::layout::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, Layout, Rect};
use super::state::{RenderState, SettingsState};
use crate::settings::GameSettings;
use crate::game::{
    Cell, Playfield, Tetromino, TetrominoKind, VISIBLE_HEIGHT, calculate_ghost_position,
};

// ============================================================================
// 타입 변환
// ============================================================================

/// Color를 SDL Color로 변환
fn to_sdl_color(c: Color) -> SdlColor {
    SdlColor::RGBA(c.r, c.g, c.b, c.a)
}

/// Rect를 SDL Rect로 변환
fn to_sdl_rect(r: Rect) -> SdlRect {
    SdlRect::new(r.x, r.y, r.width, r.height)
}

// 테스트용 From 구현
impl From<Color> for SdlColor {
    fn from(c: Color) -> Self {
        SdlColor::RGBA(c.r, c.g, c.b, c.a)
    }
}

impl From<Rect> for SdlRect {
    fn from(r: Rect) -> Self {
        SdlRect::new(r.x, r.y, r.width, r.height)
    }
}

// ============================================================================
// SDL3 렌더러
// ============================================================================

/// SDL3 기반 렌더러
pub struct Renderer {
    canvas: Canvas<Window>,
    #[allow(dead_code)]
    texture_creator: TextureCreator<WindowContext>,
    layout: Layout,
}

impl Renderer {
    /// 새 렌더러 생성
    pub fn new(sdl_context: &Sdl) -> Result<Self, String> {
        let video_subsystem = sdl_context.video().map_err(|e| e.to_string())?;

        let window = video_subsystem
            .window("Tetris", DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window.into_canvas();
        let texture_creator = canvas.texture_creator();
        let layout = Layout::default();

        Ok(Self {
            canvas,
            texture_creator,
            layout,
        })
    }

    /// 윈도우 크기 변경 처리
    pub fn handle_resize(&mut self, width: u32, height: u32) {
        self.layout = Layout::calculate(width, height);
    }

    /// 전체 화면 렌더링
    pub fn render(&mut self, state: &RenderState) -> Result<(), String> {
        // 배경 클리어
        self.canvas
            .set_draw_color(to_sdl_color(Palette::BACKGROUND));
        self.canvas.clear();

        // 플레이필드 렌더링
        self.render_playfield(&state.playfield)?;

        // 고스트 피스 렌더링
        if let Some(ref piece) = state.current_piece {
            self.render_ghost_piece(piece, &state.playfield)?;
        }

        // 현재 테트로미노 렌더링
        if let Some(ref piece) = state.current_piece {
            self.render_tetromino(piece)?;
        }

        // 홀드 영역 렌더링
        self.render_hold(state.hold_piece, state.can_hold)?;

        // Next 프리뷰 렌더링
        self.render_next_preview(&state.next_pieces)?;

        // 점수/레벨/라인 렌더링
        self.render_info(state.score, state.level, state.lines)?;

        // 게임 오버/일시정지/설정 오버레이
        if state.settings.active {
            self.render_settings_overlay(&state.settings)?;
        } else if state.game_over {
            self.render_game_over_overlay()?;
        } else if state.paused {
            self.render_pause_overlay()?;
        }

        // 화면 표시
        self.canvas.present();

        Ok(())
    }

    /// 플레이필드 렌더링
    fn render_playfield(&mut self, playfield: &Playfield) -> Result<(), String> {
        // 플레이필드 배경
        self.canvas.set_draw_color(to_sdl_color(Palette::GRID));
        self.canvas
            .fill_rect(to_sdl_rect(self.layout.playfield))
            .map_err(|e| e.to_string())?;

        // 그리드 라인
        self.render_grid_lines()?;

        // 셀 렌더링
        for y in 0..VISIBLE_HEIGHT {
            for x in 0..10 {
                if let Some(Cell::Filled(color)) = playfield.get(x, y) {
                    let cell_color = Palette::from_cell_color(*color).locked();
                    self.render_cell(x as i32, y as i32, cell_color)?;
                }
            }
        }

        Ok(())
    }

    /// 그리드 라인 렌더링
    fn render_grid_lines(&mut self) -> Result<(), String> {
        self.canvas.set_draw_color(to_sdl_color(Palette::GRID_LINE));

        let cell = self.layout.cell_size as i32;
        let field = &self.layout.playfield;

        // 수직선
        for i in 0..=10 {
            let x = field.x + i * cell;
            self.canvas
                .draw_line((x, field.y), (x, field.bottom()))
                .map_err(|e| e.to_string())?;
        }

        // 수평선
        for i in 0..=20 {
            let y = field.y + i * cell;
            self.canvas
                .draw_line((field.x, y), (field.right(), y))
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// 단일 셀 렌더링
    fn render_cell(&mut self, cell_x: i32, cell_y: i32, color: Color) -> Result<(), String> {
        let rect = self.layout.cell_rect(cell_x, cell_y);

        // 메인 색상
        self.canvas.set_draw_color(to_sdl_color(color));
        self.canvas
            .fill_rect(to_sdl_rect(rect))
            .map_err(|e| e.to_string())?;

        // 테두리 (약간 어둡게)
        let border_color = color.with_brightness(0.6);
        self.canvas.set_draw_color(to_sdl_color(border_color));
        self.canvas
            .draw_rect(to_sdl_rect(rect))
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 테트로미노 렌더링
    fn render_tetromino(&mut self, tetromino: &Tetromino) -> Result<(), String> {
        let color = Palette::from_cell_color(tetromino.color());

        for (bx, by) in tetromino.blocks() {
            // 가시 영역 내의 블록만 렌더링
            if by >= 0 && by < VISIBLE_HEIGHT as i32 {
                self.render_cell(bx, by, color)?;
            }
        }

        Ok(())
    }

    /// 고스트 피스 렌더링
    fn render_ghost_piece(
        &mut self,
        tetromino: &Tetromino,
        playfield: &Playfield,
    ) -> Result<(), String> {
        let ghost_y = calculate_ghost_position(tetromino, playfield);
        let color = Palette::from_cell_color(tetromino.color()).ghost();

        let shape = tetromino.shape();
        for (row_idx, row) in shape.iter().enumerate() {
            for (col_idx, &filled) in row.iter().enumerate() {
                if filled {
                    let bx = tetromino.x + col_idx as i32;
                    let by = ghost_y + (3 - row_idx as i32);

                    if by >= 0 && by < VISIBLE_HEIGHT as i32 {
                        self.render_cell(bx, by, color)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 홀드 영역 렌더링
    fn render_hold(
        &mut self,
        hold_piece: Option<TetrominoKind>,
        can_hold: bool,
    ) -> Result<(), String> {
        // 영역 배경
        self.canvas.set_draw_color(to_sdl_color(Palette::GRID));
        self.canvas
            .fill_rect(to_sdl_rect(self.layout.hold))
            .map_err(|e| e.to_string())?;

        // 테두리
        let border_color = if can_hold {
            Palette::BORDER
        } else {
            Color::rgb(0x50, 0x30, 0x30) // 홀드 불가 시 어두운 빨강
        };
        self.canvas.set_draw_color(to_sdl_color(border_color));
        self.canvas
            .draw_rect(to_sdl_rect(self.layout.hold))
            .map_err(|e| e.to_string())?;

        // 홀드된 미노
        if let Some(kind) = hold_piece {
            self.render_preview_mino(&self.layout.hold.clone(), kind)?;
        }

        Ok(())
    }

    /// Next 프리뷰 렌더링
    fn render_next_preview(&mut self, next_pieces: &[TetrominoKind]) -> Result<(), String> {
        // 레이아웃 복사하여 borrow 문제 해결
        let next_regions = self.layout.next;

        for (i, region) in next_regions.iter().enumerate() {
            // 영역 배경
            self.canvas.set_draw_color(to_sdl_color(Palette::GRID));
            self.canvas
                .fill_rect(to_sdl_rect(*region))
                .map_err(|e| e.to_string())?;

            // 테두리
            self.canvas.set_draw_color(to_sdl_color(Palette::BORDER));
            self.canvas
                .draw_rect(to_sdl_rect(*region))
                .map_err(|e| e.to_string())?;

            // 미노 렌더링
            if let Some(&kind) = next_pieces.get(i) {
                self.render_preview_mino(region, kind)?;
            }
        }

        Ok(())
    }

    /// 프리뷰 영역에 미노 렌더링
    fn render_preview_mino(&mut self, region: &Rect, kind: TetrominoKind) -> Result<(), String> {
        let (offset_x, offset_y) = self.layout.preview_mino_offset(region);
        let color = Palette::from_cell_color(kind.color());
        let shape = kind.shape();
        let cell_size = self.layout.cell_size;

        for (row_idx, row) in shape.iter().enumerate() {
            for (col_idx, &filled) in row.iter().enumerate() {
                if filled {
                    let px = offset_x + (col_idx as u32 * cell_size) as i32;
                    let py = offset_y + (row_idx as u32 * cell_size) as i32;
                    let rect = SdlRect::new(px, py, cell_size, cell_size);

                    self.canvas.set_draw_color(to_sdl_color(color));
                    self.canvas.fill_rect(rect).map_err(|e| e.to_string())?;

                    let border = color.with_brightness(0.6);
                    self.canvas.set_draw_color(to_sdl_color(border));
                    self.canvas.draw_rect(rect).map_err(|e| e.to_string())?;
                }
            }
        }

        Ok(())
    }

    /// 점수/레벨/라인 정보 렌더링
    fn render_info(&mut self, score: u32, level: u32, lines: u32) -> Result<(), String> {
        // 간단한 숫자 표시 (폰트 없이 사각형으로 대체)
        // TODO: SDL_ttf 연동 시 텍스트 렌더링 구현

        // 점수 영역 배경
        self.canvas.set_draw_color(to_sdl_color(Palette::GRID));
        self.canvas
            .fill_rect(to_sdl_rect(self.layout.score))
            .map_err(|e| e.to_string())?;
        self.canvas
            .fill_rect(to_sdl_rect(self.layout.level))
            .map_err(|e| e.to_string())?;
        self.canvas
            .fill_rect(to_sdl_rect(self.layout.lines))
            .map_err(|e| e.to_string())?;

        // 테두리
        self.canvas.set_draw_color(to_sdl_color(Palette::BORDER));
        self.canvas
            .draw_rect(to_sdl_rect(self.layout.score))
            .map_err(|e| e.to_string())?;
        self.canvas
            .draw_rect(to_sdl_rect(self.layout.level))
            .map_err(|e| e.to_string())?;
        self.canvas
            .draw_rect(to_sdl_rect(self.layout.lines))
            .map_err(|e| e.to_string())?;

        // 숫자 표시를 위한 간단한 바 형태
        self.render_info_bar(&self.layout.score.clone(), score, 999_999)?;
        self.render_info_bar(&self.layout.level.clone(), level, 20)?;
        self.render_info_bar(&self.layout.lines.clone(), lines, 150)?;

        Ok(())
    }

    /// 정보 바 렌더링 (진행률 표시)
    fn render_info_bar(&mut self, region: &Rect, value: u32, max_value: u32) -> Result<(), String> {
        let ratio = (value as f32 / max_value as f32).min(1.0);
        let bar_width = ((region.width - 4) as f32 * ratio) as u32;

        if bar_width > 0 {
            let bar_rect = Rect::new(region.x + 2, region.y + 2, bar_width, region.height - 4);
            self.canvas.set_draw_color(to_sdl_color(Palette::CYAN));
            self.canvas
                .fill_rect(to_sdl_rect(bar_rect))
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// 게임 오버 오버레이
    fn render_game_over_overlay(&mut self) -> Result<(), String> {
        // 반투명 검은 오버레이
        self.canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 180));
        self.canvas
            .fill_rect(to_sdl_rect(self.layout.playfield))
            .map_err(|e| e.to_string())?;

        // 중앙에 빨간 사각형 (GAME OVER 표시)
        let center_rect = Rect::new(
            self.layout.playfield.center_x() - 50,
            self.layout.playfield.center_y() - 20,
            100,
            40,
        );
        self.canvas.set_draw_color(to_sdl_color(Palette::RED));
        self.canvas
            .fill_rect(to_sdl_rect(center_rect))
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 일시정지 오버레이
    fn render_pause_overlay(&mut self) -> Result<(), String> {
        // 반투명 파란 오버레이
        self.canvas.set_draw_color(SdlColor::RGBA(0, 0, 50, 150));
        self.canvas
            .fill_rect(to_sdl_rect(self.layout.playfield))
            .map_err(|e| e.to_string())?;

        // 일시정지 아이콘 (두 개의 수직 막대)
        let bar_width = 15;
        let bar_height = 50;
        let gap = 20;
        let cx = self.layout.playfield.center_x();
        let cy = self.layout.playfield.center_y();

        let left_bar = Rect::new(
            cx - gap / 2 - bar_width as i32,
            cy - bar_height as i32 / 2,
            bar_width,
            bar_height,
        );
        let right_bar = Rect::new(
            cx + gap / 2,
            cy - bar_height as i32 / 2,
            bar_width,
            bar_height,
        );

        self.canvas.set_draw_color(to_sdl_color(Palette::TEXT));
        self.canvas
            .fill_rect(to_sdl_rect(left_bar))
            .map_err(|e| e.to_string())?;
        self.canvas
            .fill_rect(to_sdl_rect(right_bar))
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 설정 화면 오버레이
    fn render_settings_overlay(&mut self, settings: &SettingsState) -> Result<(), String> {
        let field = &self.layout.playfield;

        // 반투명 어두운 오버레이
        self.canvas.set_draw_color(SdlColor::RGBA(10, 10, 30, 200));
        self.canvas
            .fill_rect(to_sdl_rect(*field))
            .map_err(|e| e.to_string())?;

        let panel_width = (field.width as f32 * 0.8) as u32;
        let panel_height = (field.height as f32 * 0.4) as u32;
        let panel_x = field.center_x() - panel_width as i32 / 2;
        let panel_y = field.center_y() - panel_height as i32 / 2;

        // 패널 배경
        let panel = Rect::new(panel_x, panel_y, panel_width, panel_height);
        self.canvas.set_draw_color(to_sdl_color(Palette::GRID));
        self.canvas
            .fill_rect(to_sdl_rect(panel))
            .map_err(|e| e.to_string())?;
        self.canvas.set_draw_color(to_sdl_color(Palette::BORDER));
        self.canvas
            .draw_rect(to_sdl_rect(panel))
            .map_err(|e| e.to_string())?;

        let item_height = panel_height / 3;
        let bar_margin = 10;
        let bar_height = (item_height as i32 - bar_margin * 2).max(8) as u32;
        let bar_width = (panel_width as i32 - bar_margin * 4).max(20) as u32;
        let bar_x = panel_x + bar_margin * 2;

        // DAS 슬라이더 (항목 0)
        let das_y = panel_y + bar_margin;
        self.render_settings_slider(
            bar_x,
            das_y,
            bar_width,
            bar_height,
            settings.das_ms,
            GameSettings::DAS_MIN,
            GameSettings::DAS_MAX,
            settings.selected_item == 0,
            Palette::CYAN,
        )?;

        // ARR 슬라이더 (항목 1)
        let arr_y = panel_y + item_height as i32 + bar_margin;
        self.render_settings_slider(
            bar_x,
            arr_y,
            bar_width,
            bar_height,
            settings.arr_ms,
            GameSettings::ARR_MIN,
            GameSettings::ARR_MAX,
            settings.selected_item == 1,
            Palette::GREEN,
        )?;

        // 하단 힌트 영역 — 작은 사각형으로 조작 키 표시
        let hint_y = panel_y + item_height as i32 * 2 + bar_margin;
        let hint_size = 12_u32;
        let hint_gap = 6;
        let hints_total_width = hint_size * 4 + hint_gap as u32 * 3;
        let hint_start_x = panel_x + (panel_width as i32 - hints_total_width as i32) / 2;

        // 위 화살표 (항목 전환)
        self.canvas.set_draw_color(to_sdl_color(Palette::TEXT_SECONDARY));
        let up_rect = Rect::new(hint_start_x, hint_y, hint_size, hint_size);
        self.canvas
            .fill_rect(to_sdl_rect(up_rect))
            .map_err(|e| e.to_string())?;

        // 좌 화살표 (값 감소)
        let left_rect = Rect::new(
            hint_start_x + hint_size as i32 + hint_gap,
            hint_y,
            hint_size,
            hint_size,
        );
        self.canvas
            .fill_rect(to_sdl_rect(left_rect))
            .map_err(|e| e.to_string())?;

        // 우 화살표 (값 증가)
        let right_rect = Rect::new(
            hint_start_x + (hint_size as i32 + hint_gap) * 2,
            hint_y,
            hint_size,
            hint_size,
        );
        self.canvas
            .fill_rect(to_sdl_rect(right_rect))
            .map_err(|e| e.to_string())?;

        // ESC (닫기)
        let esc_rect = Rect::new(
            hint_start_x + (hint_size as i32 + hint_gap) * 3,
            hint_y,
            hint_size,
            hint_size,
        );
        self.canvas.set_draw_color(to_sdl_color(Palette::RED));
        self.canvas
            .fill_rect(to_sdl_rect(esc_rect))
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 설정 슬라이더 렌더링
    #[allow(clippy::too_many_arguments)]
    fn render_settings_slider(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        value: u64,
        min_val: u64,
        max_val: u64,
        selected: bool,
        fill_color: Color,
    ) -> Result<(), String> {
        // 선택 표시 (좌측에 작은 인디케이터)
        if selected {
            let indicator = Rect::new(x - 8, y + height as i32 / 2 - 4, 4, 8);
            self.canvas.set_draw_color(to_sdl_color(Palette::TEXT));
            self.canvas
                .fill_rect(to_sdl_rect(indicator))
                .map_err(|e| e.to_string())?;
        }

        // 슬라이더 트랙 배경
        let track = Rect::new(x, y, width, height);
        self.canvas
            .set_draw_color(to_sdl_color(Palette::BACKGROUND));
        self.canvas
            .fill_rect(to_sdl_rect(track))
            .map_err(|e| e.to_string())?;

        // 채우기 바
        let range = max_val - min_val;
        let ratio = if range > 0 {
            ((value - min_val) as f32 / range as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let fill_width = ((width - 2) as f32 * ratio) as u32;
        if fill_width > 0 {
            let fill = Rect::new(x + 1, y + 1, fill_width, height - 2);
            self.canvas.set_draw_color(to_sdl_color(fill_color));
            self.canvas
                .fill_rect(to_sdl_rect(fill))
                .map_err(|e| e.to_string())?;
        }

        // 테두리 (선택 시 밝게)
        let border_color = if selected {
            Palette::TEXT
        } else {
            Palette::BORDER
        };
        self.canvas.set_draw_color(to_sdl_color(border_color));
        self.canvas
            .draw_rect(to_sdl_rect(track))
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 현재 레이아웃 반환
    #[must_use]
    pub const fn layout(&self) -> &Layout {
        &self.layout
    }
}

// ============================================================================
// 테스트 (SDL 의존성 필요)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_sdl_color() {
        let c = Color::new(100, 150, 200, 255);
        let sdl_c: SdlColor = c.into();
        assert_eq!(sdl_c.r, 100);
        assert_eq!(sdl_c.g, 150);
        assert_eq!(sdl_c.b, 200);
        assert_eq!(sdl_c.a, 255);
    }

    #[test]
    fn test_rect_to_sdl_rect() {
        let r = Rect::new(10, 20, 100, 50);
        let sdl_r: SdlRect = r.into();
        assert_eq!(sdl_r.x, 10);
        assert_eq!(sdl_r.y, 20);
        assert_eq!(sdl_r.w, 100);
        assert_eq!(sdl_r.h, 50);
    }

    #[test]
    fn test_to_sdl_color_function() {
        let c = Color::rgb(255, 128, 64);
        let sdl_c = to_sdl_color(c);
        assert_eq!(sdl_c.r, 255);
        assert_eq!(sdl_c.g, 128);
        assert_eq!(sdl_c.b, 64);
        assert_eq!(sdl_c.a, 255);
    }

    #[test]
    fn test_to_sdl_rect_function() {
        let r = Rect::new(5, 10, 200, 100);
        let sdl_r = to_sdl_rect(r);
        assert_eq!(sdl_r.x, 5);
        assert_eq!(sdl_r.y, 10);
        assert_eq!(sdl_r.w, 200);
        assert_eq!(sdl_r.h, 100);
    }
}
