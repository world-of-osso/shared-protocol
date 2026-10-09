//! Retail item stats at the item's level: primary stats, ratings, armor and
//! weapon damage, from world.db DB2 tables and the item-level game tables.
//!
//! Formulas follow SimC `sc_item_data.cpp` (`item_database::scaled_stat`,
//! `random_suffix_type`, `item_combat_rating_type`, `armor_value`,
//! `weapon_dmg_min/max`) and `sc_const_data.cpp` (`dbc_t::weapon_dps`), SimC
//! commit 3036108; TrinityCore `Item::GetItemStatValue`,
//! `ItemTemplate::GetArmor` and `ItemTemplate::GetDamage` use the same tables.
//! Old items first take the Midnight item-level squish (SimC
//! `item_t::download_item_data`, curve 92181 from ItemSquishEra).
use std::collections::HashMap;

use crate::components::{ItemStatBlock, UnitStats, WeaponType};
use bevy::prelude::*;

const ITEM_CLASS_WEAPON: u8 = 2;
const ITEM_CLASS_ARMOR: u8 = 4;
const ARMOR_SUBCLASS_PLATE: u8 = 4;
const ARMOR_SUBCLASS_SHIELD: u8 = 6;
const WEAPON_SUBCLASS_BOW: u8 = 2;
const WEAPON_SUBCLASS_GUN: u8 = 3;
const WEAPON_SUBCLASS_DAGGER: u8 = 15;
const WEAPON_SUBCLASS_THROWN: u8 = 16;
const WEAPON_SUBCLASS_CROSSBOW: u8 = 18;
const WEAPON_SUBCLASS_WAND: u8 = 19;
/// `ITEM_FLAG2_CASTER_WEAPON` (ItemSparse `Flags_1`): DPS from the caster tables.
pub const FLAG2_CASTER_WEAPON: i64 = 0x200;

/// One `StatModifier_bonusStat_N` / `StatPercentEditor_N` /
/// `StatPercentageOfSocket_N` triple of ItemSparse.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ItemStatAllocation {
    /// `ITEM_MOD_*`; -1 or 0 when unused.
    pub stat: i8,
    /// Share of the item-level budget in 1/10000.
    pub allocation: i32,
    pub socket_multiplier: f32,
}

/// ItemSparse/Item attributes the stat formulas read.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ItemScaling {
    pub class_id: u8,
    pub subclass_id: u8,
    pub quality: u8,
    pub item_level: u16,
    pub squish_era: u8,
    pub delay_ms: u16,
    pub damage_variance: f32,
    pub flags2: i64,
    pub stats: [ItemStatAllocation; 10],
}

/// Weapon swing data. Unarmed is TrinityCore `BASE_MINDAMAGE`/`BASE_MAXDAMAGE`
/// (1-2) at `BASE_ATTACK_TIME` (2.0 s).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeaponDamage {
    pub min: f32,
    pub max: f32,
    /// Swing time in seconds.
    pub speed: f32,
    pub kind: WeaponType,
}

impl WeaponDamage {
    pub const UNARMED: Self = Self {
        min: 1.0,
        max: 2.0,
        speed: 2.0,
        kind: WeaponType::OneHand,
    };
}

impl Default for WeaponDamage {
    fn default() -> Self {
        Self::UNARMED
    }
}

/// Stats one equipped item gives.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ItemStats {
    pub item_level: u16,
    pub block: ItemStatBlock,
    pub weapon: Option<WeaponDamage>,
    /// `ITEM_MOD_HEALTH_REGEN` 46 (ItemTemplate.h:71): health per 5 s, in and out of
    /// combat (`ApplyHealthRegenBonus`, Player.cpp:8060-8061).
    pub health_regen: f32,
}

/// RandPropPoints row: `[Epic, Superior, Good][suffix type]` float budgets.
type RandProp = [[f64; 5]; 3];
/// Per-quality row (`Quality_0..6` / `Qualitymod_0..6`).
type QualityRow = [f64; 7];

