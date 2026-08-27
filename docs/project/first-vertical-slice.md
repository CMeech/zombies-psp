# Milestone 3: First Vertical Slice Contract

- Status: Accepted for implementation
- Date: 2026-08-20
- Architecture: [ADR 0003](decisions/0003-first-slice-runtime-contract.md)

## Outcome

The first slice is a deliberately small, original, single-room experience. A
player starts in a safe position, moves through the room with world collision,
fires one hitscan weapon at one stationary target, sees damage and ammunition
feedback, destroys the target, completes the round, and restarts into the same
initial state without manual navigation.

The slice proves the downstream content path, native/PocketJS boundary,
deterministic test loop, and PSP-1000 measurement path. It is not a commitment
to final controls, theme, enemy behavior, weapon roster, economy, or tuning.

## Scope boundaries

Included:

- one original BSP test room with simple original textures;
- walking, looking, world collision, and a fixed player spawn;
- one stationary damageable billboard target with a fixed spawn;
- one hitscan weapon with ammunition and a fixed fire interval;
- body damage, target destruction, round completion, and restart;
- minimal crosshair, health, ammunition, hit, phase, and completion feedback;
- deterministic headless state assertions and exact-tick captures; and
- native, PPSSPP, and physical-PSP validation responsibilities.

Excluded:

- moving enemies, navigation, enemy attacks, jumping as a slice requirement,
  economy, purchases, upgrades, multiple weapons, multiple rooms, final art,
  final controls, multiplayer, save data, and production balancing.

## Functional acceptance criteria

### Boot and round flow

1. Scenario `slice.boot` loads the original room, uses seed `1`, selects the
   declared player and target spawns, and publishes phase `starting` at tick 0.
2. PocketJS advances the round from `starting` to `live` after a scenario-set
   countdown. Rust freezes combat input while the phase gate is closed.
3. Destroying the only target produces one `targetDestroyed` event and causes
   PocketJS to request phase `won` and increment the win count once.
4. After the scenario-set end pause, PocketJS requests a reset. The next round
   restores the player, weapon, target, score-independent round state, and
   deterministic RNG state defined by the scenario.
5. No transition, score change, or reset is duplicated if the same fact remains
   visible for multiple ticks.

### Movement and collision

1. Scenario `slice.movement` settles the player on the floor, holds normalized
   forward input for 128 ticks, and shows positive horizontal displacement.
2. A scripted movement into each representative wall never places the player
   eye or collision hull in solid space.
3. Replaying the scenario produces the same selected player position and
   velocity within the checked-in numeric tolerance. Assertions use quantized
   values; screenshots do not substitute for collision assertions.
4. Interactive keyboard/mouse and PSP controls map into the same normalized
   `SimInput`; their final bindings are not decided by this slice.

### Hitscan, damage, and ammunition

1. Scenario `slice.hit` aims at the target from a declared transform and presses
   fire for one tick. Exactly one shot is accepted, ammunition decreases by one,
   and an ordered `shotFired`, `targetHit` sequence is emitted for that tick.
2. A ray that terminates at room geometry emits `shotFired` but no `targetHit`.
3. Hitscan and damage run in Rust. PocketJS may configure validated weapon
   values but performs no ray query and owns no target health.
4. Repeated accepted shots respect the configured tick-based fire interval.
   When target health reaches zero, one hit is marked fatal, health clamps to
   zero, and later shots cannot damage that target before reset.
5. Empty ammunition cannot fire or produce hit events. Reload behavior is
   retained if already supported, but is not required for slice acceptance.

### HUD

1. The HUD shows crosshair, health, magazine ammunition, and current phase at
   480×272 without covering the target at the scripted aim transform.
2. An accepted hit produces visible hit feedback; target destruction and round
   completion produce visible completion feedback.
3. Gameplay HUD nodes are mounted at boot. Per-tick updates preserve the
   existing change-gated hot path and do not require structural UI rebuilds.
