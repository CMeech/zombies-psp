# Fast Development Loop

## Default loop

For most changes:

1. Edit Rust, TypeScript, JSX, or data.
2. Run the narrowest relevant test.
3. Run a deterministic headless scenario.
4. Inspect state output or a generated screenshot.
5. Launch the native macOS build only when interaction or feel matters.

The PSP toolchain, PPSSPP, and physical hardware are validation targets rather than the default inner loop.

## Loop by change type

| Change | Primary feedback path | Secondary gate |
| --- | --- | --- |
| Rules or tuning | Headless QuickJS scenario | Native macOS run |
| HUD or menu | PocketJS browser host/DevTools | Native macOS composite |
| Core simulation | Rust tests and headless scenario | Native macOS run |
| Rendering | Headless screenshot | Native macOS window |
| PSP backend | PPSSPP scripted journey | Physical PSP |
| Performance | Native counters | PSP-1000 hardware benchmark |

## Automation requirements

- Provide named scenarios such as boot, movement, spawn, round transition, and restart.
- Give every scenario a fixed seed and bounded frame count.
- Support `--auto-quit` or an equivalent completion condition.
- Emit machine-readable state summaries.
- Capture a requested frame without opening a window.
- Treat a recorded input tape as a reusable regression test.
- Add a single command that rebuilds and relaunches only when an interactive target is needed.

## Restart strategy

- PocketJS-only changes should use guest hot reload where the host supports it.
- Native desktop changes may restart the process, but the relaunch should be scripted and return directly to a chosen scenario.
- PPSSPP should boot directly into the EBOOT and replay a stored input journey.
- PSPLINK should rebuild and relaunch the hardware build from one command.

The goal is not literally zero restarts. The goal is zero manual navigation after a restart.

## Codex-friendly acceptance contract

A feature is easiest to maintain when Codex can:

- build it from the terminal;
- start it with a named scenario;
- provide deterministic input;
- receive logs and structured state;
- request a screenshot at an exact frame;
- compare the result with a checked-in expectation;
- terminate the process automatically.

Visual desktop interaction can remain an additional exploratory tool, not the only way to verify behavior.
