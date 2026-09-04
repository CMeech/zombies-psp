//! Construct the shared simulation from a parsed cooked map and replay the
//! product configuration that the JS guest emitted before a world existed.

use alloc::vec::Vec;

use openstrike_core::StrikeSim;
use openstrike_core::sim::Command;
use pocket3d_bsp::cooked::CookedMap;

pub fn from_map(map: &CookedMap<'_>, boot_config: &[Command]) -> Result<StrikeSim, &'static str> {
    let spawn = map.ct_spawns.first().ok_or("map has no CT spawns")?;
    let target_spawn = (map.name == "slice_test_room")
        .then(|| map.t_spawns.first().copied())
        .flatten();
    let bot_spawns = if target_spawn.is_some() {
        Vec::new()
    } else if map.t_spawns.is_empty() {
        map.ct_spawns.clone()
    } else {
        map.t_spawns.clone()
    };
    let mut sim = StrikeSim::new(spawn.pos, spawn.yaw, bot_spawns, 3);
    if let Some(target) = target_spawn {
        sim.set_stationary_target(target.pos);
    }
    for command in boot_config {
        sim.apply(command.clone(), 0);
    }
    Ok(sim)
}
