//! The guest: one QuickJS realm running the OpenStrike product bundle —
//! gameplay rules (the base game is the first mod) plus the Solid JSX HUD.
//!
//! Two surfaces are mounted (RUNTIMES.md):
//!   - `ui` — the PocketJS 2D runtime (pocket-ui-wgpu), composited over
//!     the 3D frame as the HUD;
//!   - `strike` — this game's vocabulary. Facts flow guest-ward as per-tick
//!     event batches (`strike.__dispatch(state, events)`); intent flows
//!     host-ward as commands queued by ops and applied after the guest
//!     turn. No shared state, no re-entrancy.
//!
//! Turn order per fixed tick (Law 3 — one guest turn per tick):
//!   game.tick() → __dispatch(state, events) → frame(buttons) → ui.tick()
//!   → drain commands into the game.

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use pocket_mod::Guest;
use pocket_mod::qjs::{Array, CatchResultExt, Function, Object};
use pocket_ui_wgpu::{Blit, UiRenderer, UiSurface};
use pocket3d::gpu::{Gpu, OffscreenTarget};

use openstrike_core::{
    MAX_EVENTS_V1, SliceCommandBatchV1, SliceCommandV1, SlicePhaseV1, TargetConfigV1,
    WeaponConfigV1,
};

use crate::game::{GameEvent, OpenStrike};

pub struct StrikeGuest {
    guest: Guest,
    ui: UiSurface,
    /// Logical UI size (the core's viewport).
    ui_size: (u32, u32),
    gfx: Option<OverlayGfx>,
}

struct OverlayGfx {
    renderer: UiRenderer,
    offscreen: OffscreenTarget,
    blit: Blit,
    target_format: wgpu::TextureFormat,
}

/// Locate the PSP-baseline product bundle (`dist/pocket/psp`).
pub fn find_bundle() -> Result<(PathBuf, PathBuf)> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(d) = std::env::var_os("OPENSTRIKE_UI_DIST") {
        roots.push(PathBuf::from(d));
    }
    roots.push(PathBuf::from("dist/pocket/psp"));
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dist/pocket/psp"));
    for root in &roots {
        let js = root.join("openstrike.js");
        let pak = root.join("openstrike.pak");
        if js.is_file() && pak.is_file() {
            return Ok((js, pak));
        }
    }
    Err(anyhow!(
        "HUD/rules bundle not found — build it first: `bun run build:ui` \
         (searched dist/pocket/psp next to the repo root; override with OPENSTRIKE_UI_DIST)"
    ))
}

impl StrikeGuest {
    /// Boot the realm: feed the pak, mount `ui` + `strike`, eval the bundle.
    /// `ui_size` is the logical HUD resolution (window logical size).
    pub fn boot(ui_size: (u32, u32)) -> Result<StrikeGuest> {
        let (js_path, pak_path) = find_bundle()?;
        let bundle = std::fs::read_to_string(&js_path)
            .with_context(|| format!("reading {}", js_path.display()))?;
        let pak =
            std::fs::read(&pak_path).with_context(|| format!("reading {}", pak_path.display()))?;

        let ui = UiSurface::new((ui_size.0 as f32, ui_size.1 as f32));
        // The desktop runtime deliberately re-hosts the PSP product bundle.
        // Publish that plan contract before mount so PocketJS can retain its
        // target/ABI guard instead of treating this as a plan-less desktop UI.
        ui.set_identity("psp", 1);
        ui.feed_pak(&pak);
        let guest = Guest::new()?;
        ui.mount(&guest)?;

        mount_strike(&guest)?;

        guest.eval("openstrike", &bundle)?;
        if !guest.has_frame() {
            return Err(anyhow!(
                "bundle evaluated but installed no frame() — HUD missing?"
            ));
        }
        log::info!(
            "guest: booted {} ({} bytes js) at {}x{}",
            js_path.display(),
            bundle.len(),
            ui_size.0,
            ui_size.1
        );
        Ok(StrikeGuest {
            guest,
            ui,
            ui_size,
            gfx: None,
        })
    }