4. Headless captures include the composed native scene and shipped PocketJS
   HUD, not a test-only overlay.

## State ownership

| State | Authority | Guest visibility or intent |
| --- | --- | --- |
| Tick, seed, phase gate | Rust for clock/gate; PocketJS for phase policy | Snapshot; `setPhase` intent |
| Player transform, velocity, collision, health | Rust | Summary facts only; no transform mutation |
| Input sampling and normalization | Platform host | Recorded normalized input enters Rust |
| Weapon timing, ammo, ray query | Rust | Snapshot plus shot/hit events; validated configuration intent |
| Target ID, transform, health, alive state | Rust | Aggregate snapshot plus target events |
| Round countdown, win rule, reset timing, score | PocketJS policy | Command batch applied by Rust |
| HUD and presentation state | PocketJS | Derived from snapshot and events |
| Map geometry and cooked assets | Rust/Pocket3D | Selected by scenario or host lifecycle intent |

PocketJS must not receive per-frame target transforms for this stationary
slice. Future AI remains native and should use events or bounded summaries
rather than per-entity calls.

## Version 1 tick contract

The notation below defines field names and semantics, not a requirement to
allocate JSON strings in the runtime. Hosts may use native QuickJS objects or a
packed representation if their observable contract matches.

### Facts exchange

```ts
interface SliceFactsV1 {
  schema: 1;
  tick: number;                 // integer, 60 Hz, state after this tick
  seed: number;                 // effective scenario seed
  phase: "starting" | "live" | "won" | "lost";
  player: { hp: number; alive: boolean; speedQ: number };
  weapon: {
    ammo: number;
    reserve: number;
    reloading: boolean;
    reloadTicksRemaining: number;
  };
  targets: { alive: number; total: number };
  score: { wins: number; losses: number };
}
```

`speedQ` is horizontal world-units per second rounded to the scenario result's
declared precision. Runtime presentation may use an unquantized value. Tick 0
is the initialized state before the first input-bearing simulation step.

### 60 Hz baseline migration

The imported desktop/headless baseline previously advanced at 64 Hz while the
PSP host advanced once per 60 Hz display frame. Milestone 4 standardizes all
targets and scenario data on 60 fixed ticks per second. This is a deliberate
behavioral migration, not a formatting-only change.

Implementation and review must account for changes in tick-indexed positions,
collision and landing ticks, weapon/reload gates, animation and effect phases,
RNG consumption, bot timing, round completion, and exact-frame captures. Express
new gameplay durations as integer tick counts with an explicit rounding policy;
do not rely on repeated floating-point accumulation to make boundary decisions.

Existing assertions may be revised only when the new 60 Hz result is understood
and remains within the intended behavior. Do not overwrite native or PPSSPP
goldens merely because their pixels changed: first classify the difference as
an expected cadence correction or a regression, record the evidence, and keep
emulator revision differences separate. PSP already ran at 60 Hz, but HUD decay
timing previously contained a 64 Hz assumption, so transient capture frames can
still change.

### Events

```ts
type SliceEventV1 =
  | { type: "shotFired"; weaponId: number; ammo: number }
  | { type: "targetHit"; targetId: number; damage: number; hp: number; fatal: boolean }
  | { type: "targetDestroyed"; targetId: number }
  | { type: "playerDamaged"; amount: number; hp: number }
  | { type: "playerDied" }
  | { type: "roundReset"; round: number };
```

Events are ordered by their production in Rust. For a fatal shot the order is
`shotFired`, `targetHit`, then `targetDestroyed`. IDs are unsigned opaque IDs
that are not reused during a scenario run.

### Commands

