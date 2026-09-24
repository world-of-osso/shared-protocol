//! Rested XP pool. XP formulas and level thresholds are server-side
//! (game-server `experience.rs`, Retail gtXp/QuestXP tables).

/// Rested XP accumulation rate: fraction of a level per 8 hours in a rest area.
const RESTED_RATE_PER_8H: f32 = 0.05;
/// Maximum rested XP as a fraction of 1.5 levels.
const RESTED_MAX_LEVELS: f32 = 1.5;

/// Rested XP state for a player.
///
/// Accumulates while in a rest area (city/inn) or logged off.
/// When active, kill XP is doubled until the rested pool is depleted.
/// Cap: 1.5 levels worth of XP.
///
/// Ref: AzerothCore `Player::SetRestBonus()`, `GetXPRestBonus()`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RestedXp {
    /// Current rested XP pool.
    pub amount: u32,
    /// Maximum rested XP (1.5 levels of XP, recalculated on level-up).
    pub max: u32,
    /// Whether the player is currently in a rest area.
    pub in_rest_area: bool,
}

impl RestedXp {
    /// Create a new rested XP tracker for a given XP-to-level value.
    pub fn new(xp_to_level: u32) -> Self {
        Self {
            amount: 0,
            max: (xp_to_level as f32 * RESTED_MAX_LEVELS) as u32,
            in_rest_area: false,
        }
    }

    /// Update the max when the player levels up.
    pub fn update_max(&mut self, xp_to_level: u32) {
        self.max = (xp_to_level as f32 * RESTED_MAX_LEVELS) as u32;
        self.amount = self.amount.min(self.max);
    }

    /// Accumulate rested XP from time spent in a rest area.
    ///
    /// `hours` is real time spent resting (in city/inn or logged off in rest area).
    pub fn accumulate(&mut self, hours: f32, xp_to_level: u32) {
        let gain_per_8h = xp_to_level as f32 * RESTED_RATE_PER_8H;
        let gained = (gain_per_8h * hours / 8.0) as u32;
        self.amount = (self.amount + gained).min(self.max);
    }

    /// Apply rested bonus to a kill XP amount.
    ///
    /// Returns `(total_xp, rested_consumed)`. Total XP is up to 2x the base,
    /// limited by available rested pool.
    pub fn apply_bonus(&mut self, base_xp: u32) -> (u32, u32) {
        if self.amount == 0 {
            return (base_xp, 0);
        }
        let bonus = base_xp.min(self.amount);
        self.amount -= bonus;
        (base_xp + bonus, bonus)
    }

    /// Whether the player has any rested XP.
    pub fn is_rested(&self) -> bool {
        self.amount > 0
    }

    /// Rested XP as a fraction of the current level (0.0–1.5).
    pub fn rested_levels(&self, xp_to_level: u32) -> f32 {
        if xp_to_level == 0 {
            return 0.0;
        }
        self.amount as f32 / xp_to_level as f32
    }
}

#[cfg(test)]
#[path = "xp_tests.rs"]
mod tests;
