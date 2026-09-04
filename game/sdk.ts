// The `strike` surface SDK — the JS half of OpenStrike's vocabulary.
// Host side: crates/openstrike/src/guest.rs.
//
// Per tick the host calls `strike.__dispatch(state, events)` (facts), then
// the PocketJS frame turn runs (HUD). Commands issued here join one V1 batch
// that the host takes after the guest turn — state read through this module is always
// the host's last-published snapshot, never a guess.

export interface StrikeState {
  /** V1 fields are absent only in the PSP menu, where no simulation exists. */
  schema?: 1;
  tick?: number;
  seed?: number;
  player?: { hp: number; alive: boolean; speedQ: number };
  weapon?: {
    ammo: number;
    reserve: number;
    reloading: boolean;
    reloadTicksRemaining: number;
  };
  targets?: { alive: number; total: number };
  score?: { wins: number; losses: number };
  /** Presentation time and temporary flat aliases used by the imported HUD. */
  time: number;
  phase: "menu" | "starting" | "live" | "won" | "lost";
  hp: number;
  alive: boolean;
  ammo: number;
  reserve: number;
  reloading: boolean;
  reloadFrac: number;
  aliveBots: number;
  totalBots: number;
  wins: number;
  losses: number;
  speed: number;
}

export type StrikeEvent =
  | { type: "shotFired"; weaponId: number; ammo: number }
  | { type: "targetHit"; targetId: number; damage: number; hp: number; fatal: boolean }
  | { type: "targetDestroyed"; targetId: number }
  | { type: "playerDamaged"; amount: number; hp: number }
  | { type: "playerDied" }
  | { type: "roundReset"; round: number };

export interface WeaponConfig {
  magazineCapacity: number;
  reserveCapacity: number;
  fireIntervalTicks: number;
  reloadTicks: number;
  damage: number;
}

export type SliceCommand =
  | { type: "setPhase"; phase: "starting" | "live" | "won" | "lost" }
  | { type: "resetRound" }
  | { type: "addWin" }
  | { type: "addLoss" }
  | { type: "configureWeapon"; config: WeaponConfig }
  | { type: "configureTarget"; config: { health: number } };

export interface SliceCommandBatch {
  schema: 1;
  afterTick: number;
  commands: SliceCommand[];
}

export interface NativeStrike {
  /** Cooked maps available to loadMap (index-aligned), host-injected. */
  maps?: string[];
  loadMap?(index: number): void;
  toMenu?(): void;
  __dispatch?: (state: StrikeState, events: StrikeEvent[]) => void;
  __takeCommands?: (afterTick: number) => SliceCommandBatch;
}

const native = (globalThis as { strike?: NativeStrike }).strike;
if (!native) {
  throw new Error("openstrike: no `strike` surface — is this running under the game host?");
}

let current: StrikeState = {
  time: 0,
  phase: "starting",
  hp: 100,
  alive: true,
  ammo: 30,
  reserve: 90,
  reloading: false,
  reloadFrac: 0,
  aliveBots: 0,
  totalBots: 0,
  wins: 0,
  losses: 0,
  speed: 0,
};

type Handler = (e: StrikeEvent) => void;
type TickHandler = (s: StrikeState) => void;
const handlers = new Map<string, Set<Handler>>();
const tickHandlers = new Set<TickHandler>();
let commands: SliceCommand[] = [];

native.__dispatch = (state, events) => {
  current = state;
  for (const e of events) {
    const set = handlers.get(e.type);
    if (set) for (const h of [...set]) h(e);
  }
  for (const h of [...tickHandlers]) h(state);
};

native.__takeCommands = (afterTick) => {
  const batch = { schema: 1 as const, afterTick, commands };
  commands = [];
  return batch;
};

export const strike = {
  /** The last state snapshot the host published (this tick). */
  state: (): StrikeState => current,

  /** Subscribe to a game event; returns an unsubscribe. */
  on(type: StrikeEvent["type"], fn: Handler): () => void {
    let set = handlers.get(type);
    if (!set) handlers.set(type, (set = new Set()));
    set.add(fn);
    return () => set.delete(fn);
  },

  /** Runs once per tick, after events, with the fresh state. */
  onTick(fn: TickHandler): () => void {
    tickHandlers.add(fn);
    return () => tickHandlers.delete(fn);
  },

  // ---- intent (batched guest-side, taken once after this guest turn) ------
  /** Map names the host can load (empty on hosts that boot pre-loaded). */
  maps: (native.maps ?? []) as readonly string[],
  /** Ask the host to load a cooked map and start a round (menu hosts). */
  loadMap: (index: number) => native.loadMap?.(index),
  /** Leave the round and return to the main menu (menu hosts). */
  toMenu: () => native.toMenu?.(),
  setPhase: (phase: "starting" | "live" | "won" | "lost") =>
    commands.push({ type: "setPhase", phase }),
  resetRound: () => commands.push({ type: "resetRound" }),
  addWin: () => commands.push({ type: "addWin" }),
  addLoss: () => commands.push({ type: "addLoss" }),
  configureWeapon: (config: WeaponConfig) =>
    commands.push({ type: "configureWeapon", config }),
  configureTarget: (health: number) =>
    commands.push({ type: "configureTarget", config: { health } }),
};
