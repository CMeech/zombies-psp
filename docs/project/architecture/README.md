# Architecture Direction

## Runtime split

### Native Rust core

Owns state and work that must remain predictable within the frame budget:

- BSP world loading and visibility
- rendering and billboard batching
- collision and character movement
- enemy transforms and navigation
- spatial queries and hitscan
- animation clocks and sprite-frame selection
- object pools and memory budgets
- fixed-step simulation

### PocketJS guest

Owns high-level behavior that benefits from fast iteration:

- round flow and state transitions
- spawn policies and difficulty curves
- scoring and economy rules
- weapon and enemy tuning tables
- purchase and interaction policies
- HUD, menus, overlays, and debug UI
- scenario definitions for tests

### Boundary

The native core publishes one compact snapshot and an event batch per fixed tick. The guest evaluates once, updates UI and policy, and returns queued commands. Hot-path state stays native; the guest works from mirrors.

## Rendering direction

- True 3D BSP environments using Pocket3D.
- Camera-facing sprite quads for enemies, effects, and pickups.
- Screen-space weapon sprites.
- Baked lighting and cooked GPU-ready assets.
- Texture atlases and batching to minimize draw calls.
- No requirement for skeletal enemy models.

## Verification layers

1. Pure Rust and TypeScript tests for isolated behavior.
2. Headless fixed-step scenarios with state assertions.
3. Headless image captures and golden comparisons.
4. Native macOS interactive build.
5. PPSSPP scripted journeys.
6. Physical PSP-1000 performance and memory gates.

Each layer should catch a different class of failure; hardware should not be the first place normal logic errors are discovered.
