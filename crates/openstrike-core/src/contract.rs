//! Versioned guest/native wire contract for the first vertical slice.
//!
//! Keep validation here so every host rejects the same payloads. Platform
//! adapters are responsible only for translating these values to/from their
//! QuickJS API.

use alloc::vec::Vec;

use crate::sim::Phase;

pub const SLICE_SCHEMA_V1: u32 = 1;
pub const MAX_FACT_SCALARS_V1: usize = 24;
pub const MAX_EVENTS_V1: usize = 16;
pub const MAX_COMMANDS_V1: usize = 8;
pub const TICK_RATE_V1: u32 = 60;
pub const FACT_PATHS_V1: [&str; 15] = [
    "schema",
    "tick",
    "seed",
    "phase",
    "player.hp",
    "player.alive",
    "player.speedQ",
    "weapon.ammo",
    "weapon.reserve",
    "weapon.reloading",
    "weapon.reloadTicksRemaining",
    "targets.alive",
    "targets.total",
    "score.wins",
    "score.losses",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlicePhaseV1 {
    Starting,
    Live,
    Won,
    Lost,
}

impl SlicePhaseV1 {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Live => "live",
            Self::Won => "won",
            Self::Lost => "lost",
        }
    }

    pub fn from_wire_name(name: &str) -> Result<Self, ContractErrorV1> {
        match name {
            "starting" => Ok(Self::Starting),
            "live" => Ok(Self::Live),
            "won" => Ok(Self::Won),
            "lost" => Ok(Self::Lost),
            _ => Err(ContractErrorV1::UnknownPhase),
        }
    }
}

impl From<Phase> for SlicePhaseV1 {
    fn from(phase: Phase) -> Self {
        match phase {
            Phase::Starting => Self::Starting,
            Phase::Live => Self::Live,
            Phase::Ended { won: true } => Self::Won,
            Phase::Ended { won: false } => Self::Lost,
        }
    }
}

