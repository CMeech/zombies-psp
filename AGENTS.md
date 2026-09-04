# Agent Working Agreement

This file applies to the whole repository. Read it before changing code or documentation.

## Start here

Read these files in order:

1. `docs/project/ROADMAP.md`
2. `docs/project/first-vertical-slice.md`
3. `docs/project/architecture/README.md`
4. `docs/project/development-loop.md`
5. `docs/project/macos-setup.md` when build tools are needed

Current stage: **Milestone 4 — implement the first vertical slice**.

- Milestones 0–2 are complete.
- Milestone 3 is complete; its accepted contract is recorded in
  `docs/project/first-vertical-slice.md` and ADR 0003.
- Implement Milestone 4 in the independently testable order recorded in the
  slice contract. Do not expand into Milestone 5 systems or deferred product
  decisions.
- Small fixes needed to keep the verified baseline working are allowed when explicitly requested.

## Product rules

- Build an original round-based survival FPS for PSP homebrew.
- `pocket-survival` is a temporary working name.
- Do not add copyrighted game names, branding, maps, models, sprites, textures, audio, or other assets.
- Never commit proprietary BSP/WAD files or cooked derivatives.
- Use only original or suitably licensed project content.
- Defer final controls, tuning, setting, and content scope until their roadmap milestone.

## Architecture rules

- Extend OpenStrike and Pocket3D. Do not create another renderer or parallel engine.
- Rust owns rendering, collision, navigation, spatial queries, fixed-step simulation, and per-entity hot paths.
- PocketJS owns rules, state transitions, tuning, HUD, menus, debug UI, and content configuration where practical.
- Cross the guest/native boundary once per fixed tick: one batched fact/event transfer and one queued intent transfer.
- Never make per-enemy, per-object, or per-pixel native calls from JavaScript.
- Target PSP-1000 memory and performance limits first.
- Keep reusable upstream improvements separable from project-specific behavior.
- Record significant architecture decisions in `docs/project/decisions/`.

## Platform priority

- PSP is the primary product and milestone validation target. Prioritize PSP packaging, PPSSPP compatibility, and physical PSP evidence over work on secondary hosts.
- Vita and Symbian are inherited secondary hosts. Do not spend milestone time on their full builds or platform-specific features unless explicitly requested.
- When shared code affects a secondary host, use only the narrowest practical compile or contract-parity check, and report it as secondary compatibility evidence rather than milestone validation.
- A secondary-host failure blocks PSP work only when it exposes a defect in shared code used by PSP; otherwise record it as an accepted limitation and keep PSP progress moving.

## Repository map

- `crates/openstrike-core/` — portable Rust simulation shared by targets.
- `crates/openstrike/` — native macOS/headless host.
- `crates/openstrike-psp/` — PSP host and EBOOT package.
- `game/` — PocketJS rules, SDK, menus, and HUD.
- `scripts/` — UI, PSP, emulator, hardware, and platform workflows.
- `test/` — upstream tooling tests and emulator goldens.
- `docs/project/` — downstream plans, decisions, setup, and baseline records.
- `vendor/` — pinned upstream submodules. Treat as read-only unless a dependency update or upstream fix is explicitly requested.
- `local/`, `dist/`, `out/`, `target/`, `.pocket/` — ignored local/generated data. Never force-add them.

The temporary local BSP/WAD data under `local/openstrike-maps/` exists only to exercise the upstream baseline. Do not build project features around it.

## macOS environment

Use the full setup guide in `docs/project/macos-setup.md`. Common shell setup:

```sh
export PATH="/opt/homebrew/bin:$HOME/.bun/bin:$HOME/.cargo/bin:/opt/homebrew/opt/llvm/bin:$PATH"
export POCKETJS_LLVM_BIN="/opt/homebrew/opt/llvm/bin"
export PPSSPP_HEADLESS="/absolute/path/to/ppsspp/Build/PPSSPPHeadless"
export OPENSTRIKE_MAPS="$PWD/local/openstrike-maps"
```

Do not assume these variables survive into a new terminal or agent process. Detect paths or report missing prerequisites honestly.

## Build and test

Bootstrap a clean checkout:

```sh
git submodule update --init --recursive
bun run setup
bun run bootstrap
```

Use the narrowest relevant check first:

```sh
# PocketJS contracts
bun run typecheck
bun run check:platforms

# Rust simulation
cargo test --release -p openstrike-core

# Native build
bun run build:ui
cargo build --release -p openstrike

# Deterministic native scenarios
target/release/openstrike --maps-dir "$OPENSTRIKE_MAPS" --script walk --screenshot out/walk
target/release/openstrike --maps-dir "$OPENSTRIKE_MAPS" --script round --screenshot out/round

# Automated native interactive smoke
target/release/openstrike --maps-dir "$OPENSTRIKE_MAPS" --auto-quit 5

# PSP package and emulator journey
bun scripts/psp.ts --release --package
bun scripts/e2e-psp.ts
```

The installed PPSSPP revision may differ from the historical upstream golden revision. Capture liveness is useful; do not overwrite goldens merely to make a mismatch disappear.

## Verification rules

- Every gameplay feature needs a deterministic headless path when practical.
- Prefer fixed seeds, bounded frames, structured state, scripted input, and exact-frame screenshots.
- Use native macOS for interaction and feel.
- Use the browser host only for isolated PocketJS UI/component work.
- Use PPSSPP for PSP compatibility gates, not the inner edit loop.
- Use physical PSP tests for milestone performance, memory, controls, and hardware claims.
- Never present emulator results as physical PSP results.
- Record commands, measurements, failures, and accepted limitations. Do not claim unrun verification.

## Change discipline

- Keep changes small and independently testable.
- Inspect the existing implementation before adding a new abstraction.
- Preserve upstream attribution, licences, and Git history.
- Do not edit generated output when the source generator can be changed instead.
- Do not update submodule revisions incidentally.
- Do not mix dependency upgrades, architecture changes, gameplay changes, and content work in one change.
- Preserve unrelated user changes in a dirty worktree.
- Run `git diff --check` before handoff.
- Update the roadmap and current milestone contract when status or the immediate next action changes materially.

## Completion report

At handoff, state:

- what changed;
- what was verified and with which command;
- what was not verified;
- any generated/local files created;
- any remaining blocker or accepted limitation.
