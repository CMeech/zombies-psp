---
name: psp-validation
description: Validate PSP packaging, PPSSPP compatibility, emulator captures, or physical PSP behavior for this repository. Use for PSP build verification, emulator journeys, golden review, hardware measurements, and PSP evidence reports; do not use for ordinary native inner-loop tests.
---

# PSP validation

Treat PSP as the primary product target while choosing the narrowest validation that answers the request.

Before acting, read the PSP-relevant sections of:

- `AGENTS.md`
- `docs/project/development-loop.md`
- `docs/project/macos-setup.md`
- the current milestone in `docs/project/ROADMAP.md`
- `docs/project/first-vertical-slice.md` when validating Milestone 4

## Workflow

1. Inspect the existing PSP scripts before adding commands or helpers. Use `scripts/psp.ts`, `scripts/e2e-psp.ts`, and `scripts/hw.ts` as their names and current implementations warrant.
2. Check required paths and tools rather than assuming environment variables survived from another shell. Report a missing prerequisite precisely.
3. Build or package before emulator or hardware validation when the relevant inputs changed.
4. For PPSSPP, distinguish package/boot failure, journey or capture-liveness failure, byte-exact golden mismatch, and successful compatibility evidence.
5. Never update a golden merely to remove a mismatch. First identify whether the visual change is intended, confirm the emulator revision recorded in `test/goldens-psp/PPSSPP-COMMIT.txt`, and review the produced captures. Updating checked-in goldens requires explicit user intent or an already-authorized intentional visual change.
6. Treat PPSSPP as a compatibility gate, not performance or memory evidence for real hardware.
7. Attribute physical PSP claims only to an actual hardware run. Record the PSP model, command or workflow, observed behavior, measurements, and limitations.

## Handoff

State exactly which commands ran, whether packaging, boot, journey, captures, and comparisons passed independently, where local or generated artifacts were written, and what was not tested. Keep emulator and physical-hardware conclusions separate.
