# Agent Working Agreement

## Project stage

This repository is documentation-only until an OpenStrike upstream revision is selected and recorded. Do not generate implementation code unless explicitly requested.

## Product boundaries

- Build an original round-based survival FPS for PSP homebrew.
- Do not add Call of Duty, Nazi Zombies, Counter-Strike, Valve, or other copyrighted game assets, names, maps, audio, or branding.
- Treat `pocket-survival` as a temporary working name.
- Defer gameplay tuning and final controls until explicitly scheduled.

## Architecture boundaries

- Start from OpenStrike/Pocket3D rather than creating a second renderer.
- Keep rendering, collision, navigation, spatial queries, and per-entity simulation in Rust.
- Prefer PocketJS for rules, state transitions, tuning tables, HUD, menus, debug panels, and content configuration.
- Cross the guest/native boundary once per fixed tick using batched facts and queued intent.
- Never perform per-enemy or per-pixel native calls from JavaScript.
- Target PSP-1000 memory and performance limits first.

## Development-loop requirements

- Every feature should have a deterministic headless path when practical.
- Prefer scripted input and screenshot/state assertions over manual repetition.
- Use the native desktop build for interactive gameplay checks.
- Use the browser host for isolated PocketJS UI and component work.
- Use PPSSPP for target-parity gates, not as the default edit loop.
- Use physical PSP tests for milestone validation and performance budgets.
- Avoid introducing a tool or step that requires manual app reset when an automated relaunch or replay can cover it.

## Change discipline

- Keep changes small and independently testable.
- Record significant architecture choices in `docs/project/decisions/`.
- Preserve upstream attribution and licences.
- Never commit proprietary test maps or cooked derivatives of proprietary assets.