```ts
type SliceCommandV1 =
  | { type: "setPhase"; phase: "starting" | "live" | "won" | "lost" }
  | { type: "resetRound" }
  | { type: "addWin" }
  | { type: "addLoss" }
  | { type: "configureWeapon"; config: WeaponConfigV1 }
  | { type: "configureTarget"; config: TargetConfigV1 };

interface SliceCommandBatchV1 {
  schema: 1;
  afterTick: number;
  commands: SliceCommandV1[];
}
```

Configuration values use integer ticks for simulation timing. Rust validates
finite numeric ranges, ammunition capacity, positive damage, and target health.
The initial configuration batch is applied before tick 0. Runtime rule commands
are applied after their tagged tick in array order.

For this slice the facts exchange has at most 24 scalar leaves, the event batch
at most 16 entries per tick, and the command batch at most 8 entries per tick.
Exceeding a limit is a structured scenario failure. The normal quiet tick has
no events and no commands.

## Deterministic scenario contract

Scenario definitions are checked-in data with these required fields:

```json
{
  "schema": 1,
  "name": "slice.hit",
  "map": "slice_test_room",
  "seed": 1,
  "tickRate": 60,
  "maxTicks": 256,
  "playerSpawn": "player_start",
  "inputTape": "test/input/slice-hit.json",
  "assertions": [],
  "captures": [{ "tick": 96, "name": "hit" }]
}
```

The runner accepts `--scenario NAME`, with optional `--seed`, `--max-ticks`,
`--state-out PATH`, and `--capture-dir PATH` overrides. An override is echoed
in the result so it cannot masquerade as the canonical scenario.

Scenarios that assert only simulation state must run without creating a GPU
adapter, renderer, window, or offscreen target. The runner initializes graphics
only when the scenario declares a capture or rendering assertion. Movement,
collision, hitscan, damage, event ordering, and round-flow regressions therefore
remain runnable in CPU-only CI and sandboxed environments; visual scenarios
continue to exercise the real offscreen renderer.

Input tapes contain normalized simulation actions, not platform key codes:

```json
{
  "schema": 1,
  "tickRate": 60,
  "frames": [
    { "tick": 0, "move": [0, 0], "look": [0, 0], "actions": [] },
    { "tick": 96, "actions": ["fire"] },
    { "tick": 97, "actions": [] }
  ]
}
```

Frames are change points whose values remain active until the next frame.
Ticks must be strictly increasing. Movement and look components are finite and
normalized to `[-1, 1]`; actions are a closed versioned vocabulary. Unknown or
duplicate fields fail validation.

The runner writes one JSON result whether it passes or fails:

```json
{
  "schema": 1,
  "scenario": "slice.hit",
  "status": "passed",
  "seed": 1,
  "tickRate": 60,
  "ticksRun": 112,
  "final": {},
  "events": {},
  "captures": [],
  "metrics": {},
  "failures": []
}
```

`final` contains only stable, documented assertion fields. Event summaries
contain counts and first/last ticks. A capture record includes tick, dimensions,
relative filename, and SHA-256. File paths and wall-clock durations are
diagnostic fields and are excluded from deterministic equality.

Required named scenarios are:

| Scenario | Required proof |
| --- | --- |
| `slice.boot` | Map/spawns load, tick 0 facts, countdown enters live |
| `slice.movement` | Settle, displacement, collision, replayed final state |
| `slice.miss` | Shot and ammo use without target damage |
| `slice.hit` | Ordered shot/hit facts and visible hit capture |
| `slice.complete` | Fatal hit, win once, end screen, automatic reset |

Canonical screenshots are 480×272 PNGs captured after simulation, guest turn,
HUD update, and composition for the named tick. PPSSPP goldens additionally pin
the emulator revision and software renderer. Goldens are updated only for an
intentional reviewed visual change; a revision mismatch is reported separately
and never fixed by overwriting expectations.

## Original room and asset path

The committed source tree will use this separation:

```text
assets-src/slice_test_room/   editable original map and texture sources
assets/provenance/            authorship/licence record for every source asset
dist/maps/                    ignored cooked .p3d output
```