/// DB2 item-level tables, keyed by item level.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct ItemScalingTables {
    pub rand_prop_points: HashMap<u16, RandProp>,
    /// RandPropPoints DamageReplaceStatF, DamageSecondaryF, SuperiorF_0.
    /// Item spell scaling classes -8, -9, and the other nonzero classes.
    pub spell_budgets: HashMap<u16, [f32; 3]>,
    pub armor_quality: HashMap<u16, QualityRow>,
    pub armor_shield: HashMap<u16, QualityRow>,
    /// Cloth, Leather, Mail, Plate.
    pub armor_total: HashMap<u16, [f64; 4]>,
    /// ArmorLocation by inventory type: cloth, leather, chain, plate modifier.
    pub armor_location: HashMap<u8, [f64; 4]>,
    /// ItemDamageOneHand, OneHandCaster, TwoHand, TwoHandCaster.
    pub damage: [HashMap<u16, QualityRow>; 4],
    /// Newest squish era with a curve and its points: items of older eras take it.
    pub squish: Option<(u8, Vec<(f64, f64)>)>,
    pub socket_cost: Vec<f32>,
    pub stamina_multiplier: Vec<[f32; 4]>,
    pub rating_multiplier: Vec<[f32; 4]>,
}

impl ItemScalingTables {
    /// Item level after the squish of every later era (SimC `util::round`).
    pub fn item_level(&self, item: &ItemScaling) -> u16 {
        match &self.squish {
            Some((era, points)) if item.squish_era < *era && item.item_level > 0 => {
                (curve_value(points, f64::from(item.item_level)) + 0.5).floor() as u16
            }
            _ => item.item_level,
        }
    }
}

/// Piecewise-linear curve, clamped to its ends (SimC `curve_point_value`).
fn curve_value(points: &[(f64, f64)], x: f64) -> f64 {
    let Some(upper) = points.iter().position(|&(px, _)| px >= x) else {
        return points.last().map_or(x, |&(_, y)| y);
    };
    if upper == 0 {
        return points[0].1;
    }
    let (x0, y0) = points[upper - 1];
    let (x1, y1) = points[upper];
    y0 + (x - x0) / (x1 - x0) * (y1 - y0)
}

/// RandPropPoints column: 0 two-handers/head/chest/legs, 1 shoulders/waist/
/// feet/hands/trinket, 2 neck/finger/cloak/wrists, 3 one-handers/off-hands.
fn budget_slot(item: &ItemScaling, inventory_type: u8) -> Option<usize> {
    match item.class_id {
        ITEM_CLASS_WEAPON => Some(match item.subclass_id {
            1 | 2 | 3 | 5 | 6 | 8 | 10 | 16 | 18 => 0,
            _ => 3,
        }),
        ITEM_CLASS_ARMOR => match inventory_type {
            1 | 5 | 7 | 20 => Some(0),
            3 | 6 | 8 | 10 | 12 => Some(1),
            2 | 9 | 11 | 16 => Some(2),
            14 | 22 | 23 => Some(3),
            _ => None,
        },
        _ => None,
    }
}

/// RandPropPoints quality group: 0 epic/legendary, 1 rare/heirloom, 2 the rest.
fn budget_quality(quality: u8) -> usize {
    match quality {
        4 | 5 => 0,
        3 | 7 => 1,
        _ => 2,
    }
}

/// Column of the ilvl multiplier game tables: armor, weapon, trinket, jewelry.
fn multiplier_column(inventory_type: u8) -> Option<usize> {
    match inventory_type {
        1 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 14 | 16 | 20 | 23 => Some(0),
        13 | 15 | 17 | 21 | 22 | 25 | 26 => Some(1),
        12 => Some(2),
        2 | 11 => Some(3),
        _ => None,
    }
}

fn ilvl_row<T: Copy>(table: &[T], item_level: u16) -> Option<T> {
    table.get(usize::from(item_level).checked_sub(1)?).copied()
}

const ITEM_MOD_HEALTH_REGEN: i8 = 46;

/// `ITEM_MOD_*` stats the server models.
fn add_stat(block: &mut ItemStatBlock, stat: i8, value: f32) {
    let primary = &mut block.primary;
    let ratings = &mut block.secondary;
    match stat {
        3 => primary.agility += value,
        4 => primary.strength += value,
        5 => primary.intellect += value,
        7 => primary.stamina += value,
        // Hybrid primaries: each listed stat; the spec's main stat filter keeps one.
        71 => add_primaries(primary, value, true, true, true),
        72 => add_primaries(primary, value, true, true, false),
        73 => add_primaries(primary, value, false, true, true),
        74 => add_primaries(primary, value, true, false, true),
        32 => ratings.crit += value,
        36 => ratings.haste += value,
        40 => ratings.versatility += value,
        49 => ratings.mastery += value,
        // ITEM_MOD_CR_SPEED / CR_LIFESTEAL / CR_AVOIDANCE (ItemTemplate.h:86-88).
        61 => ratings.speed += value,
        62 => ratings.leech += value,
        63 => ratings.avoidance += value,
        _ => {}
    }
}

