# ADR 0002: Pin OpenStrike and Preserve Its Engine Submodules

- Status: Accepted
- Date: 2026-08-18

## Context

The project needs an exact, reproducible upstream foundation before implementation begins. OpenStrike already provides the specialized FPS runtime, desktop/headless host, PSP host, game bundle, automation, and a pinned PocketJS/Pocket3D engine. PocketJS continues to change rapidly, so selecting OpenStrike and PocketJS independently would discard the compatibility relationship tested by upstream.

This repository already has a small, unrelated planning history. The selected integration approach must preserve both that work and upstream attribution.

## Decision

Use OpenStrike commit `fcfe93e9b2821524d6f6e834d15939cb18bc6e3d` as the initial product source baseline.

Preserve the submodule revisions recorded by that commit:

- PocketJS/Pocket3D: `5bfaff7091e63a1cd93fe46fd5a4f8b61b46b335`
- quickjs-rs: `0fc946fb670c0c29bc0135f510bcb0f595415a61`
- rust-psp: `2cbaf8c9bc72569c76240a1d9743de10731e5f6b`

Make this repository a direct OpenStrike downstream:

- retain the current repository as the project origin;
- add the official OpenStrike repository as a remote named `openstrike`;
- integrate the selected OpenStrike commit while retaining upstream history;
- keep PocketJS and the PSP dependencies as pinned submodules; and
- place project planning documents under `docs/project/` when reconciling the source trees.

Do not make OpenStrike a submodule of this project, do not copy it as an untracked source archive, and do not advance PocketJS independently during the baseline import.

## Rationale

- Most planned native work changes the FPS runtime itself, making a thin wrapper an artificial boundary.
- The direct downstream keeps native, guest, asset, test, and packaging changes in one build graph.
- Preserving OpenStrike history makes attribution and future upstream comparisons clearer.
- Preserving the recorded submodule commits avoids beginning with an untested engine combination.
- Full immutable commit identifiers make the selection reproducible even when upstream branches advance.

## Consequences

### Positive

- Existing OpenStrike build and verification paths remain available.
- Reusable changes can be kept as focused commits for possible upstream contribution.
- PocketJS/Pocket3D compatibility begins from the combination selected by OpenStrike.
- Upstream licences and authorship remain visible in history and the working tree.

### Costs and risks

- Integrating two existing histories requires a deliberate one-time reconciliation.
- Project-specific renaming will touch OpenStrike's `strike` and `openstrike` vocabulary over time.
- Upstream contains a third-party soldier model and verification fixtures that cannot become final project content.
- Following later OpenStrike or PocketJS changes will require explicit compatibility updates rather than automatic branch tracking.

## Verification

The integration is correct when:

- the OpenStrike source tree is present at the repository root;
- the `openstrike` remote points to the official repository;
- upstream history remains reachable;
- each submodule gitlink matches the commit listed above;
- project planning documents are retained;
- upstream licence and asset-credit files are retained; and
- no project gameplay changes are included in the baseline integration.

## Follow-up

Milestone 2 will integrate the tree and reproduce the unmodified baseline. It must distinguish upstream claims from locally verified native, headless, PPSSPP, and hardware results.