The room source uses an openly documented, text-based brush-map format accepted
by a pinned open-source BSP compiler. Texture sources are original, lossless
images with power-of-two cooked dimensions and an explicit licence/provenance
record. No upstream BSP, WAD, texture, soldier model, or cooked derivative is
an input.

The reproducible command added in Milestone 4 must perform this fixed pipeline:

1. validate source paths, names, dimensions, licences, and slice budgets;
2. build the texture archive from committed original sources;
3. compile the text map with a repository-pinned tool and arguments;
4. invoke the existing `pocket3d-cook` path with fixed arguments and `--verify`;
5. emit `slice_test_room.p3d` plus a manifest of input hashes, tool revisions,
   arguments, output hash, and size; and
6. reproduce the same cooked bytes from a clean checkout on the supported host.

The compiler and texture-archive tool must be pinned by immutable revision and
checksum before their first output is accepted. Prefer a setup-downloaded tool
or separable submodule over committing platform binaries. The source format and
compiler choice are implementation details as long as this contract is met;
adding another runtime renderer or map format is not permitted.

The room contains only the geometry needed to test a floor, representative
walls/corners, an unobstructed target lane, an occluding wall for `slice.miss`,
one named player spawn, and one named target spawn. Cooked room data has a hard
limit of 1 MiB for the slice.

## Preliminary PSP-1000 budgets

These are engineering guardrails, not measured hardware claims. Milestone 4
records actual numbers and either meets them or proposes a reviewed revision.

| Resource | Slice budget | Measurement |
| --- | --- | --- |
| Display rate | 60 presented frames/s | Physical PSP bench window |
| Fixed simulation | 60 ticks/s, no dropped logical ticks in canonical run | Structured tick result |
| CPU work | average ≤ 12,000 µs; no recurring frame > 16,667 µs | Existing PSP `bench` JSONL, present wait excluded |
| GPU wait | average ≤ 4,000 µs; max reported, investigated if > 16,667 µs | Existing PSP `bench` JSONL |
| Guest dispatch + JS + UI | average combined ≤ 4,000 µs; max ≤ 8,000 µs outside boot/round transition | Existing segment counters |
| Runtime memory | ≤ 24 MiB attributable runtime high-water mark; ≥ 4 MiB measured reserve before load | Add PSP system/arena high-water counters; physical PSP |
| PocketJS arena | bump high-water ≤ 75% of configured capacity after ten resets | Existing arena stats plus long scenario |
| Cooked room | ≤ 1 MiB | Cook manifest/file size |
| Room textures | ≤ 512 KiB cooked texels for world and target; ≤ 8 textures; max 256×256 each | Cook manifest |
| Slice entities | 1 player, 1 target, ≤ 16 transient effects, ≤ 32 total live runtime entities | Structured counters |
| Facts/events/commands | ≤ 24 scalar facts, 16 events, and 8 commands per tick; one facts call and one command-batch transfer | Contract instrumentation |
| Draw submission | ≤ 64 world batches/draws plus ≤ 8 target/effect batches; triangles and visible faces always reported | PSP renderer/bench counters |

The memory measurement must include map buffer, renderer allocations, guest
arena, UI, simulation, and transient effects. EBOOT file size, build-process
resident memory, PPSSPP host memory, and native macOS memory are not substitutes
for PSP runtime memory. A physical PSP-1000 is required to accept performance
and memory claims; PPSSPP is a compatibility gate only.

## Validation matrix

| Layer | Required responsibility |
| --- | --- |
| Rust/TypeScript contract tests | Schema, validation, ordering, limits, tick causality, host parity |
| Native CPU-only | Non-visual scenarios, structured results, deterministic replay, no GPU initialization |
| Native headless GPU | Declared visual scenarios and exact-tick captures through the real renderer |
| Native interactive macOS | Movement/look feel, collision inspection, HUD readability, restart without navigation |
| PPSSPP software renderer | EBOOT boot, recorded input journey, capture liveness, project-authored goldens |
| Physical PSP-1000 | Frame/segment timing, memory/reserve, controls sanity, ten-reset stability |

