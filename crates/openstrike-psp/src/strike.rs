//! The `strike` surface on PSP: the same hand-written vocabulary the desktop
//! mounts through rquickjs (guest.rs), expressed through the raw QuickJS C
//! API. Ops queue [`Command`]s applied after the guest turn; facts flow the
//! other way through `strike.__dispatch(state, events)` — field for field
//! identical to the desktop build_state/build_event, so game/sdk.ts sees one
//! surface.

use alloc::vec::Vec;

use libquickjs_sys::*;
use openstrike_core::contract::{
    SliceCommandBatchV1, SliceCommandV1, SlicePhaseV1, TargetConfigV1, WeaponConfigV1,
};
use openstrike_core::sim::{Command, GameEvent, StrikeSim};
use openstrike_core::weapon::WeaponConfig;
use openstrike_core::{MAX_COMMANDS_V1, MAX_EVENTS_V1};
use pocketjs_psp::ffi::{add_fn, arg_i32};

// Symbols the vendored libquickjs-sys omits (provided by the linked QuickJS
// C library — the established local-extern pattern).
extern "C" {
    fn JS_NewStringLen(ctx: *mut JSContext, s: *const u8, len: usize) -> JSValue;
    fn JS_NewArray(ctx: *mut JSContext) -> JSValue;
    fn JS_GetPropertyUint32(ctx: *mut JSContext, this_obj: JSValue, idx: u32) -> JSValue;
    fn JS_SetPropertyUint32(ctx: *mut JSContext, this_obj: JSValue, idx: u32, val: JSValue) -> i32;
}

/// Commands queued by ops during the guest turn (single-threaded host).
static mut COMMANDS: Vec<Command> = Vec::new();

pub unsafe fn drain(mut apply: impl FnMut(Command)) {
    for cmd in COMMANDS.drain(..) {
        apply(cmd);
    }
}

fn command_for_sim(command: SliceCommandV1) -> Command {
    match command {
        SliceCommandV1::SetPhase(phase) => Command::SetPhase(phase.into()),
        SliceCommandV1::ResetRound => Command::ResetRound,
        SliceCommandV1::AddWin => Command::AddWin,
        SliceCommandV1::AddLoss => Command::AddLoss,
        SliceCommandV1::ConfigureWeapon(config) => Command::ConfigureWeapon(WeaponConfig {
            mag_size: config.magazine_capacity,
            reserve: config.reserve_capacity,
            fire_interval: config.fire_interval_ticks as f32 / 60.0,
            reload_time: config.reload_ticks as f32 / 60.0,
            damage_body: config.damage as i32,
            damage_head: config.damage as i32,
        }),
        SliceCommandV1::ConfigureTarget(config) => Command::ConfigureTarget(config.health as i32),
    }
}

/// HOST-level intents (world lifecycle, not simulation): queued like
/// Commands, drained by the frame loop after present.
#[derive(Clone, Copy, Debug)]
pub enum HostCmd {
    LoadMap(usize),
    ToMenu,
}

static mut HOST_CMDS: Vec<HostCmd> = Vec::new();

pub unsafe fn drain_host(mut apply: impl FnMut(HostCmd)) {
    for cmd in HOST_CMDS.drain(..) {
        apply(cmd);
    }
}

unsafe extern "C" fn js_load_map(
    ctx: *mut JSContext,
    _this: JSValue,
    argc: i32,
    argv: *mut JSValue,
) -> JSValue {
    let i = arg_i32(ctx, argc, argv, 0);
    if i >= 0 {
        HOST_CMDS.push(HostCmd::LoadMap(i as usize));
    }
    JS_UNDEFINED
}

unsafe extern "C" fn js_to_menu(
    _ctx: *mut JSContext,
    _this: JSValue,
    _argc: i32,
    _argv: *mut JSValue,
) -> JSValue {
    HOST_CMDS.push(HostCmd::ToMenu);
    JS_UNDEFINED
}

// ---- value helpers ---------------------------------------------------------

unsafe fn set_val(ctx: *mut JSContext, obj: JSValue, key: &'static [u8], val: JSValue) {
    // JS_SetPropertyStr consumes `val`.
    JS_SetPropertyStr(ctx, obj, key.as_ptr() as *const _, val);
}

