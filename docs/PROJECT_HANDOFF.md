# Project Handoff

Last updated: 2026-08-18

Use this document to resume the project in a new ChatGPT or Codex conversation. It records the agreed direction, current state, constraints, and immediate next work. The files in `docs/` provide the deeper supporting detail.

## Resume prompt

Start a new conversation with:

> Read `AGENTS.md` and `docs/PROJECT_HANDOFF.md`, then inspect the remaining documents under `docs/`. We are planning an original round-based survival FPS for PSP homebrew by extending OpenStrike, Pocket3D, and PocketJS. Continue from the recorded current state. Do not write implementation code until we have selected and pinned an OpenStrike upstream revision and reconciled its repository structure with our scaffold. Preserve the fast development-loop requirements and PSP-1000 target.

If the repository is attached rather than locally available, attach or select the whole repository so the new conversation can read these files.

## Project goal

Create an original round-based survival FPS for PSP homebrew. The gameplay loop may be inspired by established survival shooters, but the project must have original branding, environments, characters, sprites, audio, and other distributable assets.

The project is intended to remain free, community-led, and open source. It is not being developed for sale. This describes the project's distribution policy; the final source licence still needs to be selected and must remain compatible with the permissively licensed upstream source.

The visual direction is a true 3D environment with Doom-era presentation techniques:

- BSP-based 3D levels
- billboard sprites for enemies, pickups, and effects
- screen-space weapon sprites
- baked lighting
- hitscan combat where appropriate
- low runtime asset and memory cost

Gameplay details, final settings, and controls are intentionally deferred.

## Decisions already made

1. Target PSP-1000 constraints first so later PSP models also work.
2. Extend OpenStrike/Pocket3D instead of building a renderer from scratch.
3. Enhance reusable upstream components where practical rather than merely wrapping existing behavior.
4. Use Rust for rendering, collision, spatial queries, navigation, fixed-step simulation, and per-entity hot paths.
5. Use PocketJS/TypeScript for round rules, configuration, scoring, tuning, HUD, menus, debug UI, and other high-iteration policy.
6. Use one compact native-to-guest event/snapshot transfer and one queued guest-to-native command transfer per fixed tick.
7. Do not call native functions once per enemy or once per rendered object from JavaScript.
8. Make the base game use the same domain surface that future mods would receive.
9. Target deterministic, scriptable verification from the beginning.
10. Do not include proprietary Counter-Strike or Call of Duty assets in the repository or releases.

The rationale is recorded in `docs/decisions/0001-openstrike-pocketjs.md`.

## Confirmed technology model

OpenStrike is not written only in PocketJS:

| Area | Technology | Role |
| --- | --- | --- |
| Game and rendering core | Rust | Simulation, collision, BSP, rendering, actors, weapons |
| Script guest | TypeScript on QuickJS | Rules, policies, scoring, tuning |
| HUD and menus | Solid JSX through PocketJS | Reactive UI and overlays |
| Desktop renderer | Pocket3D with wgpu/winit | Native macOS interactive development |
| PSP renderer | Pocket3D GU with sceGu | Hardware target |
| Build and verification | Cargo plus Bun/TypeScript | Builds, asset cooking, packaging, tests |

Pocket3D is a lean 3D substrate, not a general-purpose editor-driven engine. OpenStrike is the first specialized FPS runtime built from it.

## Development-loop decision

Fast iteration is a primary product requirement.

Use this order for normal work:

1. Narrow unit or contract test.
2. Deterministic headless scenario.
3. Native macOS build when interactive feel matters.
4. PPSSPP scripted journey for PSP parity.
5. Physical PSP-1000 for milestone performance and memory validation.

The goal is not necessarily zero process restarts. The goal is zero manual navigation or manual state reconstruction after a restart.

Every significant feature should eventually support:

- named scenario selection;
- fixed random seed;
- direct map and spawn selection;
- bounded frame count or `--auto-quit`;
- recorded input tapes;
- machine-readable state output;
- exact-frame screenshot capture; and
- deterministic replay or golden comparison.

See `docs/development-loop.md` for the full workflow.

## Platform expectations

### Native macOS

This is the intended full interactive development target. OpenStrike documents a native Pocket3D desktop build using wgpu/winit, with the same core simulation and PocketJS product bundle used by the PSP target.

### Headless desktop

This is the preferred Codex verification target. OpenStrike already supports scripted, seeded scenarios, offscreen rendering, screenshots, and automatic exit. Extend this model for every new gameplay system.

### Browser

