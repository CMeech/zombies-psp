# Pocket Survival

Working repository scaffold for an original PSP round-based survival FPS.

This project is intended to extend OpenStrike and the PocketJS runtime family. It will use a native Rust simulation/rendering core for frame-critical work and a PocketJS guest for rules, configuration, UI, and other iteration-heavy behavior.

## Current phase

Planning and development-loop design only. No implementation code or third-party game assets are included yet.

## Priorities

1. Keep the normal edit-test cycle on macOS or in deterministic headless tests.
2. Preserve PSP-1000 compatibility from the first playable build.
3. Put gameplay policy in PocketJS where it remains efficient.
4. Keep per-entity and per-frame hot paths in Rust.
5. Make every important behavior scriptable and replayable.
6. Use only original or properly licensed maps, sprites, audio, and branding.

## Repository map

- `docs/` — architecture, decisions, research, and workflow.
- `crates/` — future Rust runtime/core extensions.
- `game/` — future PocketJS game rules and HUD.
- `assets/` — original source assets and cooked target assets.
- `tests/` — future headless scenarios, input tapes, and golden images.
- `tools/` — future local build and asset-pipeline helpers.

The implementation layout will be reconciled with the selected OpenStrike revision before code is introduced.