    /// One guest turn for one game tick.
    pub fn turn(&self, game: &mut OpenStrike) -> Result<()> {
        let events = std::mem::take(&mut game.events);
        if events.len() > MAX_EVENTS_V1 {
            return Err(anyhow!(
                "slice event batch at tick {} has {} entries; limit is {}",
                game.tick,
                events.len(),
                MAX_EVENTS_V1
            ));
        }
        self.guest.with(|ctx| -> Result<()> {
            let strike: Object = ctx
                .globals()
                .get("strike")
                .context("strike surface missing")?;
            let Ok(dispatch) = strike.get::<_, Function>("__dispatch") else {
                return Ok(()); // no SDK loaded — state simply doesn't flow
            };
            let state = build_state(&ctx, game)?;
            let batch = Array::new(ctx.clone())?;
            for (i, e) in events.iter().enumerate() {
                batch.set(i, build_event(&ctx, e)?)?;
            }
            dispatch
                .call::<_, ()>((state, batch))
                .catch(&ctx)
                .map_err(|e| anyhow!("strike.__dispatch threw: {e}"))?;
            Ok(())
        })?;
        self.guest.frame(0)?;
        self.ui.tick();
        let batch = self.take_commands(game.tick)?;
        batch.validate(game.tick).map_err(|error| {
            anyhow!(
                "invalid slice command batch at tick {}: {error:?}",
                game.tick
            )
        })?;
        for command in batch.commands {
            game.sim.apply_slice_command(command, game.bot_walk_clip);
        }
        Ok(())
    }

    fn take_commands(&self, published_tick: u32) -> Result<SliceCommandBatchV1> {
        self.guest.with(|ctx| -> Result<SliceCommandBatchV1> {
            let strike: Object = ctx
                .globals()
                .get("strike")
                .context("strike surface missing")?;
            let take: Function = strike
                .get("__takeCommands")
                .context("strike.__takeCommands missing")?;
            let batch: Object = take
                .call((published_tick,))
                .catch(&ctx)
                .map_err(|error| anyhow!("strike.__takeCommands threw: {error}"))?;
            parse_command_batch(&batch)
        })
    }

    /// Render the HUD over `view` (`target_px` physical pixels): the UI draws
    /// 1:1 at its logical size offscreen, then composites with a linear blit
    /// (hidpi swapchains scale smoothly).
    pub fn render_overlay(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        target_format: wgpu::TextureFormat,
    ) -> Result<()> {
        if self
            .gfx
            .as_ref()
            .is_none_or(|g| g.target_format != target_format)
        {
            let offscreen = OffscreenTarget::new(gpu, self.ui_size.0, self.ui_size.1);
            let blit = Blit::new(
                gpu,
                &offscreen.view,
                target_format,
                wgpu::FilterMode::Linear,
                true,
            );
            self.gfx = Some(OverlayGfx {
                renderer: UiRenderer::new(gpu, pocket3d::gpu::OFFSCREEN_FORMAT),
                offscreen,
                blit,
                target_format,
            });
        }
        let gfx = self.gfx.as_mut().unwrap();
        let ui_size = self.ui_size;
        self.ui.with_ui(|ui| {
            gfx.renderer.render(
                gpu,
                ui,
                encoder,
                &gfx.offscreen.view,
                ui_size,
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            )
        })?;
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hud composite"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            gfx.blit.draw(&mut pass);
        }
        Ok(())
    }
}

fn build_state<'js>(
    ctx: &pocket_mod::qjs::Ctx<'js>,
    game: &OpenStrike,
) -> pocket_mod::qjs::Result<Object<'js>> {
    let facts = game.facts_v1();
    let o = Object::new(ctx.clone())?;
    o.set("schema", facts.schema)?;
    o.set("tick", facts.tick)?;
    o.set("seed", facts.seed)?;
    o.set("time", game.time as f64)?;
    o.set("phase", facts.phase.wire_name())?;

    let player = Object::new(ctx.clone())?;
    player.set("hp", facts.player.hp)?;
    player.set("alive", facts.player.alive)?;
    player.set("speedQ", facts.player.speed_q)?;
    o.set("player", player)?;

    let weapon = Object::new(ctx.clone())?;
    weapon.set("ammo", facts.weapon.ammo)?;
    weapon.set("reserve", facts.weapon.reserve)?;
    weapon.set("reloading", facts.weapon.reloading)?;
    weapon.set("reloadTicksRemaining", facts.weapon.reload_ticks_remaining)?;
    o.set("weapon", weapon)?;

    let targets = Object::new(ctx.clone())?;
    targets.set("alive", facts.targets.alive)?;
    targets.set("total", facts.targets.total)?;
    o.set("targets", targets)?;

    let score = Object::new(ctx.clone())?;
    score.set("wins", facts.score.wins)?;
    score.set("losses", facts.score.losses)?;
    o.set("score", score)?;

    // Temporary aliases consumed by the imported HUD during migration.
    o.set("hp", facts.player.hp)?;
    o.set("alive", facts.player.alive)?;
    o.set("ammo", facts.weapon.ammo)?;
    o.set("reserve", facts.weapon.reserve)?;
    o.set("reloading", facts.weapon.reloading)?;
    let reload_frac = if game.weapon.reloading() {
        1.0 - (game.weapon.reload_left / game.weapon.cfg.reload_time).clamp(0.0, 1.0)
    } else {
        0.0
    };
    o.set("reloadFrac", reload_frac as f64)?;
    o.set("aliveBots", facts.targets.alive)?;
    o.set("totalBots", facts.targets.total)?;
    o.set("wins", facts.score.wins)?;
    o.set("losses", facts.score.losses)?;
    let v = game.player.state.vel;
    o.set("speed", ((v.x * v.x + v.z * v.z).sqrt()) as f64)?;
    Ok(o)
}

