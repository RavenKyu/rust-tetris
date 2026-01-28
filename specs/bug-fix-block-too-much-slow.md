# Bug Fix: Block Too Much Slow

## Overview

Blocks fall too slowly at low levels and spawn too high in the buffer zone, causing a noticeable delay before pieces become visible on screen. This makes the game feel unresponsive and sluggish, especially at the start of a marathon game.

## Background

The gravity table uses standard Tetris Guideline values (Level 0 = 1/60G, i.e., 1 cell per second at 60 FPS). However, the spawn position is set at Y=20 (matrix bottom), which places actual minos at rows 22-23 — entirely inside the invisible buffer zone (visible area is rows 0-19). Per the Tetris Guideline, pieces should spawn so that at least part of the piece is visible immediately (rows 20-21 for most pieces). The high spawn position combined with slow initial gravity creates a ~2-3 second blind delay before the player can see or interact with a new piece.

## Functional Requirements

- [ ] FR-001: Lower piece spawn position so that minos are partially visible at spawn (target: minos at rows 20-21 instead of 22-23)
- [ ] FR-002: Increase gravity table values at low levels — Level 0 target is approximately 4-5 cells per second (up from ~1 cell/sec)
- [ ] FR-003: Maintain a smooth progressive speed increase across levels (gravity curve should not have abrupt jumps)
- [ ] FR-004: High-level gravity values (Level 13+) remain unchanged, as they are already in the 20G range

## Non-Functional Requirements

- [ ] NFR-001: Frame rate must remain stable at 60 FPS after changes
- [ ] NFR-002: Gravity changes must not break lock delay or extended placement mechanics

## Acceptance Criteria

- [ ] AC-001: New pieces are partially visible immediately upon spawn (no blind delay in the buffer zone)
- [ ] AC-002: At Level 0, pieces fall at approximately 4-5 cells per second
- [ ] AC-003: Lock delay (500ms) and move reset (15 max) still function correctly after changes
- [ ] AC-004: All existing tests pass after changes

## Out of Scope

- Soft drop / hard drop speed changes
- DAS/ARR timing adjustments
- UI/rendering changes

## Technical Notes

- **Spawn position**: Modify `spawn_y_offset()` in `src/game/tetromino.rs:155-159` — reduce return value from `20` to `18` (or per-piece values) so minos appear at rows 20-21
- **Gravity table**: Adjust `GRAVITY_TABLE` entries for levels 0-12 in `src/game/mechanics.rs:30-51` — scale up low-level values by ~4-5x
- **Collision check**: Verify that collision detection works correctly with the new lower spawn position (pieces must not overlap existing blocks at spawn)
- **Testing**: Run existing unit tests for `GravitySystem`, `Tetromino::new()`, and playfield collision to confirm no regressions
