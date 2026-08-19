# Upstream Audit: 2026-08-18

This document records the source, licence, structure, and revision audit used to select the project's initial OpenStrike foundation. It does not claim that any target has been built or run locally; those checks belong to the unmodified-baseline milestone.

## Selection summary

| Component | Repository | Selected revision | Relationship |
| --- | --- | --- | --- |
| OpenStrike | `https://github.com/pocket-stack/open-strike.git` | `fcfe93e9b2821524d6f6e834d15939cb18bc6e3d` | Direct project baseline |
| PocketJS and Pocket3D | `https://github.com/pocket-stack/pocketjs.git` | `5bfaff7091e63a1cd93fe46fd5a4f8b61b46b335` | OpenStrike's pinned `vendor/pocketjs` submodule |
| quickjs-rs | `https://github.com/pocket-stack/quickjs-rs.git` | `0fc946fb670c0c29bc0135f510bcb0f595415a61` | OpenStrike's pinned `vendor/quickjs-rs` submodule |
| rust-psp | `https://github.com/pocket-stack/rust-psp.git` | `2cbaf8c9bc72569c76240a1d9743de10731e5f6b` | OpenStrike's pinned `vendor/rust-psp` submodule |

The revisions above are full immutable commit identifiers. The selected OpenStrike commit was the tip of its `main` branch when queried on 2026-08-18. It is 13 commits after the repository's `v0.1.0` tag and includes later PSP toolchain, menu, HUD performance, Vita, and Symbian work.

PocketJS had moved to `e0b37e20dbfaeb10a4fa2aac6564c75023afe53e` on `main` by the audit date. OpenStrike's selected PocketJS commit was an ancestor 83 commits behind that tip. Those intervening commits contain material runtime and Pocket3D changes. The baseline must therefore use OpenStrike's submodule revision rather than independently advancing PocketJS.

## Revision evidence

### OpenStrike

- Commit: `fcfe93e9b2821524d6f6e834d15939cb18bc6e3d`
- Commit date: 2026-07-26
- Subject: `feat(symbian): port full OpenStrike to Nokia E7 (#14)`
- Latest release tag observed: `v0.1.0`
- `v0.1.0` commit: `aeee7b2021a805f8a5c3218c1dfde81364aa57d1`

### PocketJS/Pocket3D

- OpenStrike pin: `5bfaff7091e63a1cd93fe46fd5a4f8b61b46b335`
- Commit date: 2026-07-26
- Subject: `fix(symbian): use E7 physical key input`
- Description relative to tags: `v0.7.0-12-g5bfaff7`
- Audit-date `main`: `e0b37e20dbfaeb10a4fa2aac6564c75023afe53e`
- Audit-date `main` description: `v0.10.1-5-ge0b37e2`

### Other OpenStrike submodules

- quickjs-rs `0fc946fb670c0c29bc0135f510bcb0f595415a61`, dated 2026-07-13.
- rust-psp `2cbaf8c9bc72569c76240a1d9743de10731e5f6b`, dated 2026-07-02.

## Licence and provenance findings

This is an engineering audit, not legal advice. Distribution should preserve the notices present in the selected source and should be checked again before release.

### Source licences

- OpenStrike is MIT licensed, copyright 2026 pocket-stack contributors.
- PocketJS, including Pocket3D in the same repository, is MIT licensed, copyright 2026 Yifeng "Evan" Wang.
- quickjs-rs is MIT licensed, copyright 2019 Christoph Herzog. Its embedded QuickJS source carries MIT notices for Fabrice Bellard and Charlie Gordon.
- rust-psp is MIT licensed, copyright 2020 Marko Mijalkovic. Its licence also reproduces the three-clause BSD notice for PSPSDK material used as a reference; source and binary redistribution conditions must be preserved.

MIT permits use and modification, including commercial distribution, provided its copyright and permission notice remain with copies or substantial portions. The project should retain upstream licence files and add a third-party notices file before distributing binaries.

The project itself is intended to be distributed free of charge as community-led open-source homebrew. That policy does not change upstream MIT rights. If the project adopts MIT or another Open Source Initiative-compatible licence, downstream commercial redistribution generally cannot be prohibited merely because the maintainers do not sell the game themselves.

### Content provenance

- OpenStrike does not track Counter-Strike BSP or WAD data. Its ignore rules exclude `assets/maps/` and `assets/wads/`, and its documentation requires users to provide that data locally.
- Generated map-bearing artifacts must not be committed or distributed by this project.
- OpenStrike tracks `assets/models/Soldier.glb`. Its credits identify a three.js example derived from Adobe Mixamo's "Vanguard" character and limit it to use as an embedded application asset rather than standalone redistribution.
- The soldier model is acceptable only for reproducing the unmodified upstream baseline. It is not original project content and must be removed or replaced before the first project-authored playable slice or distributable build.
- OpenStrike's screenshots and emulator goldens depict upstream test content. Keep them as upstream verification fixtures only while needed; do not treat them as project branding or new content.

