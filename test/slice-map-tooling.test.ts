import { describe, expect, test } from "bun:test";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { buildWad3, parsePpm } from "../scripts/slice-map-lib";

const root = resolve(import.meta.dir, "..");
describe("slice room source tooling", () => {
  test("validates the original power-of-two texture", () => {
    const image = parsePpm(resolve(root, "assets-src/slice_test_room/textures/wall.ppm"));
    expect([image.width, image.height]).toEqual([16, 16]);
  });
  test("builds deterministic WAD3 bytes", () => {
    const dir = mkdtempSync(join(tmpdir(), "slice-wad-"));
    const a = join(dir, "a.wad"), b = join(dir, "b.wad");
    const source = resolve(root, "assets-src/slice_test_room/textures/wall.ppm");
    buildWad3(source, a); buildWad3(source, b);
    expect(readFileSync(a)).toEqual(readFileSync(b));
    expect(readFileSync(a).subarray(0, 4).toString()).toBe("WAD3");
  });
  test("keeps the room sealed by the strict cooker and includes both lanes", () => {
    const map = readFileSync(resolve(root, "assets-src/slice_test_room/slice_test_room.map"), "utf8");
    const cooker = readFileSync(resolve(root, "scripts/cook-slice-room.ts"), "utf8");
    expect(map.match(/^\{$/gm)?.length).toBe(11); // world + seven brushes + three point entities
    expect(map).toContain('"targetname" "player_start"');
    expect(map).toContain('"targetname" "target_start"');
    expect(map).toContain("Miss-lane occluder");
    expect(cooker).toContain("-leaktest");
    expect(cooker).toContain("`${vis} -fast ${bsp}`");
    expect(cooker).not.toContain("-nofill");
  });
});