fn add_primaries(stats: &mut UnitStats, value: f32, str: bool, agi: bool, int: bool) {
    stats.strength += if str { value } else { 0.0 };
    stats.agility += if agi { value } else { 0.0 };
    stats.intellect += if int { value } else { 0.0 };
}

/// SimC `util::is_combat_rating` for the modelled stats.
fn is_rating(stat: i8) -> bool {
    matches!(stat, 32 | 36 | 40 | 49 | 61 | 62 | 63)
}

/// SimC `scaled_stat`: `round(alloc × budget × 0.0001 − socket penalty)`
/// with the rating or stamina multiplier of the item level and slot.
fn scaled_stat(
    tables: &ItemScalingTables,
    alloc: &ItemStatAllocation,
    budget: f64,
    item_level: u16,
    column: Option<usize>,
) -> f32 {
    let socket_cost = ilvl_row(&tables.socket_cost, item_level).map_or(0.0, f64::from);
    // Socket penalty rounds half to even (SimC `std::nearbyint`).
    let penalty = (f64::from(alloc.socket_multiplier) * socket_cost).round_ties_even();
    let mut value = f64::from(alloc.allocation) * budget * 0.0001 - penalty;
    let multiplier = |table: &[[f32; 4]]| {
        column
            .and_then(|column| Some(ilvl_row(table, item_level)?[column]))
            .map_or(1.0, f64::from)
    };
    if is_rating(alloc.stat) {
        value *= multiplier(&tables.rating_multiplier);
    } else if alloc.stat == 7 {
        value *= multiplier(&tables.stamina_multiplier);
    }
    (value + 0.5).floor() as f32
}

fn stat_budget(tables: &ItemScalingTables, item: &ItemScaling, slot: u8, ilvl: u16) -> f64 {
    let Some(column) = budget_slot(item, slot).filter(|_| item.quality > 0) else {
        return 0.0;
    };
    tables
        .rand_prop_points
        .get(&ilvl)
        .map_or(0.0, |row| row[budget_quality(item.quality)][column])
}

/// SimC `armor_value`: shields from ItemArmorShield, cloth-plate from
/// ItemArmorTotal × ItemArmorQuality × ArmorLocation.
pub fn armor(tables: &ItemScalingTables, item: &ItemScaling, inventory_type: u8, ilvl: u16) -> f64 {
    let quality = usize::from(item.quality);
    if item.class_id != ITEM_CLASS_ARMOR || item.quality > 5 {
        return 0.0;
    }
    if item.subclass_id == ARMOR_SUBCLASS_SHIELD {
        return tables
            .armor_shield
            .get(&ilvl)
            .map_or(0.0, |row| (row[quality] + 0.5).floor());
    }
    if !(1..=ARMOR_SUBCLASS_PLATE).contains(&item.subclass_id)
        || !matches!(inventory_type, 1 | 3 | 5 | 6 | 7 | 8 | 9 | 10 | 16 | 20)
    {
        return 0.0;
    }
    let material = usize::from(item.subclass_id - 1);
    let location = if inventory_type == 20 {
        5
    } else {
        inventory_type
    };
    let (Some(total), Some(quality_mod), Some(location_mod)) = (
        tables.armor_total.get(&ilvl),
        tables.armor_quality.get(&ilvl),
        tables.armor_location.get(&location),
    ) else {
        return 0.0;
    };
    (total[material] * quality_mod[quality] * location_mod[material] + 0.5).floor()
}

/// `dbc_t::weapon_dps` table: 0 one-hand, 1 one-hand caster, 2 two-hand, 3 two-hand caster.
fn damage_table(item: &ItemScaling, inventory_type: u8) -> Option<usize> {
    let caster = usize::from(item.flags2 & FLAG2_CASTER_WEAPON != 0);
    match inventory_type {
        13 | 21 | 22 => Some(caster),
        17 => Some(2 + caster),
        15 | 25 | 26 => match item.subclass_id {
            WEAPON_SUBCLASS_BOW | WEAPON_SUBCLASS_GUN | WEAPON_SUBCLASS_CROSSBOW => Some(2),
            WEAPON_SUBCLASS_THROWN => Some(0),
            WEAPON_SUBCLASS_WAND => Some(1),
            _ => None,
        },
        _ => None,
    }
}

