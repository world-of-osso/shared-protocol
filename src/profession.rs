//! Profession skill lines and recipe difficulty.
//!
//! Retail models a profession as a parent `SkillLine` (197 Tailoring) plus one
//! child line per expansion tier (2540 Classic Tailoring, `ParentSkillLineID`
//! 197). Each learned line has a rank and a cap; a recipe is a
//! `SkillLineAbility` row whose `SkillupSkillLineID` names the line it raises.

use serde::{Deserialize, Serialize};

/// Primary professions a character may know (TrinityCore `MaxPrimaryTradeSkill`).
pub const MAX_PRIMARY_PROFESSIONS: usize = 2;

/// One learned skill line: rank `rank` of `max_rank` at tier `step`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, bitcode::Encode, bitcode::Decode,
)]
pub struct ProfessionSkillLine {
    pub skill_line: u32,
    /// Tier index, 1-based (`SKILL_STEP` base points).
    pub step: u16,
    pub rank: u16,
    pub max_rank: u16,
}

/// Retail `Enum.TradeskillRelativeDifficulty` (orange, yellow, green, grey).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecipeDifficulty {
    Optimal,
    Medium,
    Easy,
    Trivial,
}

impl RecipeDifficulty {
    /// Skill-up chance in permille: TrinityCore `SkillChance.Orange/Yellow/Green/Grey`
    /// defaults 100/75/25/0 percent (worldserver.conf.dist).
    pub fn skill_up_chance_permille(self) -> u32 {
        match self {
            Self::Optimal => 1000,
            Self::Medium => 750,
            Self::Easy => 250,
            Self::Trivial => 0,
        }
    }
}

/// Difficulty of a recipe at `rank`, from its `SkillLineAbility` trivial ranks.
///
/// TrinityCore `Player::UpdateCraftSkill` calls `SkillGainChance(rank, high,
/// (high + low) / 2, low)`: grey at `high`, green from the midpoint, yellow from
/// `low`, orange below.
pub fn recipe_difficulty(rank: u16, trivial_low: u16, trivial_high: u16) -> RecipeDifficulty {
    let green = (u32::from(trivial_high) + u32::from(trivial_low)) / 2;
    if rank >= trivial_high {
        RecipeDifficulty::Trivial
    } else if u32::from(rank) >= green {
        RecipeDifficulty::Easy
    } else if rank >= trivial_low {
        RecipeDifficulty::Medium
    } else {
        RecipeDifficulty::Optimal
    }
}

#[cfg(test)]
#[path = "profession_tests.rs"]
mod tests;
