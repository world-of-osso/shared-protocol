//! Retail creature level scaling (ContentTuning), as TrinityCore master
//! (a352b1fa) implements it. A tuned creature has one native level for its
//! shared health pool, but every unit sees and fights it at its own level
//! clamped to the ContentTuning range. Server combat math and client level
//! text both call `LevelScaling::level_for_target`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::UnitLevel;

/// `MAX_LEVEL` (DBCEnums.h:45).
pub const MAX_LEVEL: u8 = 123;

/// A tuned creature's `UF::UnitData` ContentTuningID, ScalingLevelMin/Max
/// and ScalingLevelDelta (Creature::ApplyLevelScaling, Creature.cpp:3161-3171).
/// Creatures without a ContentTuningID have no `LevelScaling`: every unit
/// sees their `UnitLevel`.
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct LevelScaling {
    pub content_tuning_id: u32,
    /// ContentTuning level range (DB2Manager::GetContentTuningData).
    pub min_level: u8,
    pub max_level: u8,
    /// `creature_template_difficulty.LevelScalingDeltaMin..Max`, rolled per spawn.
    pub delta: i8,
}

impl LevelScaling {
    /// Creature::SelectLevel (Creature.cpp:1611-1618): ScalingLevelMax +
    /// ScalingLevelDelta, clamped to 1..=255. Health, damage and armor are
    /// evaluated at this level; `level_for_target` rescales them per unit.
    pub fn native_level(&self) -> u8 {
        (i32::from(self.max_level) + i32::from(self.delta)).clamp(1, i32::from(u8::MAX)) as u8
    }

    /// Creature::GetLevelForTarget (Creature.cpp:3226-3257): the target's
    /// level clamped to the range, plus the delta, clamped to
    /// 1..=MAX_LEVEL + 3. The player-side `MaxCreatureScalingLevel` and
    /// `ScalingPlayerLevelDelta` (Chromie Time, group scaling) and
    /// `ScalingFactionGroup` are not modelled, so they add nothing.
    pub fn level_for_target(&self, target_level: u8) -> u8 {
        // RoundToInterval (Util.h:97-100): min(max(level, floor), ceil).
        let clamped = target_level.max(self.min_level).min(self.max_level);
        (i32::from(clamped) + i32::from(self.delta)).clamp(1, i32::from(MAX_LEVEL) + 3) as u8
    }
}

/// The level `viewer_level` sees and fights a unit at: its scaled level when
/// tuned, its `UnitLevel` otherwise (Unit::GetLevelForTarget, Unit.h:771).
pub fn level_for_viewer(level: UnitLevel, scaling: Option<&LevelScaling>, viewer_level: u8) -> u8 {
    scaling.map_or(level.0, |scaling| scaling.level_for_target(viewer_level))
}

/// `Trinity::XP::GetGrayLevel` (Formulas.h:70-89): units at or below this
/// level give no experience and show grey.
pub fn gray_level(player_level: u8) -> u8 {
    match player_level {
        0..7 => 0,
        7..35 => {
            let multiples_of_five_from_15 =
                (15..=player_level).filter(|l| l % 5 == 0).count() as u8;
            player_level - 7 - multiples_of_five_from_15 + 1
        }
        _ => player_level - 10,
    }
}

/// Level colour of a unit against the player (FrameXML
/// `GetRelativeDifficultyColor`, Blizzard_UIParent/Mainline/UIParent.lua:2626-2640).
/// The green/grey boundary is the XP grey level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelDifficulty {
    /// Grey.
    Trivial,
    /// Green.
    Standard,
    /// Yellow.
    Difficult,
    /// Orange.
    VeryDifficult,
    /// Red.
    Impossible,
}

impl LevelDifficulty {
    pub fn for_levels(player_level: u8, unit_level: u8) -> Self {
        let diff = i32::from(unit_level) - i32::from(player_level);
        if diff >= 5 {
            Self::Impossible
        } else if diff >= 3 {
            Self::VeryDifficult
        } else if diff >= -4 {
            Self::Difficult
        } else if unit_level > gray_level(player_level) {
            Self::Standard
        } else {
            Self::Trivial
        }
    }

    /// `QuestDifficultyColors` (Blizzard_FrameXMLBase/Constants.lua:210-215), RGB.
    pub fn color(self) -> [f32; 3] {
        match self {
            Self::Impossible => [1.0, 0.1, 0.1],
            Self::VeryDifficult => [1.0, 0.5, 0.25],
            Self::Difficult => [1.0, 0.82, 0.0],
            Self::Standard => [0.25, 0.75, 0.25],
            Self::Trivial => [0.5, 0.5, 0.5],
        }
    }
}

#[cfg(test)]
#[path = "level_scaling_tests.rs"]
mod tests;
