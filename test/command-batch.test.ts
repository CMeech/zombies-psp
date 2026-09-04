import { expect, test } from "bun:test";

const native: Record<string, unknown> = {};
(globalThis as { strike?: Record<string, unknown> }).strike = native;
const { strike } = await import("../game/sdk.ts");

test("simulation intents return once in an ordered tick-tagged V1 batch", () => {
  strike.configureWeapon({
    magazineCapacity: 12,
    reserveCapacity: 48,
    fireIntervalTicks: 9,
    reloadTicks: 120,
    damage: 25,
  });
  strike.configureTarget(75);
  strike.addWin();
  strike.setPhase("won");

  const take = native.__takeCommands as (tick: number) => unknown;
  expect(take(42)).toEqual({
    schema: 1,
    afterTick: 42,
    commands: [
      {
        type: "configureWeapon",
        config: {
          magazineCapacity: 12,
          reserveCapacity: 48,
          fireIntervalTicks: 9,
          reloadTicks: 120,
          damage: 25,
        },
      },
      { type: "configureTarget", config: { health: 75 } },
      { type: "addWin" },
      { type: "setPhase", phase: "won" },
    ],
  });
  expect(take(43)).toEqual({ schema: 1, afterTick: 43, commands: [] });
});