impl From<SlicePhaseV1> for Phase {
    fn from(phase: SlicePhaseV1) -> Self {
        match phase {
            SlicePhaseV1::Starting => Self::Starting,
            SlicePhaseV1::Live => Self::Live,
            SlicePhaseV1::Won => Self::Ended { won: true },
            SlicePhaseV1::Lost => Self::Ended { won: false },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerFactsV1 {
    pub hp: i32,
    pub alive: bool,
    pub speed_q: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WeaponFactsV1 {
    pub ammo: u32,
    pub reserve: u32,
    pub reloading: bool,
    pub reload_ticks_remaining: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetFactsV1 {
    pub alive: u32,
    pub total: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScoreFactsV1 {
    pub wins: u32,
    pub losses: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SliceFactsV1 {
    pub schema: u32,
    pub tick: u32,
    pub seed: u32,
    pub phase: SlicePhaseV1,
    pub player: PlayerFactsV1,
    pub weapon: WeaponFactsV1,
    pub targets: TargetFactsV1,
    pub score: ScoreFactsV1,
}

impl SliceFactsV1 {
    /// Number of scalar leaves in the canonical nested wire representation.
    pub const SCALAR_LEAVES: usize = FACT_PATHS_V1.len();

    pub fn validate(&self) -> Result<(), ContractErrorV1> {
        require_schema(self.schema)?;
        if Self::SCALAR_LEAVES > MAX_FACT_SCALARS_V1 {
            return Err(ContractErrorV1::TooManyFactScalars);
        }
        if self.player.hp < 0 || self.targets.alive > self.targets.total {
            return Err(ContractErrorV1::InvalidFacts);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SliceEventV1 {
    ShotFired {
        weapon_id: u32,
        ammo: u32,
    },
    TargetHit {
        target_id: u32,
        damage: u32,
        hp: u32,
        fatal: bool,
    },
    TargetDestroyed {
        target_id: u32,
    },
    PlayerDamaged {
        amount: u32,
        hp: u32,
    },
    PlayerDied,
    RoundReset {
        round: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WeaponConfigV1 {
    pub magazine_capacity: u32,
    pub reserve_capacity: u32,
    pub fire_interval_ticks: u32,
    pub reload_ticks: u32,
    pub damage: u32,
}

impl WeaponConfigV1 {
    pub fn validate(&self) -> Result<(), ContractErrorV1> {
        if self.magazine_capacity == 0
            || self.magazine_capacity > 1_024
            || self.reserve_capacity > 65_535
            || self.fire_interval_ticks == 0
            || self.fire_interval_ticks > 3_600
            || self.reload_ticks > 36_000
            || self.damage == 0
            || self.damage > 1_000_000
        {
            return Err(ContractErrorV1::InvalidWeaponConfig);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetConfigV1 {
    pub health: u32,
}

impl TargetConfigV1 {
    pub fn validate(&self) -> Result<(), ContractErrorV1> {
        if self.health == 0 || self.health > 1_000_000 {
            Err(ContractErrorV1::InvalidTargetConfig)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SliceCommandV1 {
    SetPhase(SlicePhaseV1),
    ResetRound,
    AddWin,
    AddLoss,
    ConfigureWeapon(WeaponConfigV1),
    ConfigureTarget(TargetConfigV1),
}

impl SliceCommandV1 {
    pub fn validate(&self) -> Result<(), ContractErrorV1> {
        match self {
            Self::ConfigureWeapon(config) => config.validate(),
            Self::ConfigureTarget(config) => config.validate(),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliceCommandBatchV1 {
    pub schema: u32,
    pub after_tick: u32,
    pub commands: Vec<SliceCommandV1>,
}

impl SliceCommandBatchV1 {
    pub fn validate(&self, published_tick: u32) -> Result<(), ContractErrorV1> {
        require_schema(self.schema)?;
        if self.after_tick != published_tick {
            return Err(ContractErrorV1::WrongCommandTick);
        }
        if self.commands.len() > MAX_COMMANDS_V1 {
            return Err(ContractErrorV1::TooManyCommands);
        }
        for command in &self.commands {
            command.validate()?;
        }
        Ok(())
    }
}

pub fn validate_event_batch_v1(
    schema: u32,
    events: &[SliceEventV1],
) -> Result<(), ContractErrorV1> {
    require_schema(schema)?;
    if events.len() > MAX_EVENTS_V1 {
        Err(ContractErrorV1::TooManyEvents)
    } else {
        Ok(())
    }
}

fn require_schema(schema: u32) -> Result<(), ContractErrorV1> {
    if schema == SLICE_SCHEMA_V1 {
        Ok(())
    } else {
        Err(ContractErrorV1::UnsupportedSchema)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractErrorV1 {
    UnsupportedSchema,
    UnknownPhase,
    TooManyFactScalars,
    TooManyEvents,
    TooManyCommands,
    WrongCommandTick,
    InvalidFacts,
    InvalidWeaponConfig,
    InvalidTargetConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weapon() -> WeaponConfigV1 {
        WeaponConfigV1 {
            magazine_capacity: 8,
            reserve_capacity: 24,
            fire_interval_ticks: 6,
            reload_ticks: 90,
            damage: 40,
        }
    }

    #[test]
    fn phase_wire_names_are_closed_and_round_trip() {
        for phase in [
            SlicePhaseV1::Starting,
            SlicePhaseV1::Live,
            SlicePhaseV1::Won,
            SlicePhaseV1::Lost,
        ] {
            assert_eq!(SlicePhaseV1::from_wire_name(phase.wire_name()), Ok(phase));
        }
        assert_eq!(
            SlicePhaseV1::from_wire_name("menu"),
            Err(ContractErrorV1::UnknownPhase)
        );
        assert_eq!(
            Phase::from(SlicePhaseV1::from(Phase::Ended { won: true })),
            Phase::Ended { won: true }
        );
    }

    #[test]
    fn command_batch_preserves_order_and_tick_causality() {
        let commands = vec![
            SliceCommandV1::ConfigureWeapon(weapon()),
            SliceCommandV1::SetPhase(SlicePhaseV1::Live),
            SliceCommandV1::AddWin,
        ];
        let batch = SliceCommandBatchV1 {
            schema: SLICE_SCHEMA_V1,
            after_tick: 12,
            commands: commands.clone(),
        };
        assert_eq!(batch.validate(12), Ok(()));
        assert_eq!(batch.commands, commands);
        assert_eq!(batch.validate(11), Err(ContractErrorV1::WrongCommandTick));
    }

    #[test]
    fn limits_and_configurations_fail_instead_of_clamping() {
        let oversized = SliceCommandBatchV1 {
            schema: SLICE_SCHEMA_V1,
            after_tick: 0,
            commands: vec![SliceCommandV1::ResetRound; MAX_COMMANDS_V1 + 1],
        };
        assert_eq!(oversized.validate(0), Err(ContractErrorV1::TooManyCommands));

        let mut invalid = weapon();
        invalid.fire_interval_ticks = 0;
        assert_eq!(
            invalid.validate(),
            Err(ContractErrorV1::InvalidWeaponConfig)
        );
        assert_eq!(
            TargetConfigV1 { health: 0 }.validate(),
            Err(ContractErrorV1::InvalidTargetConfig)
        );
    }

    #[test]
    fn schema_and_event_limit_are_shared_host_gates() {
        assert_eq!(SliceFactsV1::SCALAR_LEAVES, 15);
        assert!(FACT_PATHS_V1.len() <= MAX_FACT_SCALARS_V1);
        let events = vec![SliceEventV1::PlayerDied; MAX_EVENTS_V1];
        assert_eq!(validate_event_batch_v1(SLICE_SCHEMA_V1, &events), Ok(()));
        assert_eq!(
            validate_event_batch_v1(SLICE_SCHEMA_V1 + 1, &events),
            Err(ContractErrorV1::UnsupportedSchema)
        );
        let too_many = vec![SliceEventV1::PlayerDied; MAX_EVENTS_V1 + 1];
        assert_eq!(
            validate_event_batch_v1(SLICE_SCHEMA_V1, &too_many),
            Err(ContractErrorV1::TooManyEvents)
        );
    }
}
