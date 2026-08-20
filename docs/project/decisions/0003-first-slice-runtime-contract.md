# ADR 0003: First-Slice Runtime Contract

- Status: Accepted
- Date: 2026-08-20

## Context

The selected OpenStrike baseline already has a useful native/guest boundary:
Rust owns fixed-step simulation and publishes a state object plus an event
array, while PocketJS owns round transitions and queues native commands. The
desktop and PSP hosts implement equivalent vocabulary.

The baseline is not yet the final downstream contract. Its state has no schema
version or tick identity, target identifiers are vector indices, and each
PocketJS intent calls a native operation separately. Headless scripts also use
hand-authored input and human-readable assertions rather than reusable input
tapes and structured results.

The first project-authored slice needs one precise contract before gameplay
implementation begins.

## Decision

### Ownership

Rust is authoritative for fixed-tick time, player transform and collision,
weapon timing and ammunition, hitscan queries, target identity and health, and
event ordering. PocketJS is authoritative for round policy, score, tuning,
scenario policy, and HUD presentation. PocketJS mirrors published facts; it
does not mutate native state directly.

Configuration is policy owned by PocketJS but validated and applied by Rust at
a tick boundary. Rejected configuration or commands must fail visibly in test
output rather than being silently clamped.

### Tick exchange

Use one versioned exchange per fixed tick:

1. Rust advances tick `N` from normalized input.
2. Rust publishes snapshot `N` and the ordered events produced by tick `N`.
3. PocketJS evaluates rules and UI once against that immutable exchange.
4. PocketJS returns one ordered command batch tagged for application after
   tick `N`.
5. Rust validates and applies that batch before tick `N + 1`.

Commands never affect the snapshot that produced them. Events are delivered
at most once. Both hosts must preserve the same field names, units, ordering,
and validation behavior.

The existing `strike.__dispatch(state, events)` and individual native intent
operations are migration surfaces, not the final versioned batch contract.
Milestone 4 may retain the public TypeScript convenience methods, but they must
append to a guest-side queue that is transferred to Rust once after the guest
turn.

### Stable identity and determinism

Entities visible across the boundary use opaque, non-recycled IDs within a
scenario run. Array positions are not IDs. Simulation time is represented by
an integer tick at 64 Hz; floating-point seconds are presentation-only. A
scenario declares its seed, map, spawn, tick limit, input tape, assertions, and
capture ticks. The run result repeats those effective inputs.

The canonical machine-readable representation is JSON with a required schema
version. Tests compare explicit fields, not debug-formatted Rust output.

### Error handling and limits

Unknown schema versions, commands, event types, input actions, or required
fields are errors. A batch over its declared limit fails the scenario and logs
the offending tick. Production builds may reject excess policy work without
crashing, but tests must not silently drop it.

The first-slice limits and schemas are specified in
`docs/project/first-vertical-slice.md`.

## Rationale

- Tick IDs and ordered batches make causality testable.
- Stable IDs keep guest policy independent of Rust container layout.
- Integer time and recorded normalized input remove platform clock and device
  mapping from deterministic tests.
- One batched return preserves the intended guest/native cost model as entity
  counts grow.
- Strict versioning and errors prevent desktop and PSP surfaces from drifting
  silently.

## Consequences

- The desktop and PSP host implementations must share serialization semantics,
  preferably through portable contract types and focused parity tests.
- Existing PocketJS code can keep ergonomic methods such as `setPhase`, but
  their implementation changes from immediate native calls to local queuing.
- The first slice needs stable target IDs and a versioned scenario runner before
  its acceptance scenarios can be considered complete.
- High-frequency transforms remain native and are deliberately absent from the
  first-slice guest snapshot.

## Verification

The decision is implemented when contract tests prove:

- desktop and PSP-facing encoders expose the same versioned schema;
- one tick produces one facts exchange and at most one command batch;
- events and commands retain deterministic order;
- commands produced from tick `N` cannot change snapshot `N`;
- invalid or oversized payloads produce structured failures; and
- replaying the same scenario and tape produces identical selected state and
  capture output on the deterministic host.
