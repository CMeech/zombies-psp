# Project Roadmap

This roadmap organizes work around verifiable outcomes rather than target dates. The project remains documentation-only until the upstream selection milestone is complete.

## Guiding constraints

- Build an original round-based survival FPS for PSP homebrew.
- Distribute the project as free, community-led, open-source homebrew rather than selling it as a commercial game.
- Target PSP-1000 memory and performance limits first.
- Extend OpenStrike, Pocket3D, and PocketJS rather than creating a second renderer.
- Keep frame-critical simulation and rendering in Rust.
- Keep rules, tuning, HUD, menus, and content configuration in PocketJS where practical.
- Cross the guest/native boundary once per fixed tick with batched facts and queued intent.
- Preserve deterministic headless verification alongside native, emulator, and hardware paths.
- Use only original or appropriately licensed branding, environments, characters, sprites, audio, and other assets.

## Milestone 0: Repository foundation

**Status:** Complete

Establish a clean planning repository without committing to an implementation layout before upstream is inspected.

### Deliverables

- Repository working agreement and product boundaries.
- Project handoff, architecture direction, and development-loop requirements.
- Initial architecture decision record.
- Git ignore rules, text normalization, and editor conventions.
- Provisional directories for documentation, game code, assets, tests, and tools.

### Exit criteria

- The repository purpose and constraints are understandable from `README.md`, `AGENTS.md`, and `docs/project/PROJECT_HANDOFF.md`.
- Generated files, local secrets, build products, and editor metadata are excluded from version control.
- No implementation code or third-party game assets have been introduced.

## Milestone 1: Select and pin upstream

**Status:** Complete

Determine the exact foundation on which development will begin.

### Deliverables

- Inspect the current OpenStrike, PocketJS, and Pocket3D source trees and documentation.
- Review upstream licences, notices, attribution requirements, and asset provenance.
- Record exact upstream repository URLs and commit revisions.
- Decide whether to fork OpenStrike directly or maintain a thin repository that pins it.
- Compare this scaffold with the selected upstream workspace layout.
- Record the integration decision and rationale in `docs/project/decisions/`.
- Accept, revise, or replace ADR 0001 based on the findings.

### Exit criteria

- Exact upstream revisions are recorded and reproducibly obtainable.
- Licence and attribution obligations are understood and documented.
- The repository integration strategy is decided.
- The intended location of the existing planning documents is known.
- Any divergence between upstream documentation and source is recorded.

### Development gate

Do not introduce product implementation code before this milestone is complete.

## Milestone 2: Reproduce the unmodified baseline

**Status:** Current — partially verified; see `baseline-2026-08-18.md` for local map, PSP toolchain, and PPSSPP blockers

Prove that the selected upstream foundation works before changing it.

### Deliverables

- Document required toolchains and exact version information.
- Build unmodified OpenStrike for native macOS.
- Run an existing deterministic headless scenario.
- Launch the existing native interactive build.
- Build the PSP EBOOT.
- Run the existing PPSSPP journey or closest documented equivalent.
- Record exact build, run, test, packaging, and relaunch commands.
- Record baseline build time, relaunch time, frame time, and memory usage where measurable.
- Record failures, workarounds, and unsupported claims without presenting them as verified.

### Exit criteria

- A clean checkout can reproduce the documented native and headless baseline.
- The PSP build succeeds with a documented toolchain, or a specific blocker is recorded.
- The PPSSPP path succeeds, or a specific blocker is recorded.
- Baseline measurements and known limitations are checked into the repository.
- No project gameplay changes are mixed into the baseline.

## Milestone 3: Define the first vertical slice

**Status:** Blocked by Milestone 2

Translate the product direction into the smallest measurable end-to-end experience.

### Proposed scope

- One original test room.
- Player movement and world collision.
- One stationary target or minimal enemy.
- One hitscan weapon.
- Damage, target death, and reset behavior.
- Minimal HUD feedback.
- Basic round start, completion, and restart states.

### Contract work

- Define ownership of state between Rust and PocketJS.
- Define the fixed-tick snapshot, event batch, and queued-command contracts.
- Define scenario selection, fixed seeds, bounded execution, and structured state output.
- Define input-tape and exact-frame screenshot formats.
- Establish preliminary PSP-1000 budgets for memory, frame time, entity counts, texture use, and guest/native traffic.
- Identify which improvements are generic enough to remain separable for possible upstream contribution.

### Exit criteria

