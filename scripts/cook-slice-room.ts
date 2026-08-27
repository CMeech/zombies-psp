import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { createHash } from "node:crypto";
import { $ } from "bun";
import { buildWad3, MAP_TOOL_ARCHIVE_SHA256, MAP_TOOL_REVISION } from "./slice-map-lib";

const root = resolve(import.meta.dir, "..");
const sourceDir = resolve(root, "assets-src/slice_test_room");
const outDir = resolve(root, "dist/maps");
const toolBuild = resolve(root, "local/tools/ericw-tools", MAP_TOOL_REVISION, "build");
const qbsp = resolve(toolBuild, "qbsp/qbsp"), vis = resolve(toolBuild, "vis/vis"), light = resolve(toolBuild, "light/light");
for (const tool of [qbsp, vis, light]) if (!existsSync(tool)) throw new Error(`missing ${tool}; run bun run setup:map-tools`);
const map = resolve(sourceDir, "slice_test_room.map"), ppm = resolve(sourceDir, "textures/wall.ppm");
const provenance = resolve(root, "assets/provenance/slice_test_room.md");
for (const input of [map, ppm, provenance]) if (!existsSync(input)) throw new Error(`missing required source ${input}`);
mkdirSync(outDir, { recursive: true });
const wad = resolve(outDir, "slice_test_room.wad");
const texture = buildWad3(ppm, wad);
if (texture.width > 256 || texture.height > 256) throw new Error("slice texture exceeds 256x256 budget");
const bsp = resolve(outDir, "slice_test_room.bsp");
await $`${qbsp} -hlbsp -noallowupgrade -leaktest -wadpath ${outDir} ${map} ${bsp}`;
await $`${vis} -fast ${bsp}`;
await $`${light} -extra4 ${bsp}`;
const p3d = resolve(outDir, "slice_test_room.p3d");
await $`cargo run --release -q --manifest-path ${resolve(root, "vendor/pocketjs/engine/Cargo.toml")} -p pocket3d-cook -- ${bsp} --wads ${outDir} --subdivide 32 -o ${p3d} --verify`;
if (statSync(p3d).size > 1024 * 1024) throw new Error("cooked room exceeds 1 MiB budget");
const sha256 = (path: string) => createHash("sha256").update(readFileSync(path)).digest("hex");
const manifest = {
  schema: 1, map: "slice_test_room", compiler: { repository: "https://github.com/ericwa/ericw-tools", revision: MAP_TOOL_REVISION, sourceArchiveSha256: MAP_TOOL_ARCHIVE_SHA256 },
  arguments: { qbsp: ["-hlbsp", "-noallowupgrade", "-leaktest", "-wadpath", "dist/maps"], vis: ["-fast"], light: ["-extra4"], pocket3dCook: ["--wads", "dist/maps", "--subdivide", "32", "--verify"] },
  inputs: [{ path: "assets-src/slice_test_room/slice_test_room.map", sha256: sha256(map) }, { path: "assets-src/slice_test_room/textures/wall.ppm", sha256: sha256(ppm) }, { path: "assets/provenance/slice_test_room.md", sha256: sha256(provenance) }, { path: "scripts/slice-map-lib.ts", sha256: sha256(resolve(root, "scripts/slice-map-lib.ts")) }, { path: "scripts/cook-slice-room.ts", sha256: sha256(resolve(root, "scripts/cook-slice-room.ts")) }],
  textures: { sourceCount: 1, cookedCount: 2, generatedPlaceholders: 1, level0Bytes: texture.width * texture.height, maxDimensions: [texture.width, texture.height] },
  output: { path: "dist/maps/slice_test_room.p3d", sha256: sha256(p3d), bytes: statSync(p3d).size }
};
writeFileSync(resolve(outDir, "slice_test_room.manifest.json"), JSON.stringify(manifest, null, 2) + "\n");
console.log(JSON.stringify(manifest, null, 2));