fn weapon_kind(item: &ItemScaling, inventory_type: u8) -> WeaponType {
    match inventory_type {
        _ if item.subclass_id == WEAPON_SUBCLASS_DAGGER => WeaponType::Dagger,
        15 | 25 | 26 => WeaponType::Ranged,
        17 => WeaponType::TwoHand,
        _ => WeaponType::OneHand,
    }
}

/// SimC `weapon_dmg_min/max`: DPS × speed × (1 ∓ variance / 2), min floored,
/// max rounded.
pub fn weapon(
    tables: &ItemScalingTables,
    item: &ItemScaling,
    inventory_type: u8,
    ilvl: u16,
) -> Option<WeaponDamage> {
    if item.class_id != ITEM_CLASS_WEAPON || item.delay_ms == 0 {
        return None;
    }
    let dps = weapon_dps(tables, item, inventory_type, ilvl)?;
    let speed = f64::from(item.delay_ms) / 1000.0;
    let variance = f64::from(item.damage_variance);
    Some(WeaponDamage {
        min: (dps * speed * (1.0 - variance / 2.0)).floor() as f32,
        max: (dps * speed * (1.0 + variance / 2.0) + 0.5).floor() as f32,
        speed: speed as f32,
        kind: weapon_kind(item, inventory_type),
    })
}

/// Stats of `template` at its (squished) item level.
pub fn item_stats(item: &ItemScaling, slot: u8, tables: &ItemScalingTables) -> ItemStats {
    let ilvl = tables.item_level(item);
    let mut block = ItemStatBlock::default();
    let mut health_regen = 0.0;
    for (stat, value) in stat_values(tables, item, slot) {
        if stat == ITEM_MOD_HEALTH_REGEN {
            health_regen += value;
        } else {
            add_stat(&mut block, stat, value);
        }
    }
    block.secondary.armor = armor(tables, item, slot, ilvl) as f32;
    ItemStats {
        item_level: ilvl,
        block,
        weapon: weapon(tables, item, slot, ilvl),
        health_regen,
    }
}

/// Allocated values in authored order, shared by tooltips and applied stats.
pub fn stat_values<'a>(
    tables: &'a ItemScalingTables,
    item: &'a ItemScaling,
    slot: u8,
) -> impl Iterator<Item = (i8, f32)> + 'a {
    let ilvl = tables.item_level(item);
    let budget = stat_budget(tables, item, slot, ilvl);
    item.stats
        .iter()
        .filter(move |alloc| budget > 0.0 && alloc.allocation > 0)
        .map(move |alloc| {
            (
                alloc.stat,
                scaled_stat(tables, alloc, budget, ilvl, multiplier_column(slot)),
            )
        })
}

/// Table DPS before swing-range rounding.
pub fn weapon_dps(
    tables: &ItemScalingTables,
    item: &ItemScaling,
    slot: u8,
    ilvl: u16,
) -> Option<f64> {
    let quality = usize::from(if item.quality > 6 { 4 } else { item.quality });
    Some(
        tables.damage[damage_table(item, slot)?]
            .get(&ilvl)
            .map_or(0.0, |row| row[quality]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squish_and_socket_cost_precede_stamina_and_rating_multipliers() {
        let tables = ItemScalingTables {
            squish: Some((2, vec![(10.0, 10.0), (342.0, 70.0)])),
            rand_prop_points: HashMap::from([(11, [[10.0; 5]; 3])]),
            socket_cost: vec![2.5; 11],
            stamina_multiplier: vec![[2.0; 4]; 11],
            rating_multiplier: vec![[1.5; 4]; 11],
            ..Default::default()
        };
        let mut item = ItemScaling {
            class_id: ITEM_CLASS_ARMOR,
            quality: 2,
            item_level: 16,
            ..Default::default()
        };
        item.stats[0] = ItemStatAllocation {
            stat: 7,
            allocation: 10000,
            socket_multiplier: 1.0,
        };
        item.stats[1] = ItemStatAllocation {
            stat: 32,
            allocation: 10000,
            socket_multiplier: 1.0,
        };
        let result = item_stats(&item, 5, &tables);
        assert_eq!(result.item_level, 11);
        assert_eq!(result.block.primary.stamina, 16.0);
        assert_eq!(result.block.secondary.crit, 12.0);
        assert_eq!(
            stat_values(&tables, &item, 5).collect::<Vec<_>>(),
            [(7, 16.0), (32, 12.0)]
        );
        item.squish_era = 2;
        assert_eq!(tables.item_level(&item), 16);
    }
}