unsafe fn set_str(ctx: *mut JSContext, obj: JSValue, key: &'static [u8], s: &str) {
    let v = JS_NewStringLen(ctx, s.as_ptr(), s.len());
    set_val(ctx, obj, key, v);
}

unsafe fn get_f32(ctx: *mut JSContext, obj: JSValue, key: &'static [u8], default: f32) -> f32 {
    let v = JS_GetPropertyStr(ctx, obj, key.as_ptr() as *const _);
    if JS_IsUndefined(v) {
        JS_FreeValue(ctx, v);
        return default;
    }
    let mut out = 0f64;
    let bad = JS_ToFloat64(ctx, &mut out, v) != 0;
    JS_FreeValue(ctx, v);
    if bad {
        default
    } else {
        out as f32
    }
}

unsafe fn get_i32(ctx: *mut JSContext, obj: JSValue, key: &'static [u8], default: i32) -> i32 {
    let f = get_f32(ctx, obj, key, default as f32);
    f as i32
}

unsafe fn get_u32(ctx: *mut JSContext, obj: JSValue, key: &'static [u8], default: u32) -> u32 {
    get_i32(ctx, obj, key, default as i32).max(0) as u32
}

unsafe fn property_string(
    ctx: *mut JSContext,
    object: JSValue,
    key: &'static [u8],
) -> Option<alloc::string::String> {
    let value = JS_GetPropertyStr(ctx, object, key.as_ptr() as *const _);
    let mut result = None;
    let mut len: size_t = 0;
    let text = JS_ToCStringLen2(ctx, &mut len, value, 0);
    if !text.is_null() {
        if let Ok(text) = core::str::from_utf8(core::slice::from_raw_parts(text as *const u8, len))
        {
            result = Some(text.into());
        }
        JS_FreeCString(ctx, text);
    }
    JS_FreeValue(ctx, value);
    result
}

unsafe fn parse_slice_command(ctx: *mut JSContext, object: JSValue) -> Option<SliceCommandV1> {
    let kind = property_string(ctx, object, b"type\0")?;
    Some(match kind.as_str() {
        "setPhase" => SliceCommandV1::SetPhase(
            SlicePhaseV1::from_wire_name(&property_string(ctx, object, b"phase\0")?).ok()?,
        ),
        "resetRound" => SliceCommandV1::ResetRound,
        "addWin" => SliceCommandV1::AddWin,
        "addLoss" => SliceCommandV1::AddLoss,
        "configureWeapon" => {
            let config = JS_GetPropertyStr(ctx, object, b"config\0".as_ptr() as *const _);
            let command = SliceCommandV1::ConfigureWeapon(WeaponConfigV1 {
                magazine_capacity: get_u32(ctx, config, b"magazineCapacity\0", 0),
                reserve_capacity: get_u32(ctx, config, b"reserveCapacity\0", 0),
                fire_interval_ticks: get_u32(ctx, config, b"fireIntervalTicks\0", 0),
                reload_ticks: get_u32(ctx, config, b"reloadTicks\0", 0),
                damage: get_u32(ctx, config, b"damage\0", 0),
            });
            JS_FreeValue(ctx, config);
            command
        }
        "configureTarget" => {
            let config = JS_GetPropertyStr(ctx, object, b"config\0".as_ptr() as *const _);
            let command = SliceCommandV1::ConfigureTarget(TargetConfigV1 {
                health: get_u32(ctx, config, b"health\0", 0),
            });
            JS_FreeValue(ctx, config);
            command
        }
        _ => return None,
    })
}

