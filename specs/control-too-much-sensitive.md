# Bug Fix: Control Too Much Sensitive

## Overview

Horizontal movement controls are too sensitive — DAS (100ms) triggers auto-repeat too quickly, and ARR (0ms) causes instant wall teleportation. Default values should be changed to casual-friendly settings, and a settings menu should be added for player customization.

## Background

The current DAS=100ms / ARR=0ms configuration is a competitive "zero-ARR" setup favored by experienced speedrunners. For most players, this feels twitchy and uncontrollable — a brief key hold causes the piece to instantly fly across the board. The Tetris Guideline suggests DAS around 133-167ms (8-10 frames at 60fps) and ARR around 50ms (3 frames at 60fps) as reasonable defaults. The `DasArrConfig` struct already supports custom values, but there is no way for players to change them at runtime.

## Functional Requirements

- [ ] FR-001: Change default DAS from 100ms to 167ms (10 frames at 60fps)
- [ ] FR-002: Change default ARR from 0ms to 50ms (3 frames at 60fps)
- [ ] FR-003: Add an in-game settings screen to configure DAS value (range: 50-300ms)
- [ ] FR-004: Add ARR configuration to settings screen (range: 0-100ms)
- [ ] FR-005: Persist DAS/ARR settings across game sessions (e.g., JSON config file)

## Non-Functional Requirements

- [ ] NFR-001: Settings changes take effect immediately without requiring a game restart
- [ ] NFR-002: Settings UI must be accessible from the pause menu or main menu

## Acceptance Criteria

- [ ] AC-001: Default DAS is 167ms — holding a key for less than 167ms does not trigger auto-repeat
- [ ] AC-002: Default ARR is 50ms — after DAS charges, piece moves 1 cell every 50ms
- [ ] AC-003: Player can adjust DAS/ARR values from a settings menu
- [ ] AC-004: Adjusted DAS/ARR values persist after closing and reopening the game
- [ ] AC-005: All existing input tests pass (updated for new defaults)

## Out of Scope

- Soft drop speed configuration
- Key rebinding UI
- Gamepad/controller support

## Technical Notes

- **Default constants**: Modify `DEFAULT_DAS_MS` (100 → 167) and `DEFAULT_ARR_MS` (0 → 50) in `src/input/mod.rs:13-16`
- **Runtime configuration**: `DasArrConfig::new()` already supports custom values — wire it to a settings system
- **Settings persistence**: Add a JSON config file (e.g., `settings.json`) to load/save DAS/ARR preferences
- **Settings UI**: New SDL3 screen with sliders or numeric inputs for DAS and ARR values
- **Integration points**: Settings must update `InputHandler.das_arr` at runtime; add a setter method if one doesn't exist
