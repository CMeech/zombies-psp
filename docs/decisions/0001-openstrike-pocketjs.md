# ADR 0001: Extend OpenStrike and PocketJS

- Status: Proposed
- Date: 2026-08-18

## Context

The project needs a true 3D PSP world, sprite-based actors, fast iteration, and PSP-1000 performance. Rebuilding the renderer, BSP pipeline, collision system, packaging, emulator tests, and hardware workflow would delay gameplay work and repeat solved platform engineering.

## Decision

Use OpenStrike as the initial game-runtime reference and Pocket3D/PocketJS as the architectural foundation.

- Reuse and enhance the native 3D substrate and PSP backend.
- Create an original specialized survival-game surface instead of permanently overloading the existing `strike` vocabulary.
- Keep the base game expressible through the same PocketJS surface available to future mods.
- Replace proprietary assumptions and assets with original project content.
- Preserve a desktop and headless implementation for every PSP-facing feature where practical.

## Consequences

### Positive

- Proven PSP renderer and asset pipeline.
- Native macOS, headless, emulator, and hardware verification paths.
- TypeScript/JS iteration for rules and UI.
- Rust performance for simulation and rendering.
- Improvements can potentially be contributed upstream when they are generic.

### Costs and risks

- OpenStrike and PocketJS are young and may change quickly.
- The project must track a pinned upstream revision.
- Pocket3D is a substrate rather than a complete editor-driven engine.
- New gameplay concepts require deliberate native surface design.
- Licence and attribution checks are required before redistribution.

## Revisit when

- the required gameplay vocabulary cannot remain small;
- the PSP build cannot meet its memory/frame budget;
- upstream changes make the maintenance burden unreasonable; or
- a more mature PocketJS game-runtime extension point becomes available.
