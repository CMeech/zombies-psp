# PocketJS Cross-Reference

Research snapshot: 2026-08-18.

## Confirmed capabilities

| Need | PocketJS/OpenStrike capability | Planned use |
| --- | --- | --- |
| Fast UI work | Browser WASM development host | HUD and menu isolation |
| Native Mac play | Pocket3D wgpu/winit desktop host | Main interactive gameplay loop |
| Automated visual checks | Headless renderer and PNG capture | Frame-specific golden tests |
| Repeatable behavior | Fixed-step deterministic simulation | Scripted scenarios and replay |
| Runtime inspection | Component tree, console, REPL, pause/step | UI and guest debugging |
| No manual hardware reset | DevTools/PSPLINK rebuild and relaunch | Periodic PSP testing |
| PSP parity | PPSSPP input journeys and frame goldens | Pre-hardware compatibility gate |
| Fast rules iteration | QuickJS guest and domain surface | Rounds, scoring, tuning, HUD |
| PSP performance | Rust cores and one guest turn per tick | Keep hot paths native |

## Important constraints

- PocketJS is not the entire OpenStrike engine.
- The full OpenStrike simulation and 3D rendering core are Rust.
- PocketJS hosts the game-rule bundle and Solid JSX HUD through QuickJS.
- PocketJS supports browser hosts, but the published OpenStrike project documents desktop, PSP, Vita, and Symbian full-game targets rather than a full browser OpenStrike target.
- Therefore, browser use should initially focus on isolated UI and guest behavior; native macOS is the authoritative full interactive development target.
- PocketJS's browser rebuild-on-change workflow is currently documented as manual rebuild plus page reload. We can add a repository-level watcher later without changing the runtime architecture.

## Sources

- https://pocketjs.dev/docs/getting-started/
- https://pocketjs.dev/docs/architecture/
- https://pocketjs.dev/docs/devtools/
- https://pocketjs.dev/docs/build-pipeline/
- https://github.com/pocket-stack/pocketjs/blob/main/docs/RUNTIMES.md
- https://github.com/pocket-stack/pocketjs/blob/main/engine/pocket3d/README.md
- https://github.com/pocket-stack/open-strike
