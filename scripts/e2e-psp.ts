// PSP e2e: deterministic PPSSPPHeadless runs of the original Milestone 4 room
// against byte-exact project goldens. Input uses the runtime's extended
// frame:mask:lx:ly capture format.
//
//   bun scripts/e2e-psp.ts            # compare against test/goldens-psp
//   UPDATE=1 bun scripts/e2e-psp.ts   # re-baseline
//
// Requires PPSSPPHeadless (PPSSPP_HEADLESS env or the repository-adjacent
// ppsspp checkout) and ImageMagick. Software renderer only; goldens are
// promised only for the commit recorded in test/goldens-psp/PPSSPP-COMMIT.txt.

import { $ } from "bun";
import { existsSync, mkdirSync, readdirSync, rmSync } from "node:fs";
import { resolve } from "node:path";

const repo = new URL("..", import.meta.url).pathname;
const home = process.env.HOME ?? "";
const goldens = `${repo}test/goldens-psp`;
const update = process.env.UPDATE === "1";

// PSP button bits.
const R = 0x200; // fire

interface Spec {
  name: string;
  // frame:mask:lx:ly entries (lx/ly optional, default 128).
  input: string;
  capStart: number;
  capN: number;
  // Frame indices (relative to capStart) to keep as goldens.
  shots: number[];
}

// Round flow: rules.ts freezes combat for 72 ticks. The room's spawn yaw aims
// directly down the clear lane at the stationary target.
const ALL_SPECS: Spec[] = [
  {
    // Live round: original world, procedural target, viewmodel, and HUD.
    name: "slice-live",
    input: "0:0",
    capStart: 88,
    capN: 2,
    shots: [0],
  },
  {
    // One accepted shot: muzzle/tracer, hit marker, and ammo decrement.
    name: "slice-hit",
    input: `0:0,75:${R},76:0`,
    capStart: 75,
    capN: 4,
    shots: [1],
  },
  {
    // Three interval-spaced hits destroy the target and show completion.
    name: "slice-complete",
    input: `0:0,75:${R},76:0,82:${R},83:0,89:${R},90:0`,
    capStart: 89,
    capN: 8,
    shots: [1, 6],
  },
];
const selectedSpec = process.env.E2E_PSP_SPEC;
const SPECS = selectedSpec ? ALL_SPECS.filter((spec) => spec.name === selectedSpec) : ALL_SPECS;
if (SPECS.length === 0) {
  console.error(`unknown E2E_PSP_SPEC '${selectedSpec}'`);
  process.exit(1);
}

const adjacentPpsspp = `${repo}../ppsspp/Build/PPSSPPHeadless`;
const legacyPpsspp = `${home}/ppsspp-src/build/PPSSPPHeadless`;
const ppsspp = process.env.PPSSPP_HEADLESS ??
  (existsSync(adjacentPpsspp) ? adjacentPpsspp : legacyPpsspp);
if (!existsSync(ppsspp)) {
  console.error(`PPSSPPHeadless not found at ${ppsspp}`);
  process.exit(1);
}

const capDir = `${home}/.ppsspp/dc_cap`;
const outDir = `${repo}out/e2e-psp`;
mkdirSync(outDir, { recursive: true });
mkdirSync(goldens, { recursive: true });

let failures = 0;
for (const spec of SPECS) {
  console.log(`\n## ${spec.name} (input: ${spec.input})`);
  console.log("# build capture EBOOT ...");
  await $`bun scripts/psp.ts --capture --map slice_test_room`
    .cwd(repo)
    .env({
      ...process.env,
      OPENSTRIKE_PSP_CAPTURE_INPUT: spec.input,
      OPENSTRIKE_PSP_CAP_START: String(spec.capStart),
      OPENSTRIKE_PSP_CAP_N: String(spec.capN),
    })
    .quiet();

  rmSync(capDir, { recursive: true, force: true });
  const eboot = `${repo}crates/openstrike-psp/target/mipsel-sony-psp/debug/EBOOT.PBP`;
  rmSync(`${repo}crates/openstrike-psp/target/mipsel-sony-psp/debug/pocketjs-dbg`, {
    recursive: true,
    force: true,
  });
  console.log("# PPSSPPHeadless (software renderer) ...");
  await $`${ppsspp} --graphics=software --timeout=180 ${eboot}`.nothrow().quiet();

  const raws = existsSync(capDir)
    ? readdirSync(capDir).filter((f) => f.endsWith(".raw")).sort()
    : [];
  if (raws.length !== spec.capN) {
    console.error(`FAIL ${spec.name}: ${raws.length}/${spec.capN} frames dumped`);
    failures++;
    continue;
  }
  console.log(`liveness: ${raws.length}/${spec.capN} frames dumped`);

  for (const shot of spec.shots) {
    const raw = `${capDir}/f${String(shot).padStart(4, "0")}.raw`;
    const png = `${outDir}/${spec.name}.f${shot}.png`;
    await $`magick -size 512x272 -depth 8 RGBA:${raw} -alpha off -crop 480x272+0+0 +repage -define png:exclude-chunks=date,time PNG24:${png}`.quiet();

    // Degenerate-frame guard: a real frame has plenty of distinct colors.
    const ident = await $`magick ${png} -format %k info:`.text();
    if (parseInt(ident.trim(), 10) < 16) {
      console.error(`FAIL ${spec.name}.f${shot}: degenerate frame (${ident.trim()} colors)`);
      failures++;
      continue;
    }

    const golden = `${goldens}/${spec.name}.f${shot}.png`;
    if (update) {
      await Bun.write(golden, Bun.file(png));
      console.log(`baseline ${spec.name}.f${shot} written`);
    } else if (!existsSync(golden)) {
      console.error(`FAIL ${spec.name}.f${shot}: golden missing (review ${png}, then run UPDATE=1 intentionally)`);
      failures++;
    } else {
      const a = Buffer.from(await Bun.file(png).arrayBuffer());
      const b = Buffer.from(await Bun.file(golden).arrayBuffer());
      if (a.equals(b)) {
        console.log(`ok ${spec.name}.f${shot} (byte-exact)`);
      } else {
        console.error(`FAIL ${spec.name}.f${shot}: differs from golden (see ${png})`);
        failures++;
      }
    }
  }
}

if (update) {
  const ppssppRepo = resolve(ppsspp, "../..");
  const commit = await $`git -C ${ppssppRepo} rev-parse HEAD`.nothrow().text();
  if (commit.trim()) {
    await Bun.write(`${goldens}/PPSSPP-COMMIT.txt`, commit);
  }
}

if (failures > 0) {
  console.error(`\nE2E FAILED (${failures})`);
  process.exit(1);
}
console.log("\nE2E OK");
