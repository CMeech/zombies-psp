import { describe, expect, test } from "bun:test";

const ROOT = new URL("../", import.meta.url);

const canonicalKeys = [
  "schema",
  "tick",
  "seed",
  "phase",
  "player",
  "hp",
  "alive",
  "speedQ",
  "weapon",
  "ammo",
  "reserve",
  "reloading",
  "reloadTicksRemaining",
  "targets",
  "total",
  "score",
  "wins",
  "losses",
] as const;

describe("V1 host facts parity", () => {
  test("desktop and PSP state encoders consume shared facts and expose every V1 key", async () => {
    const [desktop, psp] = await Promise.all([
      Bun.file(new URL("crates/openstrike/src/guest.rs", ROOT)).text(),
      Bun.file(new URL("crates/openstrike-psp/src/strike.rs", ROOT)).text(),
    ]).then((sources) =>
      sources.map((source, index) => {
        const start = source.indexOf("fn build_state");
        const end = source.indexOf("fn build_event", start);
        expect(start, `encoder ${index} start`).toBeGreaterThanOrEqual(0);
        expect(end, `encoder ${index} end`).toBeGreaterThan(start);
        return source.slice(start, end);
      }),
    );

    for (const encoder of [desktop, psp]) {
      expect(encoder).toContain("facts_v1()");
      for (const key of canonicalKeys) expect(encoder).toContain(key);
    }
  });

  test("desktop and PSP event encoders expose the same V1 event vocabulary", async () => {
    const sources = await Promise.all([
      Bun.file(new URL("crates/openstrike/src/guest.rs", ROOT)).text(),
      Bun.file(new URL("crates/openstrike-psp/src/strike.rs", ROOT)).text(),
    ]);
    const eventNames = [
      "shotFired",
      "targetHit",
      "targetDestroyed",
      "playerDamaged",
      "playerDied",
      "roundReset",
    ];
    for (const source of sources) {
      const start = source.indexOf("fn build_event");
      const end = source.indexOf("fn mount_strike", start) >= 0
        ? source.indexOf("fn mount_strike", start)
        : source.indexOf("dispatch_menu", start);
      expect(start).toBeGreaterThanOrEqual(0);
      expect(end).toBeGreaterThan(start);
      const encoder = source.slice(start, end);
      for (const event of eventNames) expect(encoder).toContain(event);
    }
  });
});
