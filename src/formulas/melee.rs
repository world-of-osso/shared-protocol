/// Melee hit table outcome from a single roll.
///
/// Order matches WoW's single-roll table: miss → dodge → parry → glancing →
/// block → crit → hit. Ref: AzerothCore `RollMeleeOutcomeAgainst()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeleeOutcome {
    Miss,
    Dodge,
    Parry,
    Glancing,
    Block,
    Crit,
    Hit,
}

/// Input chances for the melee hit table, each as a value in 0..=10000
/// (hundredths of a percent, matching AzerothCore's internal scale).
///
/// Example: 300 = 3.00% miss chance.
#[derive(Debug, Clone, Copy, Default)]
pub struct MeleeHitChances {
    pub miss: u32,
    pub dodge: u32,
    pub parry: u32,
    pub glancing: u32,
    pub block: u32,
    pub crit: u32,
}

/// Resolve a melee attack outcome using the single-roll hit table.
///
/// `roll` must be in 0..10000.  The table is evaluated in priority order:
/// miss → dodge → parry → glancing → block → crit → hit.
///
/// Each chance occupies a contiguous range on the 0–10000 number line.
/// The first range the roll falls into determines the outcome.
pub fn resolve_melee_outcome(chances: &MeleeHitChances, roll: u32) -> MeleeOutcome {
    let mut threshold = 0u32;

    threshold += chances.miss;
    if roll < threshold {
        return MeleeOutcome::Miss;
    }

    threshold += chances.dodge;
    if roll < threshold {
        return MeleeOutcome::Dodge;
    }

    threshold += chances.parry;
    if roll < threshold {
        return MeleeOutcome::Parry;
    }

    threshold += chances.glancing;
    if roll < threshold {
        return MeleeOutcome::Glancing;
    }

    threshold += chances.block;
    if roll < threshold {
        return MeleeOutcome::Block;
    }

    threshold += chances.crit;
    if roll < threshold {
        return MeleeOutcome::Crit;
    }

    MeleeOutcome::Hit
}

/// Base melee miss chance in hundredths of percent (3.00% = 300).
const BASE_MISS_CHANCE: i32 = 300;
/// Additional miss per level the target is above the attacker (1.00% = 100).
const MISS_PER_LEVEL_DIFF: i32 = 100;

/// Calculate melee miss chance based on attacker and target levels.
///
/// Returns a value in 0..=10000 (hundredths of percent) for use in
/// `MeleeHitChances::miss`. Clamped to 0 minimum (can't go negative).
///
/// - Equal level: 3%
/// - Target +1: 4%, +2: 5%, +3: 6%
/// - Attacker higher: 2%, 1%, 0%
pub fn miss_chance(attacker_level: u8, target_level: u8) -> u32 {
    let level_diff = target_level as i32 - attacker_level as i32;
    let chance = BASE_MISS_CHANCE + level_diff * MISS_PER_LEVEL_DIFF;
    chance.max(0) as u32
}

// --- Glancing blows ---

/// Maximum glancing blow chance (40% = 4000 in 0..10000 scale).
const MAX_GLANCING_CHANCE: u32 = 4000;
/// Max level difference for glancing damage reduction.
const MAX_GLANCING_LEVEL_DIFF: i32 = 3;
/// Damage reduction per level difference (10%).
const GLANCING_REDUCTION_PER_LEVEL: f32 = 0.1;

/// Glancing blow chance for auto-attacks vs higher-level mobs.
///
/// Returns 0 if the target is same level or lower (glancing can't happen).
/// Otherwise `(10 + level_diff * 5) * 100`, capped at 4000 (40%).
/// Value in 0..=10000 for `MeleeHitChances::glancing`.
pub fn glancing_chance(attacker_level: u8, target_level: u8) -> u32 {
    if target_level <= attacker_level {
        return 0;
    }
    let level_diff = (target_level - attacker_level) as u32;
    let chance = (10 + level_diff * 5) * 100;
    chance.min(MAX_GLANCING_CHANCE)
}

/// Damage multiplier for a glancing blow.
///
/// `1.0 - min(level_diff, 3) * 0.1`:
/// - +1 level: 0.9 (90% damage)
/// - +2 levels: 0.8 (80% damage)
/// - +3+ levels: 0.7 (70% damage)
///
/// Returns 1.0 if target is same level or lower (shouldn't glance).
pub fn glancing_damage_multiplier(attacker_level: u8, target_level: u8) -> f32 {
    if target_level <= attacker_level {
        return 1.0;
    }
    let level_diff = (target_level as i32 - attacker_level as i32).min(MAX_GLANCING_LEVEL_DIFF);
    1.0 - level_diff as f32 * GLANCING_REDUCTION_PER_LEVEL
}