PocketJS supports a WASM browser host, DevTools, and UI/component development. The published OpenStrike project does not currently document its complete 3D game as a browser target. Initially use the browser for isolated HUD, menu, and guest behavior work. Do not make a full browser port a prerequisite.

### PPSSPP

Use scripted journeys and frame goldens as a PSP compatibility gate. It should not be the default edit-test loop.

### Physical PSP

Use PocketJS DevTools and PSPLINK for one-command build/relaunch, logs, screenshots, pause/step, and input-tape debugging. Use the PSP-1000 for performance and memory acceptance.

## Current repository state

The repository is a documentation-only scaffold. It currently contains no implementation code and no third-party game assets.

Milestone 1 is complete. The upstream audit selected the following immutable revisions:

- OpenStrike: `fcfe93e9b2821524d6f6e834d15939cb18bc6e3d`
- PocketJS/Pocket3D: `5bfaff7091e63a1cd93fe46fd5a4f8b61b46b335`
- quickjs-rs: `0fc946fb670c0c29bc0135f510bcb0f595415a61`
- rust-psp: `2cbaf8c9bc72569c76240a1d9743de10731e5f6b`

ADR 0002 selects a direct OpenStrike downstream while retaining the engine and PSP dependencies as pinned submodules. The upstream source has not yet been integrated, built, or run locally; those tasks belong to Milestone 2.

- `AGENTS.md` — constraints and working agreement for coding agents.
- `README.md` — project overview.
- `docs/architecture/README.md` — proposed native/guest split.
- `docs/development-loop.md` — fast-loop requirements.
- `docs/decisions/0001-openstrike-pocketjs.md` — initial ADR.
- `docs/research/pocketjs-cross-reference.md` — documentation findings and sources.
- `crates/` — reserved for future Rust extensions.
- `game/` — reserved for PocketJS rules and HUD.
- `assets/` — reserved for original source/cooked assets.
- `tests/` — reserved for scenarios, tapes, and goldens.
- `tools/` — reserved for project workflow helpers.

The empty implementation directories are provisional. Do not force them onto OpenStrike if its established structure provides a better location.

## Immediate next steps

Do these in order:

1. Add the official OpenStrike repository as the `openstrike` remote.
2. Integrate the selected OpenStrike commit while preserving both histories and making no product changes.
3. Preserve the exact PocketJS, quickjs-rs, and rust-psp submodule pins.
4. Reconcile the scaffold with the upstream layout and move planning documents under `docs/project/`.
5. Build unmodified OpenStrike on macOS.
6. Run at least one deterministic headless scenario using local, uncommitted map data.
7. Build its PSP EBOOT and run its existing PPSSPP journey.
8. Record baseline build time, relaunch time, frame time, and memory before modifications.
9. Replace the upstream soldier and proprietary-map dependency with original test content before the first project-authored playable slice.
10. Only then implement the smallest vertical slice: original test room, movement, one target, and one hitscan weapon.

## Questions intentionally deferred

- Project and game title
- Setting and visual theme
- Final controls
- Weapon roster
- Economy details
- Round formulas
- Enemy types
- Perks or upgrade systems
- Multiplayer
- Final map tooling and editor workflow

Do not block foundation work on these decisions.

## Working principles for a new assistant

- Read the repository before proposing new structure.
- Cross-reference current PocketJS/OpenStrike documentation when architecture depends on it.
- Prefer modifying and validating small slices over generating large amounts of speculative code.
- Preserve deterministic tests and desktop/headless parity with every PSP feature.
- Keep reusable improvements separable so they can potentially be contributed upstream.
- Report any divergence between documentation and the pinned source tree.
- Never claim a feature runs on hardware until it has actually been tested there.

## Primary references

- OpenStrike: https://github.com/pocket-stack/open-strike
- PocketJS: https://github.com/pocket-stack/pocketjs
- PocketJS overview: https://pocketjs.dev/docs/overview/
- PocketJS architecture: https://pocketjs.dev/docs/architecture/
- PocketJS getting started: https://pocketjs.dev/docs/getting-started/
- PocketJS DevTools: https://pocketjs.dev/docs/devtools/
- PocketJS build pipeline: https://pocketjs.dev/docs/build-pipeline/
- PocketJS runtime family: https://github.com/pocket-stack/pocketjs/blob/main/docs/RUNTIMES.md
- Pocket3D: https://github.com/pocket-stack/pocketjs/blob/main/engine/pocket3d/README.md

## Definition of a successful handoff

A new conversation should be able to read `AGENTS.md` and this file, explain the architecture and current state accurately, and identify upstream pinning plus baseline verification as the next task without needing the prior chat transcript.