- The vertical slice has explicit functional acceptance criteria.
- Every important behavior has a deterministic headless verification path where practical.
- Native macOS, PPSSPP, and physical PSP validation responsibilities are distinguished.
- Performance and memory budgets are written down, including how they will be measured.
- No unresolved ownership or boundary decision blocks implementation.

## Milestone 4: Implement the first vertical slice

**Status:** Blocked by Milestone 3

Build the smallest playable loop without expanding into a broad content backlog.

### Deliverables

- Original test-room content and its reproducible asset-cooking path.
- Player movement and collision.
- Target or minimal enemy simulation.
- Hitscan firing, damage, death, and reset behavior.
- Minimal round flow and HUD.
- Named deterministic scenarios and scripted input.
- Machine-readable state assertions and exact-frame screenshot coverage.
- Native macOS interactive verification.
- PPSSPP parity check.
- Physical PSP-1000 milestone validation when hardware is available.

### Exit criteria

- The complete slice can be built and exercised from documented commands.
- Headless scenarios terminate automatically and produce stable results.
- The native build is interactively playable.
- PSP-specific results are clearly separated into emulator-tested and hardware-tested claims.
- Measurements fit the initial budgets or deviations are documented and addressed.

## Milestone 5: Establish the core survival loop

**Status:** Blocked by Milestone 4

Expand the verified slice into a reusable round-based survival foundation.

### Scope

- Enemy spawning, navigation, attacks, damage, and cleanup.
- Round progression and deterministic difficulty inputs.
- Scoring and a minimal economy or purchase interaction.
- Data-driven weapon and enemy definitions.
- Object pools, sprite batching, and bounded entity lifetimes.
- Restart, failure, and replay behavior.
- Debug panels and performance counters.

### Exit criteria

- Multiple rounds can be completed or failed without manual state reconstruction.
- Rules and tuning can change without moving per-entity hot paths into JavaScript.
- Long-running deterministic scenarios detect leaks, pool exhaustion, and state drift.
- The loop remains within revised PSP-1000 budgets on the strongest available validation target.

## Milestone 6: Content-production foundation

**Status:** Blocked by Milestone 5

Make original content creation repeatable without committing to final production scope.

### Scope

- Documented source-to-cooked pipelines for maps, sprites, textures, audio, and configuration.
- Asset validation and licence/provenance records.
- Texture-atlas and platform-budget checks.
- A representative original environment and cohesive placeholder art direction.
- Scenario and golden-update workflows for intentional visual changes.

### Exit criteria

- A contributor can add an original asset through documented, reproducible steps.
- Cooked output is deterministic where practical.
- Invalid formats and platform-budget violations fail early.
- No proprietary assets or cooked derivatives are required by builds or tests.

## Milestone 7: Production planning

**Status:** Blocked by Milestone 6

Use evidence from the validated core to decide the size and shape of the complete game.

### Decisions to schedule here

- Final project and game title.
- Setting and visual theme.
- Final controls and accessibility options.
- Weapon, enemy, encounter, and upgrade scope.
- Economy and round formulas.
- Map count and progression structure.
- Audio and music scope.
- Save data and settings behavior.
- Distribution, release packaging, and supported hardware expectations.
- Community contribution, governance, and release-maintainer expectations.

### Exit criteria

- Product scope is matched to measured PSP limits and available production capacity.
- Major content systems have acceptance criteria and ownership.
- Release milestones distinguish feature completion, content completion, optimization, and certification-style testing.

## Milestone 8: Release readiness

**Status:** Blocked by production milestones

Prepare a stable, legally distributable homebrew release.

### Exit criteria

- Required notices, licences, attribution, and source obligations are satisfied.
- All distributed branding and content are original or properly licensed.
- Clean builds produce documented release packages.
- Deterministic regression suites and PPSSPP journeys pass.
- Physical PSP-1000 performance, memory, controls, suspend/resume, and extended-play behavior are validated.
- Known limitations and supported configurations are documented.

## Deferred decisions

The following should not block foundation work and should only be finalized when their milestone is reached:

- Final title and branding.
- Setting, story, and visual theme.
- Final controls.
- Weapon and enemy rosters.
- Economy, upgrade, and round-tuning details.
- Multiplayer.
- Final map tooling and editor workflow.
- Content quantity and release date.

## Roadmap maintenance

- Update milestone status only when its exit criteria have been checked.
- Add significant architectural decisions to `docs/project/decisions/` rather than burying them here.
- Record verified commands and measurements in dedicated baseline or test documents.
- Keep future milestones broad until evidence from the current milestone supports finer planning.
- Update `docs/project/PROJECT_HANDOFF.md` whenever the current milestone or immediate next action changes materially.
