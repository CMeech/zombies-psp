//! Strict, versioned scenario and normalized input-tape formats.
//!
//! Execution is added independently from parsing so malformed checked-in data
//! fails before map or GPU initialization.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use openstrike_core::{GameEvent, Rng, SLICE_SCHEMA_V1, SimInput, TICK_RATE_V1};

use crate::args::Args;
use crate::game::OpenStrike;
use crate::guest::StrikeGuest;
use crate::scripts::Headless;

const TICK_SECONDS: f32 = 1.0 / TICK_RATE_V1 as f32;
const LOOK_RADIANS_PER_TICK: f32 = 3.0 * core::f32::consts::PI / 180.0;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScenarioV1 {
    pub schema: u32,
    pub name: String,
    pub map: String,
    pub seed: u32,
    pub tick_rate: u32,
    pub max_ticks: u32,
    pub player_spawn: String,
    pub input_tape: PathBuf,
    #[serde(default)]
    pub assertions: Vec<AssertionV1>,
    #[serde(default)]
    pub captures: Vec<CaptureV1>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum AssertionV1 {
    Equals {
        tick: u32,
        path: String,
        value: Value,
    },
    Near {
        tick: u32,
        path: String,
        value: f64,
        tolerance: f64,
    },
    EventCount {
        event: EventNameV1,
        count: u32,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum EventNameV1 {
    ShotFired,
    TargetHit,
    TargetDestroyed,
    PlayerDamaged,
    PlayerDied,
    RoundReset,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CaptureV1 {
    pub tick: u32,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct InputTapeV1 {
    pub schema: u32,
    pub tick_rate: u32,
    pub frames: Vec<InputFrameV1>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InputFrameV1 {
    pub tick: u32,
    #[serde(default)]
    pub r#move: Option<[f32; 2]>,
    #[serde(default)]
    pub look: Option<[f32; 2]>,
    #[serde(default)]
    pub actions: Option<Vec<ActionV1>>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum ActionV1 {
    Fire,
    Reload,
}

impl ScenarioV1 {
    pub fn load(path: &Path) -> Result<Self> {
        let bytes =
            std::fs::read(path).with_context(|| format!("reading scenario {}", path.display()))?;
        let scenario: Self = serde_json::from_slice(&bytes)
            .with_context(|| format!("parsing scenario {}", path.display()))?;
        scenario.validate()?;
        Ok(scenario)
    }

    pub fn validate(&self) -> Result<()> {
        validate_header(self.schema, self.tick_rate)?;
        if self.name.is_empty() || self.map.is_empty() || self.player_spawn.is_empty() {
            bail!("scenario name, map, and playerSpawn must be non-empty");
        }
        if self.max_ticks == 0 {
            bail!("scenario maxTicks must be positive");
        }
        for assertion in &self.assertions {
            match assertion {
                AssertionV1::Equals { tick, path, .. } => {
                    validate_tick(*tick, self.max_ticks, "assertion")?;
                    validate_path(path)?;
                }
                AssertionV1::Near {
                    tick,
                    path,
                    value,
                    tolerance,
                } => {
                    validate_tick(*tick, self.max_ticks, "assertion")?;
                    validate_path(path)?;
                    if !value.is_finite() || !tolerance.is_finite() || *tolerance < 0.0 {
                        bail!("near assertion values must be finite and tolerance non-negative");
                    }
                }
                AssertionV1::EventCount { .. } => {}
            }
        }
        let mut capture_names = BTreeSet::new();
        let mut capture_ticks = BTreeSet::new();
        for capture in &self.captures {
            validate_tick(capture.tick, self.max_ticks, "capture")?;
            if capture.name.is_empty() {
                bail!("capture name must be non-empty");
            }
            if !capture_names.insert(capture.name.as_str()) {
                bail!("duplicate capture name '{}'", capture.name);
            }
            if !capture_ticks.insert(capture.tick) {
                bail!("duplicate capture tick {}", capture.tick);
            }
        }
        Ok(())
    }
}

impl InputTapeV1 {
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path)
            .with_context(|| format!("reading input tape {}", path.display()))?;
        let tape: Self = serde_json::from_slice(&bytes)
            .with_context(|| format!("parsing input tape {}", path.display()))?;
        tape.validate()?;
        Ok(tape)
    }

    pub fn validate(&self) -> Result<()> {
        validate_header(self.schema, self.tick_rate)?;
        let mut previous = None;
        for frame in &self.frames {
            if previous.is_some_and(|tick| frame.tick <= tick) {
                bail!("input frame ticks must be strictly increasing");
            }
            previous = Some(frame.tick);
            validate_axes(frame.r#move, "move")?;
            validate_axes(frame.look, "look")?;
            if let Some(actions) = &frame.actions {
                let mut seen = BTreeSet::new();
                for action in actions {
                    if !seen.insert(*action) {
                        bail!("duplicate input action {action:?} at tick {}", frame.tick);
                    }
                }
            }
        }
        Ok(())
    }
}

fn validate_header(schema: u32, tick_rate: u32) -> Result<()> {
    if schema != SLICE_SCHEMA_V1 {
        bail!("unsupported schema {schema}; expected {SLICE_SCHEMA_V1}");
    }
    if tick_rate != TICK_RATE_V1 {
        bail!("unsupported tickRate {tick_rate}; expected {TICK_RATE_V1}");
    }
    Ok(())
}

fn validate_tick(tick: u32, max_ticks: u32, kind: &str) -> Result<()> {
    if tick > max_ticks {
        bail!("{kind} tick {tick} exceeds maxTicks {max_ticks}");
    }
    Ok(())
}

fn validate_path(path: &str) -> Result<()> {
    if path.is_empty() || path.split('.').any(str::is_empty) {
        bail!("assertion path must contain non-empty dot-separated segments");
    }
    Ok(())
}

fn validate_axes(axes: Option<[f32; 2]>, name: &str) -> Result<()> {
    if let Some(axes) = axes {
        if axes
            .into_iter()
            .any(|axis| !axis.is_finite() || !(-1.0..=1.0).contains(&axis))
        {
            bail!("input {name} axes must be finite and normalized to [-1, 1]");
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Default)]
struct TapeState {
    movement: [f32; 2],
    look: [f32; 2],
    actions: BTreeSet<ActionV1>,
}

impl TapeState {
    fn apply(&mut self, frame: &InputFrameV1) {
        if let Some(movement) = frame.r#move {
            self.movement = movement;
        }
        if let Some(look) = frame.look {
            self.look = look;
        }
        if let Some(actions) = &frame.actions {
            self.actions = actions.iter().copied().collect();
        }
    }

    fn sim_input(&self) -> SimInput {
        SimInput {
            move_x: self.movement[0],
            move_y: self.movement[1],
            fire: self.actions.contains(&ActionV1::Fire),
            reload: self.actions.contains(&ActionV1::Reload),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct EventSummary {
    count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_tick: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_tick: Option<u32>,
}

impl EventSummary {
    fn record(&mut self, tick: u32) {
        self.count += 1;
        self.first_tick.get_or_insert(tick);
        self.last_tick = Some(tick);
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureResult {
    tick: u32,
    width: u32,
    height: u32,
    filename: String,
    sha256: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScenarioResultV1 {
    schema: u32,
    scenario: String,
    status: &'static str,
    seed: u32,
    tick_rate: u32,
    ticks_run: u32,
    overrides: BTreeMap<String, Value>,
    #[serde(rename = "final")]
    final_state: Value,
    events: BTreeMap<String, EventSummary>,
    captures: Vec<CaptureResult>,
    metrics: Value,
    failures: Vec<String>,
}

pub fn run(args: &Args) -> Result<()> {
    let requested = args
        .scenario
        .as_deref()
        .context("--scenario needs a value")?;
    let output = args.state_out.clone();
    let result = run_inner(args, requested);
    let document = match result {
        Ok(document) => document,
        Err(error) => ScenarioResultV1 {
            schema: SLICE_SCHEMA_V1,
            scenario: requested.to_owned(),
            status: "failed",
            seed: args.seed_override.unwrap_or(0),
            tick_rate: TICK_RATE_V1,
            ticks_run: 0,
            overrides: requested_overrides(args),
            final_state: json!({}),
            events: BTreeMap::new(),
            captures: Vec::new(),
            metrics: json!({ "gpuInitialized": false }),
            failures: vec![format!("{error:#}")],
        },
    };
    write_result(output.as_deref(), &document)?;
    if document.status == "failed" {
        bail!("scenario '{}' failed", document.scenario);
    }
    Ok(())
}

fn run_inner(args: &Args, requested: &str) -> Result<ScenarioResultV1> {
    let scenario_path = resolve_scenario_path(requested)?;
    let mut scenario = ScenarioV1::load(&scenario_path)?;
    if scenario.name != requested && !requested.ends_with(".json") && !requested.contains('/') {
        bail!(
            "scenario name '{}' does not match requested name '{requested}'",
            scenario.name
        );
    }
    if let Some(seed) = args.seed_override {
        scenario.seed = seed;
    }
    if scenario.seed == 0 {
        bail!("scenario seed must be non-zero");
    }
    if let Some(max_ticks) = args.max_ticks_override {
        if max_ticks == 0 {
            bail!("--max-ticks must be positive");
        }
        scenario.max_ticks = max_ticks;
    }
    scenario.validate()?;

    let tape_path = resolve_input_path(&scenario_path, &scenario.input_tape)?;
    let tape = InputTapeV1::load(&tape_path)?;
    if let Some(frame) = tape.frames.last() {
        if frame.tick > scenario.max_ticks {
            bail!(
                "input frame tick {} exceeds effective maxTicks {}",
                frame.tick,
                scenario.max_ticks
            );
        }
    }

    let mut map_args = args.clone();
    map_args.map = scenario.map.clone();
    let map_path = map_args.resolve_map_path()?;
    let map = pocket3d::bsp::load_map(&map_path, &map_args.wad_dirs())
        .with_context(|| format!("loading scenario map {}", map_path.display()))?;
    let spawn = select_spawn(&map, &scenario.player_spawn)?;
    let mut game = OpenStrike::new(map, spawn.pos, spawn.yaw, 1);
    game.seed = scenario.seed;
    game.rng = Rng::seeded(scenario.seed);
    let mut guest = StrikeGuest::boot(args.size)?;

    let capture_dir = args
        .capture_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("out/scenarios").join(&scenario.name));
    let mut headless = if scenario.captures.is_empty() {
        None
    } else {
        let headless = Headless::new(args.size)?;
        Some(headless)
    };
    if let Some(headless) = &headless {
        game.upload_world(&headless.gpu, &headless.renderer);
        fs::create_dir_all(&capture_dir)
            .with_context(|| format!("creating capture directory {}", capture_dir.display()))?;
    }

    // The boot-time rules queue configuration while the bundle evaluates.
    // One initial guest turn applies it before the tick-0 state is recorded.
    guest.turn(&mut game)?;

    let mut snapshots = BTreeMap::new();
    snapshots.insert(0, snapshot(0, scenario.seed, &game));
    let mut event_summaries = BTreeMap::new();
    let mut captures = Vec::new();
    capture_if_requested(
        0,
        &scenario,
        &capture_dir,
        &mut headless,
        &mut game,
        &mut guest,
        &mut captures,
    )?;

    let mut tape_state = TapeState::default();
    let mut next_frame = 0usize;
    while next_frame < tape.frames.len() && tape.frames[next_frame].tick == 0 {
        tape_state.apply(&tape.frames[next_frame]);
        next_frame += 1;
    }

    for tick in 1..=scenario.max_ticks {
        while next_frame < tape.frames.len() && tape.frames[next_frame].tick == tick {
            tape_state.apply(&tape.frames[next_frame]);
            next_frame += 1;
        }
        game.player.yaw -= tape_state.look[0] * LOOK_RADIANS_PER_TICK;
        game.player.pitch = (game.player.pitch - tape_state.look[1] * LOOK_RADIANS_PER_TICK)
            .clamp(-89f32.to_radians(), 89f32.to_radians());
        let collision = &game.map.collision;
        game.sim
            .tick(collision, TICK_SECONDS, &tape_state.sim_input());
        record_events(tick, &game, &mut event_summaries);
        guest.turn(&mut game)?;
        snapshots.insert(tick, snapshot(tick, scenario.seed, &game));
        capture_if_requested(
            tick,
            &scenario,
            &capture_dir,
            &mut headless,
            &mut game,
            &mut guest,
            &mut captures,
        )?;
    }

    let failures = evaluate_assertions(&scenario, &snapshots, &event_summaries);
    let final_state = snapshots
        .remove(&scenario.max_ticks)
        .context("runner produced no final snapshot")?;
    Ok(ScenarioResultV1 {
        schema: SLICE_SCHEMA_V1,
        scenario: scenario.name,
        status: if failures.is_empty() {
            "passed"
        } else {
            "failed"
        },
        seed: scenario.seed,
        tick_rate: TICK_RATE_V1,
        ticks_run: scenario.max_ticks,
        overrides: requested_overrides(args),
        final_state,
        events: event_summaries,
        captures,
        metrics: json!({
            "gpuInitialized": headless.is_some(),
            "fixedTicks": scenario.max_ticks,
        }),
        failures,
    })
}

fn resolve_scenario_path(requested: &str) -> Result<PathBuf> {
    let direct = PathBuf::from(requested);
    let mut candidates = Vec::new();
    if requested.ends_with(".json") || requested.contains('/') {
        candidates.push(direct);
    } else {
        candidates.push(PathBuf::from("test/scenarios").join(format!("{requested}.json")));
        candidates.push(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../test/scenarios")
                .join(format!("{requested}.json")),
        );
        candidates.push(PathBuf::from("local/scenarios").join(format!("{requested}.json")));
    }
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .with_context(|| format!("could not find scenario '{requested}'"))
}

fn resolve_input_path(scenario_path: &Path, input: &Path) -> Result<PathBuf> {
    let candidates = [
        input.to_path_buf(),
        scenario_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(input),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(input),
    ];
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .with_context(|| format!("could not find input tape '{}'", input.display()))
}

fn select_spawn(map: &pocket3d::bsp::MapData, name: &str) -> Result<pocket3d::bsp::SpawnPoint> {
    let spawn = match name {
        // The original room's named spawn maps to its first player spawn.
        // Temporary baseline maps use this same deterministic selection.
        "player_start" => map.ct_spawns.first().or(map.t_spawns.first()),
        _ => None,
    };
    spawn
        .copied()
        .with_context(|| format!("map has no spawn matching playerSpawn '{name}'"))
}

fn snapshot(tick: u32, seed: u32, game: &OpenStrike) -> Value {
    let facts = game.facts_v1();
    debug_assert_eq!(facts.tick, tick);
    debug_assert_eq!(facts.seed, seed);
    let pos = game.player.state.pos;
    let vel = game.player.state.vel;
    json!({
        "tick": facts.tick,
        "seed": facts.seed,
        "phase": facts.phase.wire_name(),
        "player": {
            "hp": facts.player.hp,
            "alive": facts.player.alive,
            "position": [quantize(pos.x), quantize(pos.y), quantize(pos.z)],
            "velocity": [quantize(vel.x), quantize(vel.y), quantize(vel.z)],
            "speedQ": facts.player.speed_q,
        },
        "weapon": {
            "ammo": facts.weapon.ammo,
            "reserve": facts.weapon.reserve,
            "reloading": facts.weapon.reloading,
            "reloadTicksRemaining": facts.weapon.reload_ticks_remaining,
        },
        "targets": { "alive": facts.targets.alive, "total": facts.targets.total },
        "score": { "wins": facts.score.wins, "losses": facts.score.losses },
    })
}

fn quantize(value: f32) -> f64 {
    (value as f64 * 1000.0).round() / 1000.0
}

fn record_events(tick: u32, game: &OpenStrike, summaries: &mut BTreeMap<String, EventSummary>) {
    for event in &game.events {
        let name = match event {
            GameEvent::ShotFired { .. } => "shotFired",
            GameEvent::TargetHit { .. } => "targetHit",
            GameEvent::TargetDestroyed { .. } => "targetDestroyed",
            GameEvent::PlayerDamaged { .. } => "playerDamaged",
            GameEvent::PlayerDied => "playerDied",
            GameEvent::RoundReset { .. } => "roundReset",
        };
        summaries.entry(name.into()).or_default().record(tick);
    }
}

#[allow(clippy::too_many_arguments)]
fn capture_if_requested(
    tick: u32,
    scenario: &ScenarioV1,
    capture_dir: &Path,
    headless: &mut Option<Headless>,
    game: &mut OpenStrike,
    guest: &mut StrikeGuest,
    results: &mut Vec<CaptureResult>,
) -> Result<()> {
    let Some(capture) = scenario
        .captures
        .iter()
        .find(|capture| capture.tick == tick)
    else {
        return Ok(());
    };
    let headless = headless
        .as_mut()
        .context("capture requested without GPU initialization")?;
    let filename = format!("{}.png", capture.name);
    let path = capture_dir.join(&filename);
    let path_text = path.to_string_lossy();
    headless.shot_with_hud(game, Some(guest), tick as f32 * TICK_SECONDS, &path_text)?;
    let bytes = fs::read(&path).with_context(|| format!("hashing capture {}", path.display()))?;
    results.push(CaptureResult {
        tick,
        width: headless.target.size.0,
        height: headless.target.size.1,
        filename,
        sha256: format!("{:x}", Sha256::digest(bytes)),
    });
    Ok(())
}

fn evaluate_assertions(
    scenario: &ScenarioV1,
    snapshots: &BTreeMap<u32, Value>,
    events: &BTreeMap<String, EventSummary>,
) -> Vec<String> {
    let mut failures = Vec::new();
    for assertion in &scenario.assertions {
        match assertion {
            AssertionV1::Equals { tick, path, value } => {
                let actual = snapshots
                    .get(tick)
                    .and_then(|state| value_at_path(state, path));
                if actual != Some(value) {
                    failures.push(format!(
                        "tick {tick} {path}: expected {value}, got {}",
                        actual.map_or_else(|| "<missing>".into(), Value::to_string)
                    ));
                }
            }
            AssertionV1::Near {
                tick,
                path,
                value,
                tolerance,
            } => {
                let actual = snapshots
                    .get(tick)
                    .and_then(|state| value_at_path(state, path))
                    .and_then(Value::as_f64);
                if actual.is_none_or(|actual| (actual - value).abs() > *tolerance) {
                    failures.push(format!(
                        "tick {tick} {path}: expected {value} ± {tolerance}, got {}",
                        actual.map_or_else(|| "<missing/non-number>".into(), |v| v.to_string())
                    ));
                }
            }
            AssertionV1::EventCount { event, count } => {
                let name = event_name(*event);
                let actual = events.get(name).map_or(0, |summary| summary.count);
                if actual != *count {
                    failures.push(format!(
                        "event {name}: expected count {count}, got {actual}"
                    ));
                }
            }
        }
    }
    failures
}

fn value_at_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.')
        .try_fold(root, |value, segment| match value {
            Value::Object(object) => object.get(segment),
            Value::Array(array) => segment
                .parse::<usize>()
                .ok()
                .and_then(|index| array.get(index)),
            _ => None,
        })
}

fn event_name(event: EventNameV1) -> &'static str {
    match event {
        EventNameV1::ShotFired => "shotFired",
        EventNameV1::TargetHit => "targetHit",
        EventNameV1::TargetDestroyed => "targetDestroyed",
        EventNameV1::PlayerDamaged => "playerDamaged",
        EventNameV1::PlayerDied => "playerDied",
        EventNameV1::RoundReset => "roundReset",
    }
}

fn requested_overrides(args: &Args) -> BTreeMap<String, Value> {
    let mut overrides = BTreeMap::new();
    if let Some(seed) = args.seed_override {
        overrides.insert("seed".into(), json!(seed));
    }
    if let Some(max_ticks) = args.max_ticks_override {
        overrides.insert("maxTicks".into(), json!(max_ticks));
    }
    if let Some(capture_dir) = &args.capture_dir {
        overrides.insert("captureDir".into(), json!(capture_dir));
    }
    overrides
}

fn write_result(path: Option<&Path>, result: &ScenarioResultV1) -> Result<()> {
    let json = serde_json::to_vec_pretty(result)?;
    if let Some(path) = path {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating result directory {}", parent.display()))?;
        }
        fs::write(path, &json).with_context(|| format!("writing result {}", path.display()))?;
    } else {
        println!("{}", String::from_utf8(json).expect("JSON is UTF-8"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scenario(json: &str) -> Result<ScenarioV1> {
        let value: ScenarioV1 = serde_json::from_str(json)?;
        value.validate()?;
        Ok(value)
    }

    fn tape(json: &str) -> Result<InputTapeV1> {
        let value: InputTapeV1 = serde_json::from_str(json)?;
        value.validate()?;
        Ok(value)
    }

    #[test]
    fn accepts_versioned_scenario_and_normalized_change_points() {
        let definition = scenario(
            r#"{
                "schema":1,"name":"slice.hit","map":"slice_test_room","seed":1,
                "tickRate":60,"maxTicks":112,"playerSpawn":"player_start",
                "inputTape":"test/input/slice-hit.json",
                "assertions":[{"type":"eventCount","event":"targetHit","count":1}],
                "captures":[{"tick":96,"name":"hit"}]
            }"#,
        )
        .unwrap();
        assert_eq!(definition.name, "slice.hit");

        let input = tape(
            r#"{"schema":1,"tickRate":60,"frames":[
                {"tick":0,"move":[0,0],"look":[0,0],"actions":[]},
                {"tick":96,"actions":["fire"]},{"tick":97,"actions":[]}
            ]}"#,
        )
        .unwrap();
        assert_eq!(input.frames.len(), 3);
    }

    #[test]
    fn rejects_unknown_fields_versions_actions_and_non_normalized_axes() {
        assert!(tape(r#"{"schema":2,"tickRate":60,"frames":[]}"#).is_err());
        assert!(
            tape(r#"{"schema":1,"tickRate":60,"frames":[{"tick":0,"move":[0,1.1]}]}"#).is_err()
        );
        assert!(
            tape(r#"{"schema":1,"tickRate":60,"frames":[{"tick":0,"actions":["jump"]}]}"#).is_err()
        );
        assert!(tape(r#"{"schema":1,"tickRate":60,"extra":true,"frames":[]}"#).is_err());
    }

    #[test]
    fn rejects_duplicate_or_decreasing_change_points_and_captures() {
        assert!(tape(r#"{"schema":1,"tickRate":60,"frames":[{"tick":2},{"tick":2}]}"#).is_err());
        assert!(
            scenario(
                r#"{
                    "schema":1,"name":"x","map":"m","seed":1,"tickRate":60,
                    "maxTicks":10,"playerSpawn":"p","inputTape":"i",
                    "assertions":[],"captures":[{"tick":3,"name":"a"},{"tick":3,"name":"b"}]
                }"#
            )
            .is_err()
        );
    }

    #[test]
    fn change_points_persist_and_empty_actions_release_fire() {
        let input = tape(
            r#"{"schema":1,"tickRate":60,"frames":[
                {"tick":0,"move":[0.25,1.0],"actions":["fire"]},
                {"tick":4,"actions":[]}
            ]}"#,
        )
        .unwrap();
        let mut state = TapeState::default();
        state.apply(&input.frames[0]);
        assert_eq!(state.sim_input().move_y, 1.0);
        assert!(state.sim_input().fire);
        state.apply(&input.frames[1]);
        assert_eq!(state.sim_input().move_x, 0.25);
        assert!(!state.sim_input().fire);
    }

    #[test]
    fn assertions_read_nested_objects_and_array_components() {
        let scenario = scenario(
            r#"{
                "schema":1,"name":"x","map":"m","seed":1,"tickRate":60,
                "maxTicks":1,"playerSpawn":"player_start","inputTape":"i",
                "assertions":[
                    {"type":"equals","tick":1,"path":"phase","value":"live"},
                    {"type":"near","tick":1,"path":"player.position.0","value":2.0,"tolerance":0.01}
                ]
            }"#,
        )
        .unwrap();
        let snapshots =
            BTreeMap::from([(1, json!({"phase":"live","player":{"position":[2.005,0,0]}}))]);
        assert!(evaluate_assertions(&scenario, &snapshots, &BTreeMap::new()).is_empty());
    }
}