/// Take and validate the one simulation-command batch produced by this guest
/// turn. Parsed commands retain their array order for the existing drain step.
pub unsafe fn take_commands(ctx: *mut JSContext, global: JSValue, published_tick: u32) -> bool {
    COMMANDS.clear();
    let strike = JS_GetPropertyStr(ctx, global, b"strike\0".as_ptr() as *const _);
    let take = JS_GetPropertyStr(ctx, strike, b"__takeCommands\0".as_ptr() as *const _);
    if JS_IsUndefined(take) {
        JS_FreeValue(ctx, take);
        JS_FreeValue(ctx, strike);
        return false;
    }
    let mut args = [JS_NewInt32(ctx, published_tick as i32)];
    let batch_value = JS_Call(ctx, take, strike, 1, args.as_mut_ptr());
    JS_FreeValue(ctx, args[0]);
    JS_FreeValue(ctx, take);
    JS_FreeValue(ctx, strike);
    if JS_ValueGetTag(batch_value) == JS_TAG_EXCEPTION {
        JS_FreeValue(ctx, batch_value);
        return false;
    }

    let schema = get_u32(ctx, batch_value, b"schema\0", 0);
    let after_tick = get_u32(ctx, batch_value, b"afterTick\0", u32::MAX);
    let array = JS_GetPropertyStr(ctx, batch_value, b"commands\0".as_ptr() as *const _);
    let length = get_u32(ctx, array, b"length\0", u32::MAX);
    if length as usize > MAX_COMMANDS_V1 {
        JS_FreeValue(ctx, array);
        JS_FreeValue(ctx, batch_value);
        return false;
    }
    let mut commands = Vec::new();
    for index in 0..length {
        let object = JS_GetPropertyUint32(ctx, array, index);
        let Some(command) = parse_slice_command(ctx, object) else {
            JS_FreeValue(ctx, object);
            JS_FreeValue(ctx, array);
            JS_FreeValue(ctx, batch_value);
            return false;
        };
        JS_FreeValue(ctx, object);
        commands.push(command);
    }
    JS_FreeValue(ctx, array);
    JS_FreeValue(ctx, batch_value);
    let batch = SliceCommandBatchV1 {
        schema,
        after_tick,
        commands,
    };
    if batch.validate(published_tick).is_err() {
        return false;
    }
    COMMANDS.extend(batch.commands.into_iter().map(command_for_sim));
    true
}

/// Install `globalThis.strike` (intent ops; the SDK adds `__dispatch`).
pub unsafe fn register(ctx: *mut JSContext, global: JSValue, maps: &[alloc::string::String]) {
    let obj = JS_NewObject(ctx);
    add_fn(ctx, obj, b"loadMap\0", js_load_map, 1);
    add_fn(ctx, obj, b"toMenu\0", js_to_menu, 0);
    // The cooked-map catalogue (menu hosts): strike.maps = ["de_dust2", …].
    let arr = JS_NewArray(ctx);
    for (i, name) in maps.iter().enumerate() {
        let v = JS_NewStringLen(ctx, name.as_ptr(), name.len());
        JS_SetPropertyUint32(ctx, arr, i as u32, v);
    }
    set_val(ctx, obj, b"maps\0", arr);
    JS_SetPropertyStr(ctx, global, b"strike\0".as_ptr() as *const _, obj);
}

// ---- state/events → guest ---------------------------------------------------