fn build_event<'js>(
    ctx: &pocket_mod::qjs::Ctx<'js>,
    e: &GameEvent,
) -> pocket_mod::qjs::Result<Object<'js>> {
    let o = Object::new(ctx.clone())?;
    match e {
        GameEvent::ShotFired { weapon_id, ammo } => {
            o.set("type", "shotFired")?;
            o.set("weaponId", *weapon_id)?;
            o.set("ammo", *ammo)?;
        }
        GameEvent::TargetHit {
            target_id,
            damage,
            hp,
            fatal,
        } => {
            o.set("type", "targetHit")?;
            o.set("targetId", *target_id)?;
            o.set("damage", *damage)?;
            o.set("hp", *hp)?;
            o.set("fatal", *fatal)?;
        }
        GameEvent::TargetDestroyed { target_id } => {
            o.set("type", "targetDestroyed")?;
            o.set("targetId", *target_id)?;
        }
        GameEvent::PlayerDamaged { amount, hp } => {
            o.set("type", "playerDamaged")?;
            o.set("amount", *amount)?;
            o.set("hp", *hp)?;
        }
        GameEvent::PlayerDied => o.set("type", "playerDied")?,
        GameEvent::RoundReset { round } => {
            o.set("type", "roundReset")?;
            o.set("round", *round)?;
        }
    }
    Ok(o)
}

/// Mount only native lifecycle operations. Simulation intent returns through
/// the SDK-installed `__takeCommands` batch after each guest turn.
fn mount_strike(guest: &Guest) -> Result<()> {
    guest.mount("strike", |ctx, ns| {
        macro_rules! op {
            ($name:literal, $f:expr) => {
                ns.set($name, Function::new(ctx.clone(), $f)?)?;
            };
        }

        // Menu-host vocabulary (surface parity with the PSP EBOOT). The
        // desktop build pre-loads its map from the CLI and never enters the
        // menu, so these are honest no-ops and the catalogue is empty.
        ns.set("maps", Vec::<String>::new())?;
        op!("loadMap", move |_i: i32| {
            log::warn!("strike.loadMap: desktop pre-loads its map (--map)");
        });
        op!("toMenu", move || {
            log::warn!("strike.toMenu: no menu on the desktop host (exit and rerun)");
        });

        Ok(())
    })
}

fn parse_command_batch(batch: &Object) -> Result<SliceCommandBatchV1> {
    let commands: Array = batch
        .get("commands")
        .context("command batch commands missing")?;
    let mut parsed = Vec::with_capacity(commands.len());
    for index in 0..commands.len() {
        let command: Object = commands.get(index)?;
        let kind: String = command.get("type")?;
        parsed.push(match kind.as_str() {
            "setPhase" => SliceCommandV1::SetPhase(
                SlicePhaseV1::from_wire_name(&command.get::<_, String>("phase")?)
                    .map_err(|error| anyhow!("command {index} has invalid phase: {error:?}"))?,
            ),
            "resetRound" => SliceCommandV1::ResetRound,
            "addWin" => SliceCommandV1::AddWin,
            "addLoss" => SliceCommandV1::AddLoss,
            "configureWeapon" => {
                let config: Object = command.get("config")?;
                SliceCommandV1::ConfigureWeapon(WeaponConfigV1 {
                    magazine_capacity: config.get("magazineCapacity")?,
                    reserve_capacity: config.get("reserveCapacity")?,
                    fire_interval_ticks: config.get("fireIntervalTicks")?,
                    reload_ticks: config.get("reloadTicks")?,
                    damage: config.get("damage")?,
                })
            }
            "configureTarget" => {
                let config: Object = command.get("config")?;
                SliceCommandV1::ConfigureTarget(TargetConfigV1 {
                    health: config.get("health")?,
                })
            }
            _ => return Err(anyhow!("command {index} has unknown type '{kind}'")),
        });
    }
    Ok(SliceCommandBatchV1 {
        schema: batch.get("schema")?,
        after_tick: batch.get("afterTick")?,
        commands: parsed,
    })
}