// --- Critical strikes ---

/// Base melee crit damage multiplier (200% = double damage).
const BASE_MELEE_CRIT_MULTIPLIER: f32 = 2.0;

/// Melee critical strike damage multiplier.
///
/// Base is 2.0× (double damage). `aura_modifier` is an additive bonus from
/// talents/buffs (e.g. 0.1 for a talent that adds 10% crit damage → 2.1×).
pub fn crit_damage_multiplier(aura_modifier: f32) -> f32 {
    BASE_MELEE_CRIT_MULTIPLIER + aura_modifier
}

/// Apply crit multiplier to a damage value.
pub fn apply_crit(damage: f32, aura_modifier: f32) -> f32 {
    damage * crit_damage_multiplier(aura_modifier)
}

// --- Block ---

/// Apply block: subtract shield block value from damage, minimum 0.
pub fn apply_block(damage: f32, block_value: f32) -> f32 {
    (damage - block_value).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_chances() -> MeleeHitChances {
        MeleeHitChances {
            miss: 300,  // 3%
            dodge: 500, // 5%
            parry: 500, // 5%
            glancing: 0,
            block: 0,
            crit: 1000, // 10%
        }
    }

    #[test]
    fn roll_miss() {
        let c = default_chances();
        assert_eq!(resolve_melee_outcome(&c, 0), MeleeOutcome::Miss);
        assert_eq!(resolve_melee_outcome(&c, 299), MeleeOutcome::Miss);
    }

    #[test]
    fn roll_dodge() {
        let c = default_chances();
        // dodge range: 300..800
        assert_eq!(resolve_melee_outcome(&c, 300), MeleeOutcome::Dodge);
        assert_eq!(resolve_melee_outcome(&c, 799), MeleeOutcome::Dodge);
    }

    #[test]
    fn roll_parry() {
        let c = default_chances();
        // parry range: 800..1300
        assert_eq!(resolve_melee_outcome(&c, 800), MeleeOutcome::Parry);
        assert_eq!(resolve_melee_outcome(&c, 1299), MeleeOutcome::Parry);
    }

    #[test]
    fn roll_crit() {
        let c = default_chances();
        // crit range: 1300..2300 (glancing=0, block=0)
        assert_eq!(resolve_melee_outcome(&c, 1300), MeleeOutcome::Crit);
        assert_eq!(resolve_melee_outcome(&c, 2299), MeleeOutcome::Crit);
    }

    #[test]
    fn roll_hit() {
        let c = default_chances();
        // everything above 2300
        assert_eq!(resolve_melee_outcome(&c, 2300), MeleeOutcome::Hit);
        assert_eq!(resolve_melee_outcome(&c, 9999), MeleeOutcome::Hit);
    }

    #[test]
    fn roll_glancing() {
        let c = MeleeHitChances {
            miss: 300,
            dodge: 0,
            parry: 0,
            glancing: 2000, // 20%
            block: 0,
            crit: 1000,
        };
        // glancing range: 300..2300
        assert_eq!(resolve_melee_outcome(&c, 300), MeleeOutcome::Glancing);
        assert_eq!(resolve_melee_outcome(&c, 2299), MeleeOutcome::Glancing);
    }

    #[test]
    fn roll_block() {
        let c = MeleeHitChances {
            miss: 0,
            dodge: 0,
            parry: 0,
            glancing: 0,
            block: 1500, // 15%
            crit: 1000,
        };
        assert_eq!(resolve_melee_outcome(&c, 0), MeleeOutcome::Block);
        assert_eq!(resolve_melee_outcome(&c, 1499), MeleeOutcome::Block);
        assert_eq!(resolve_melee_outcome(&c, 1500), MeleeOutcome::Crit);
    }

    #[test]
    fn all_zero_chances_always_hit() {
        let c = MeleeHitChances::default();
        assert_eq!(resolve_melee_outcome(&c, 0), MeleeOutcome::Hit);
        assert_eq!(resolve_melee_outcome(&c, 5000), MeleeOutcome::Hit);
    }

    #[test]
    fn full_table_boundaries() {
        let c = MeleeHitChances {
            miss: 300,
            dodge: 500,
            parry: 400,
            glancing: 1000,
            block: 500,
            crit: 1500,
        };
        // miss: 0..300
        assert_eq!(resolve_melee_outcome(&c, 299), MeleeOutcome::Miss);
        // dodge: 300..800
        assert_eq!(resolve_melee_outcome(&c, 300), MeleeOutcome::Dodge);
        // parry: 800..1200
        assert_eq!(resolve_melee_outcome(&c, 800), MeleeOutcome::Parry);
        // glancing: 1200..2200
        assert_eq!(resolve_melee_outcome(&c, 1200), MeleeOutcome::Glancing);
        // block: 2200..2700
        assert_eq!(resolve_melee_outcome(&c, 2200), MeleeOutcome::Block);
        // crit: 2700..4200
        assert_eq!(resolve_melee_outcome(&c, 2700), MeleeOutcome::Crit);
        // hit: 4200+
        assert_eq!(resolve_melee_outcome(&c, 4200), MeleeOutcome::Hit);
    }

    #[test]
    fn crit_pushes_off_table() {
        // If miss+dodge+parry+glancing+block+crit >= 10000, no room for hit
        let c = MeleeHitChances {
            miss: 2000,
            dodge: 2000,
            parry: 2000,
            glancing: 2000,
            block: 1000,
            crit: 1000,
        };
        // Total = 10000, so roll 9999 → crit (last slot)
        assert_eq!(resolve_melee_outcome(&c, 9999), MeleeOutcome::Crit);
    }

    // --- Miss chance tests ---

    #[test]
    fn miss_equal_level() {
        assert_eq!(miss_chance(80, 80), 300); // 3%
    }

    #[test]
    fn miss_target_higher() {
        assert_eq!(miss_chance(80, 81), 400); // 4%
        assert_eq!(miss_chance(80, 83), 600); // 6%
    }

    #[test]
    fn miss_attacker_higher() {
        assert_eq!(miss_chance(80, 79), 200); // 2%
        assert_eq!(miss_chance(80, 78), 100); // 1%
    }

    #[test]
    fn miss_clamps_to_zero() {
        // attacker 10 levels above → would be -7%, clamped to 0
        assert_eq!(miss_chance(80, 70), 0);
    }

    #[test]
    fn miss_low_levels() {
        assert_eq!(miss_chance(1, 1), 300);
        assert_eq!(miss_chance(1, 3), 500);
    }

    // --- Glancing blow tests ---

    #[test]
    fn glancing_chance_equal_level() {
        assert_eq!(glancing_chance(80, 80), 0);
    }

    #[test]
    fn glancing_chance_attacker_higher() {
        assert_eq!(glancing_chance(80, 79), 0);
    }

    #[test]
    fn glancing_chance_target_1_above() {
        // (10 + 1*5) * 100 = 1500
        assert_eq!(glancing_chance(80, 81), 1500);
    }

    #[test]
    fn glancing_chance_target_3_above() {
        // (10 + 3*5) * 100 = 2500
        assert_eq!(glancing_chance(80, 83), 2500);
    }

    #[test]
    fn glancing_chance_caps_at_40_percent() {
        // (10 + 10*5) * 100 = 6000, capped to 4000
        assert_eq!(glancing_chance(70, 80), 4000);
    }

    #[test]
    fn glancing_multiplier_equal_level() {
        assert_eq!(glancing_damage_multiplier(80, 80), 1.0);
    }

    #[test]
    fn glancing_multiplier_by_level_diff() {
        assert!((glancing_damage_multiplier(80, 81) - 0.9).abs() < 0.001);
        assert!((glancing_damage_multiplier(80, 82) - 0.8).abs() < 0.001);
        assert!((glancing_damage_multiplier(80, 83) - 0.7).abs() < 0.001);
    }

    #[test]
    fn glancing_multiplier_caps_at_3_levels() {
        // +5 levels still 0.7 (capped at 3)
        assert!((glancing_damage_multiplier(75, 80) - 0.7).abs() < 0.001);
    }

    // --- Critical strike tests ---

    #[test]
    fn crit_damage_base_multiplier() {
        assert_eq!(crit_damage_multiplier(0.0), 2.0);
    }

    #[test]
    fn crit_damage_with_aura_bonus() {
        // Talent adding 10% crit damage
        assert!((crit_damage_multiplier(0.1) - 2.1).abs() < 0.001);
    }

    #[test]
    fn apply_crit_doubles_damage() {
        assert_eq!(apply_crit(100.0, 0.0), 200.0);
    }

    #[test]
    fn apply_crit_with_modifier() {
        // 100 damage, 2.3x crit (0.3 aura bonus)
        assert!((apply_crit(100.0, 0.3) - 230.0).abs() < 0.01);
    }

    // --- Block tests ---

    #[test]
    fn block_reduces_damage() {
        assert_eq!(apply_block(500.0, 200.0), 300.0);
    }

    #[test]
    fn block_clamps_to_zero() {
        assert_eq!(apply_block(100.0, 500.0), 0.0);
    }

    #[test]
    fn block_zero_block_value() {
        assert_eq!(apply_block(500.0, 0.0), 500.0);
    }
}