unsafe fn build_state(ctx: *mut JSContext, sim: &StrikeSim) -> JSValue {
    let facts = sim.facts_v1();
    let o = JS_NewObject(ctx);
    set_val(ctx, o, b"schema\0", JS_NewInt32(ctx, facts.schema as i32));
    set_val(ctx, o, b"tick\0", JS_NewInt32(ctx, facts.tick as i32));
    set_val(ctx, o, b"seed\0", JS_NewInt32(ctx, facts.seed as i32));
    set_val(ctx, o, b"time\0", JS_NewFloat64(ctx, sim.time as f64));
    set_str(ctx, o, b"phase\0", facts.phase.wire_name());

    let player = JS_NewObject(ctx);
    set_val(ctx, player, b"hp\0", JS_NewInt32(ctx, facts.player.hp));
    set_val(ctx, player, b"alive\0", JS_NewBool(ctx, facts.player.alive));
    set_val(
        ctx,
        player,
        b"speedQ\0",
        JS_NewInt32(ctx, facts.player.speed_q),
    );
    set_val(ctx, o, b"player\0", player);

    let weapon = JS_NewObject(ctx);
    set_val(
        ctx,
        weapon,
        b"ammo\0",
        JS_NewInt32(ctx, facts.weapon.ammo as i32),
    );
    set_val(
        ctx,
        weapon,
        b"reserve\0",
        JS_NewInt32(ctx, facts.weapon.reserve as i32),
    );
    set_val(
        ctx,
        weapon,
        b"reloading\0",
        JS_NewBool(ctx, facts.weapon.reloading),
    );
    set_val(
        ctx,
        weapon,
        b"reloadTicksRemaining\0",
        JS_NewInt32(ctx, facts.weapon.reload_ticks_remaining as i32),
    );
    set_val(ctx, o, b"weapon\0", weapon);

    let targets = JS_NewObject(ctx);
    set_val(
        ctx,
        targets,
        b"alive\0",
        JS_NewInt32(ctx, facts.targets.alive as i32),
    );
    set_val(
        ctx,
        targets,
        b"total\0",
        JS_NewInt32(ctx, facts.targets.total as i32),
    );
    set_val(ctx, o, b"targets\0", targets);

    let score = JS_NewObject(ctx);
    set_val(
        ctx,
        score,
        b"wins\0",
        JS_NewInt32(ctx, facts.score.wins as i32),
    );
    set_val(
        ctx,
        score,
        b"losses\0",
        JS_NewInt32(ctx, facts.score.losses as i32),
    );
    set_val(ctx, o, b"score\0", score);

    // Temporary aliases consumed by the imported HUD during migration.
    set_val(ctx, o, b"hp\0", JS_NewInt32(ctx, facts.player.hp));
    set_val(ctx, o, b"alive\0", JS_NewBool(ctx, facts.player.alive));
    set_val(
        ctx,
        o,
        b"ammo\0",
        JS_NewInt32(ctx, facts.weapon.ammo as i32),
    );
    set_val(
        ctx,
        o,
        b"reserve\0",
        JS_NewInt32(ctx, facts.weapon.reserve as i32),
    );
    set_val(
        ctx,
        o,
        b"reloading\0",
        JS_NewBool(ctx, facts.weapon.reloading),
    );
    set_val(
        ctx,
        o,
        b"reloadFrac\0",
        JS_NewFloat64(ctx, sim.reload_frac() as f64),
    );
    set_val(
        ctx,
        o,
        b"aliveBots\0",
        JS_NewInt32(ctx, facts.targets.alive as i32),
    );
    set_val(
        ctx,
        o,
        b"totalBots\0",
        JS_NewInt32(ctx, facts.targets.total as i32),
    );
    set_val(ctx, o, b"wins\0", JS_NewInt32(ctx, facts.score.wins as i32));
    set_val(
        ctx,
        o,
        b"losses\0",
        JS_NewInt32(ctx, facts.score.losses as i32),
    );
    set_val(
        ctx,
        o,
        b"speed\0",
        JS_NewFloat64(ctx, sim.ground_speed() as f64),
    );
    o
}

unsafe fn build_event(ctx: *mut JSContext, e: &GameEvent) -> JSValue {
    let o = JS_NewObject(ctx);
    match e {
        GameEvent::ShotFired { weapon_id, ammo } => {
            set_str(ctx, o, b"type\0", "shotFired");
            set_val(ctx, o, b"weaponId\0", JS_NewInt32(ctx, *weapon_id as i32));
            set_val(ctx, o, b"ammo\0", JS_NewInt32(ctx, *ammo as i32));
        }
        GameEvent::TargetHit {
            target_id,
            damage,
            hp,
            fatal,
        } => {
            set_str(ctx, o, b"type\0", "targetHit");
            set_val(ctx, o, b"targetId\0", JS_NewInt32(ctx, *target_id as i32));
            set_val(ctx, o, b"damage\0", JS_NewInt32(ctx, *damage as i32));
            set_val(ctx, o, b"hp\0", JS_NewInt32(ctx, *hp as i32));
            set_val(ctx, o, b"fatal\0", JS_NewBool(ctx, *fatal));
        }
        GameEvent::TargetDestroyed { target_id } => {
            set_str(ctx, o, b"type\0", "targetDestroyed");
            set_val(ctx, o, b"targetId\0", JS_NewInt32(ctx, *target_id as i32));
        }
        GameEvent::PlayerDamaged { amount, hp } => {
            set_str(ctx, o, b"type\0", "playerDamaged");
            set_val(ctx, o, b"amount\0", JS_NewInt32(ctx, *amount));
            set_val(ctx, o, b"hp\0", JS_NewInt32(ctx, *hp));
        }
        GameEvent::PlayerDied => set_str(ctx, o, b"type\0", "playerDied"),
        GameEvent::RoundReset { round } => {
            set_str(ctx, o, b"type\0", "roundReset");
            set_val(ctx, o, b"round\0", JS_NewInt32(ctx, *round as i32));
        }
    }
    o
}