## Source-tree findings

The OpenStrike source tree already supplies the structure that the scaffold anticipated:

| Upstream path | Purpose | Project treatment |
| --- | --- | --- |
| `crates/openstrike-core/` | Shared Rust simulation | Extend and progressively rename project-specific vocabulary |
| `crates/openstrike/` | Desktop and headless host | Preserve as the primary edit and interactive test loop |
| `crates/openstrike-psp/` | PSP EBOOT host | Preserve as the PSP target |
| `crates/openstrike-vita/` | Vita host | Retain upstream initially; PSP remains the project priority |
| `crates/openstrike-symbian/` | Symbian host | Retain upstream initially; do not expand project scope around it |
| `game/` | TypeScript rules, SDK, menus, and HUD | Replace product vocabulary and content incrementally |
| `scripts/` | Build, packaging, emulator, and hardware automation | Use instead of creating duplicate helpers in `tools/` |
| `test/` | Platform tests and emulator goldens | Use upstream's singular directory rather than the scaffold's `tests/` |
| `vendor/pocketjs` | Pinned PocketJS/Pocket3D engine | Preserve as a submodule and upgrade deliberately |
| `vendor/quickjs-rs` | Pinned QuickJS Rust integration | Preserve as a submodule |
| `vendor/rust-psp` | Pinned Rust PSP support | Preserve as a submodule |

### Reconciliation decisions

- Use the OpenStrike workspace as the repository root; do not place it beneath the current scaffold.
- Keep `AGENTS.md` at the root.
- Move project-owned planning material under `docs/project/` during source integration, with decisions under `docs/project/decisions/` and research under `docs/project/research/`. This avoids confusing project planning with upstream visual fixtures.
- Merge the project overview and upstream attribution/build context into the root `README.md` rather than keeping two competing root introductions.
- Adopt upstream's `scripts/` and `test/` conventions. Remove provisional empty `tools/` and `tests/` directories when they no longer serve a distinct purpose.
- Preserve original-asset directories only when the first original test content is introduced; do not force empty scaffold paths onto the upstream tree.

## Integration recommendation

Use a direct OpenStrike downstream rather than nesting OpenStrike as a submodule or vendored archive.

The native FPS surface will change substantially, so a wrapper repository would create awkward cross-repository edits and obscure which source produces the game. A direct downstream preserves one build graph, keeps reusable changes separable, and retains OpenStrike's tested relationship with PocketJS.

For this already-initialized repository:

1. Add `https://github.com/pocket-stack/open-strike.git` as a read-only remote named `openstrike`.
2. Integrate commit `fcfe93e9b2821524d6f6e834d15939cb18bc6e3d` while preserving both existing project history and OpenStrike history.
3. Resolve root documentation and ignore-file conflicts according to the reconciliation decisions above.
4. Initialize submodules recursively and verify that all three gitlinks match the revisions in this audit.
5. Tag or otherwise record the imported baseline before making product changes.

Do not upgrade PocketJS during integration. Engine upgrades should happen later as isolated changes with native, headless, PPSSPP, and PSP compatibility evidence.

## Documentation/source mismatches and cautions

- OpenStrike's README describes a mature set of platform and hardware results, but none of those results has been independently reproduced in this repository yet. Treat them as upstream claims until Milestone 2 records local evidence.
- The selected Pocket3D README calls PVS culling a v0.1 non-goal, while the selected OpenStrike README describes cooked PVS behavior. The implementation and cooker output must be inspected during baseline work before relying on either statement.
- The Pocket3D README's displayed layout contains a duplicated `engine/pocket3d` segment under its tree example; the actual path in the selected repository is `engine/pocket3d/crates/pocket3d`.
- OpenStrike headless examples require locally supplied proprietary maps. The project must not commit those maps or their cooked derivatives. Milestone 2 should separate an exact upstream reproduction using private local inputs from a future original, redistributable test map.
- OpenStrike's `main` branch was newer than its `v0.1.0` release tag and PocketJS was moving rapidly. All future upstream comparisons must use full commit identifiers, not branch names or loose version descriptions.

## Milestone conclusion

The selected source and integration strategy satisfy the upstream-selection gate. The next milestone is to integrate the selected OpenStrike tree without product modifications and reproduce its native, headless, PSP, and PPSSPP baseline.
