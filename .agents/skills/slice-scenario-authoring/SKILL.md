---
name: slice-scenario-authoring
description: Create or modify deterministic first-slice scenarios, input tapes, assertions, structured results, or exact-tick captures in this repository. Use for files under test/scenarios and test/input or for scenario-runner behavior; do not use for general gameplay implementation without scenario work.
---

# Slice scenario authoring

Preserve the deterministic scenario contract while making the smallest independently testable change.

Before editing, read:

- `AGENTS.md`
- `docs/project/first-vertical-slice.md`, especially **Deterministic scenario contract**
- `docs/project/development-loop.md`
- the nearest existing examples in `test/scenarios/` and `test/input/`

Inspect the current parser and runner before inferring a schema from examples. Keep scenario definitions, input tapes, parser behavior, and documentation consistent.

## Authoring rules

- Give a canonical scenario a stable name, fixed seed, bounded tick count, and explicit map and spawn inputs required by the contract.
- Keep input tapes normalized and deterministic. Use tick-addressed actions and preserve their defined ordering.
- Prefer structured state assertions for simulation behavior. Add an exact-tick screenshot only when the scenario declares a rendering or presentation assertion.
- Keep non-visual scenarios CPU-only; do not introduce GPU initialization merely to exercise logic.
- Use stable entity IDs and contract vocabulary. Do not rely on array positions as identities or add undocumented fields.
- Reject malformed, unknown, out-of-range, or over-limit data rather than silently normalizing it, unless the accepted contract explicitly defines normalization.
- When allowing seed, tick-limit, or input overrides, ensure the result records the effective values so an override cannot masquerade as the canonical scenario.
- Do not build scenarios around ignored proprietary baseline maps or assets. Use project-authored content.

## Verification

Run the narrowest parser, contract, or scenario test first. Then run the canonical scenario through the native CPU-only or headless-GPU path appropriate to its assertions. Confirm automatic termination, structured output, and deterministic replay; for visual scenarios, inspect the requested exact-frame capture.

At handoff, identify changed scenario and input files, commands run, effective seed and tick bound when relevant, generated captures and results, and any validation layer not run.