/// Menu-mode dispatch: no simulation exists, publish a minimal snapshot
/// with phase "menu" (and an empty event batch).
pub unsafe fn dispatch_menu(ctx: *mut JSContext, global: JSValue, time: f64) -> bool {
    let strike = JS_GetPropertyStr(ctx, global, b"strike\0".as_ptr() as *const _);
    if JS_IsUndefined(strike) {
        JS_FreeValue(ctx, strike);
        return true;
    }
    let dispatch = JS_GetPropertyStr(ctx, strike, b"__dispatch\0".as_ptr() as *const _);
    let mut ok = true;
    if !JS_IsUndefined(dispatch) {
        let o = JS_NewObject(ctx);
        set_val(ctx, o, b"time\0", JS_NewFloat64(ctx, time));
        set_str(ctx, o, b"phase\0", "menu");
        set_val(ctx, o, b"hp\0", JS_NewInt32(ctx, 100));
        set_val(ctx, o, b"alive\0", JS_NewBool(ctx, true));
        set_val(ctx, o, b"ammo\0", JS_NewInt32(ctx, 0));
        set_val(ctx, o, b"reserve\0", JS_NewInt32(ctx, 0));
        set_val(ctx, o, b"reloading\0", JS_NewBool(ctx, false));
        set_val(ctx, o, b"reloadFrac\0", JS_NewFloat64(ctx, 0.0));
        set_val(ctx, o, b"aliveBots\0", JS_NewInt32(ctx, 0));
        set_val(ctx, o, b"totalBots\0", JS_NewInt32(ctx, 0));
        set_val(ctx, o, b"wins\0", JS_NewInt32(ctx, 0));
        set_val(ctx, o, b"losses\0", JS_NewInt32(ctx, 0));
        set_val(ctx, o, b"speed\0", JS_NewFloat64(ctx, 0.0));
        let batch = JS_NewArray(ctx);
        let mut args = [o, batch];
        let r = JS_Call(ctx, dispatch, strike, 2, args.as_mut_ptr());
        if JS_ValueGetTag(r) == JS_TAG_EXCEPTION {
            ok = false;
        }
        JS_FreeValue(ctx, r);
        JS_FreeValue(ctx, o);
        JS_FreeValue(ctx, batch);
    }
    JS_FreeValue(ctx, dispatch);
    JS_FreeValue(ctx, strike);
    ok
}

/// One guest-ward dispatch: drain the sim's event batch and call
/// `strike.__dispatch(state, events)` if the SDK installed it.
pub unsafe fn dispatch(ctx: *mut JSContext, global: JSValue, sim: &mut StrikeSim) -> bool {
    let events = core::mem::take(&mut sim.events);
    if events.len() > MAX_EVENTS_V1 {
        return false;
    }
    let strike = JS_GetPropertyStr(ctx, global, b"strike\0".as_ptr() as *const _);
    if JS_IsUndefined(strike) {
        JS_FreeValue(ctx, strike);
        return true;
    }
    let dispatch = JS_GetPropertyStr(ctx, strike, b"__dispatch\0".as_ptr() as *const _);
    let mut ok = true;
    if !JS_IsUndefined(dispatch) {
        let state = build_state(ctx, sim);
        let batch = JS_NewArray(ctx);
        for (i, e) in events.iter().enumerate() {
            // JS_SetPropertyUint32 consumes the value.
            JS_SetPropertyUint32(ctx, batch, i as u32, build_event(ctx, e));
        }
        let mut args = [state, batch];
        let r = JS_Call(ctx, dispatch, strike, 2, args.as_mut_ptr());
        if JS_ValueGetTag(r) == JS_TAG_EXCEPTION {
            ok = false;
        }
        JS_FreeValue(ctx, r);
        JS_FreeValue(ctx, state);
        JS_FreeValue(ctx, batch);
    }
    JS_FreeValue(ctx, dispatch);
    JS_FreeValue(ctx, strike);
    ok
}