Physical hardware unavailability does not block implementing the slice, but it
blocks claiming the PSP performance and memory budgets are met. Results must
state emulator and hardware evidence separately.

## Implementation order

Milestone 4 should remain a sequence of independently testable changes:

1. add shared versioned contract types, validation, and host parity tests;
2. add the scenario/result/input-tape formats and runner using a temporary
   baseline map only in local verification; keep non-visual runs CPU-only and
   initialize the offscreen GPU path only for declared captures;
3. pin the map compiler/tooling and add the original room source, provenance,
   deterministic cooker, and manifest;
4. replace the baseline bot/model dependency with the stationary original
   target and stable target IDs;
5. implement the slice events, batched command return, and acceptance scenarios;
6. adapt the existing HUD without introducing per-frame structural work;
7. pass native headless and interactive gates, then PPSSPP; and
8. collect physical PSP-1000 evidence when hardware is available.

Reusable contract, scenario, cooking, and instrumentation improvements should
be kept separable from project-specific room, target, rules, and presentation.

## Milestone 3 exit checklist

- [x] Functional slice acceptance criteria are explicit.
- [x] Rust/PocketJS state ownership and tick causality are decided.
- [x] Snapshot, events, and command-batch semantics are versioned and bounded.
- [x] Deterministic scenarios, state output, input tapes, and screenshots are defined.
- [x] Preliminary PSP-1000 budgets and measurement methods are recorded.
- [x] Original-room source/cooking/provenance requirements are defined.
- [x] Native, PPSSPP, and physical-hardware responsibilities are distinguished.
- [x] No ownership or boundary decision blocks implementation.

Milestone 4 may begin with the implementation order above. Budget changes require
recorded measurements and a documentation update rather than silent relaxation.

## Milestone 4 implementation progress

As of 2026-08-27:

- Shared `no_std` V1 facts, events, commands, limits, and validation exist in
  `openstrike-core`.
- Tick, scenario seed, phase vocabulary, quantized facts, and deterministic RNG
  reset state are native simulation state. Desktop and PSP publish the same
  nested V1 facts and contracted event vocabulary from the shared state;
  focused source-shape parity tests cover both encoders.
- The native CLI accepts strict scenario and normalized input-tape formats,
  seed/tick/capture overrides, nested assertions, event-count assertions, and
  structured pass or failure output.
- Non-visual scenarios do not create a GPU. A local-only baseline-map smoke run
  verified this through the structured `gpuInitialized: false` metric; no
  baseline map or scenario was committed.
- Declared captures lazily initialize the existing offscreen renderer and
  record their dimensions and SHA-256 in the result.
- The original room source/provenance layout, deterministic WAD3 texture
  builder, ericw-tools pin, setup command, cooker, manifest, and Pocket3D
  verification path are implemented. The sealed seven-brush room passes strict
  leak testing and VIS, contains a clear target lane plus an offset miss-lane
  occluder, and produces a 37,840-byte P3D below the 1 MiB room budget.
- Two consecutive cooks produced identical WAD and P3D SHA-256 values. The
  intermediate lit BSP differed for a reason not yet isolated, but Pocket3D
  normalized both inputs to identical shipped bytes. CPU-only boot and movement checks loaded the named player spawn,
  settled on the floor, crossed the clear lane, and stopped at the east wall
  with `gpuInitialized: false`.

The immediate next action is to replace the baseline bot/model dependency with
the stationary original target and stable target IDs. The checked-in complete
`slice.*` scenario set remains blocked on that target. Guest commands still use
the imported per-operation queue;
the V1 batched return replaces it with the slice-specific command vocabulary
when the stationary target is introduced in steps 4–5 above.
