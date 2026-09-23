use super::*;

#[test]
fn warrior_level_1_base_hp() {
    assert_eq!(base_hp(1, 1), Some(20));
}

#[test]
fn warrior_level_80_base_hp() {
    assert_eq!(base_hp(1, 80), Some(8121));
}

#[test]
fn paladin_level_70_base_hp() {
    assert_eq!(base_hp(2, 70), Some(3377));
}

#[test]
fn dk_below_55_returns_none() {
    assert_eq!(base_hp(6, 54), None);
}

#[test]
fn dk_at_55_returns_data() {
    assert_eq!(base_hp(6, 55), Some(1359));
}

#[test]
fn invalid_class_returns_none() {
    assert_eq!(base_hp(10, 1), None);
    assert_eq!(base_hp(0, 1), None);
}

#[test]
fn druid_level_80_base_hp() {
    assert_eq!(base_hp(11, 80), Some(7417));
}

#[test]
fn hp_from_stamina_low_level_below_threshold() {
    // 15 stamina at level 30: 15 * 1 = 15
    assert_eq!(hp_from_stamina(15.0, 30), 15.0);
}

#[test]
fn hp_from_stamina_low_level_above_threshold() {
    // 50 stamina at level 30: 20*1 + 30*10 = 320
    assert_eq!(hp_from_stamina(50.0, 30), 320.0);
}

#[test]
fn hp_from_stamina_at_threshold_exact() {
    assert_eq!(hp_from_stamina(20.0, 30), 20.0);
}

#[test]
fn hp_from_stamina_retail_level() {
    // 150 stamina at level 70: 150 * 20 = 3000
    assert_eq!(hp_from_stamina(150.0, 70), 3000.0);
}

#[test]
fn hp_from_stamina_retail_level_80() {
    // 200 stamina at level 80: 200 * 20 = 4000
    assert_eq!(hp_from_stamina(200.0, 80), 4000.0);
}

#[test]
fn max_health_warrior_level_1() {
    // base 20 + 22 stam (WotLK: 20*1 + 2*10 = 40) = 60
    assert_eq!(max_health(1, 1, 22.0), Some(60.0));
}

#[test]
fn max_health_warrior_level_80() {
    // base 8121 + 159 stam * 20 = 8121 + 3180 = 11301
    assert_eq!(max_health(1, 80, 159.0), Some(11301.0));
}

#[test]
fn max_health_invalid_class_returns_none() {
    assert_eq!(max_health(0, 1, 100.0), None);
}

#[test]
fn hp_from_stamina_zero() {
    assert_eq!(hp_from_stamina(0.0, 1), 0.0);
    assert_eq!(hp_from_stamina(0.0, 80), 0.0);
}

// --- Mana tests ---

#[test]
fn paladin_level_1_base_mana() {
    assert_eq!(base_mana(2, 1), Some(60));
}

#[test]
fn paladin_level_80_base_mana() {
    assert_eq!(base_mana(2, 80), Some(4394));
}

#[test]
fn mage_level_70_base_mana() {
    assert_eq!(base_mana(8, 70), Some(2241));
}

#[test]
fn warrior_has_no_mana() {
    assert_eq!(base_mana(1, 1), None);
    assert_eq!(base_mana(1, 80), None);
}

#[test]
fn rogue_has_no_mana() {
    assert_eq!(base_mana(4, 1), None);
    assert_eq!(base_mana(4, 80), None);
}

#[test]
fn mana_from_intellect_low_level_below_threshold() {
    // 15 int at level 30: 15 * 1 = 15
    assert_eq!(mana_from_intellect(15.0, 30), 15.0);
}

#[test]
fn mana_from_intellect_low_level_above_threshold() {
    // 50 int at level 30: 20*1 + 30*15 = 470
    assert_eq!(mana_from_intellect(50.0, 30), 470.0);
}

#[test]
fn mana_from_intellect_at_threshold_exact() {
    assert_eq!(mana_from_intellect(20.0, 30), 20.0);
}

#[test]
fn mana_from_intellect_retail_level() {
    // 100 int at level 70: 100 * 20 = 2000
    assert_eq!(mana_from_intellect(100.0, 70), 2000.0);
}

#[test]
fn max_mana_paladin_level_80() {
    // base 4394 + 98 int * 20 = 4394 + 1960 = 6354
    assert_eq!(max_mana(2, 80, 98.0), Some(6354.0));
}

#[test]
fn max_mana_warrior_returns_none() {
    assert_eq!(max_mana(1, 80, 100.0), None);
}

#[test]
fn mana_from_intellect_zero() {
    assert_eq!(mana_from_intellect(0.0, 1), 0.0);
    assert_eq!(mana_from_intellect(0.0, 80), 0.0);
}

// --- Cross-checks against AzerothCore SQL (player_class_stats) ---
// Source: ~/Repos/azerothcore/data/sql/base/db_world/player_class_stats.sql
// Formula ref: AzerothCore src/server/game/Entities/Unit/StatSystem.cpp
//   GetHealthBonusFromStamina: first 20 stam = 1 HP, above 20 = 10 HP each
//   GetManaBonusFromIntellect: first 20 int = 1 mana, above 20 = 15 mana each

#[test]
fn crosscheck_warrior_80_hp() {
    // AzerothCore player_class_stats: class=1, level=80, basehp=8121
    // Warrior has no base mana.
    assert_eq!(base_hp(1, 80), Some(8121));
    assert_eq!(base_mana(1, 80), None);

    // With 100 stamina at L80 (retail model): 100 * 20 = 2000
    // Total: 8121 + 2000 = 10121
    assert_eq!(max_health(1, 80, 100.0), Some(10121.0));
}

#[test]
fn crosscheck_mage_70_hp_mana() {
    // AzerothCore player_class_stats: class=8, level=70, basehp=3393, basemana=2241
    assert_eq!(base_hp(8, 70), Some(3393));
    assert_eq!(base_mana(8, 70), Some(2241));

    // With 50 stamina at L70 (retail model): 50 * 20 = 1000
    // Total HP: 3393 + 1000 = 4393
    assert_eq!(max_health(8, 70, 50.0), Some(4393.0));

    // With 150 intellect at L70 (retail model): 150 * 20 = 3000
    // Total mana: 2241 + 3000 = 5241
    assert_eq!(max_mana(8, 70, 150.0), Some(5241.0));
}

#[test]
fn crosscheck_paladin_1_hp_mana() {
    // AzerothCore player_class_stats: class=2, level=1, basehp=28, basemana=60
    assert_eq!(base_hp(2, 1), Some(28));
    assert_eq!(base_mana(2, 1), Some(60));

    // With 25 stamina at L1 (WotLK model): 20*1 + 5*10 = 70
    // Total HP: 28 + 70 = 98
    assert_eq!(max_health(2, 1, 25.0), Some(98.0));

    // With 25 intellect at L1 (WotLK model): 20*1 + 5*15 = 95
    // Total mana: 60 + 95 = 155
    assert_eq!(max_mana(2, 1, 25.0), Some(155.0));
}

// --- Equipment stat aggregation tests ---

#[test]
fn sum_equipment_stats_empty() {
    let (primary, secondary) = sum_equipment_stats(&[]);
    assert_eq!(primary, UnitStats::default());
    assert_eq!(secondary, CombatRatings::default());
}

#[test]
fn sum_equipment_stats_single_item() {
    let chest = ItemStatBlock {
        primary: UnitStats {
            stamina: 50.0,
            strength: 30.0,
            ..Default::default()
        },
        secondary: CombatRatings {
            crit: 20.0,
            haste: 15.0,
            ..Default::default()
        },
    };
    let (primary, secondary) = sum_equipment_stats(&[chest]);
    assert_eq!(primary.stamina, 50.0);
    assert_eq!(primary.strength, 30.0);
    assert_eq!(secondary.crit, 20.0);
    assert_eq!(secondary.haste, 15.0);
}

#[test]
fn sum_equipment_stats_multiple_items() {
    let helm = ItemStatBlock {
        primary: UnitStats {
            stamina: 40.0,
            intellect: 35.0,
            ..Default::default()
        },
        secondary: CombatRatings {
            crit: 18.0,
            mastery: 12.0,
            ..Default::default()
        },
    };
    let chest = ItemStatBlock {
        primary: UnitStats {
            stamina: 55.0,
            intellect: 45.0,
            spirit: 10.0,
            ..Default::default()
        },
        secondary: CombatRatings {
            haste: 25.0,
            versatility: 20.0,
            armor: 500.0,
            ..Default::default()
        },
    };
    let legs = ItemStatBlock {
        primary: UnitStats {
            stamina: 48.0,
            intellect: 40.0,
            ..Default::default()
        },
        secondary: CombatRatings {
            crit: 22.0,
            mastery: 15.0,
            armor: 400.0,
            ..Default::default()
        },
    };

    let (primary, secondary) = sum_equipment_stats(&[helm, chest, legs]);

    assert_eq!(primary.stamina, 143.0);
    assert_eq!(primary.intellect, 120.0);
    assert_eq!(primary.spirit, 10.0);
    assert_eq!(primary.strength, 0.0);
    assert_eq!(secondary.crit, 40.0);
    assert_eq!(secondary.mastery, 27.0);
    assert_eq!(secondary.haste, 25.0);
    assert_eq!(secondary.versatility, 20.0);
    assert_eq!(secondary.armor, 900.0);
}

// --- AP contribution tests ---

#[test]
fn ap_bonus_with_2h_weapon() {
    // 1200 AP, 3.3 speed: (1200/6) * 3.3 = 200 * 3.3 = 660
    assert_eq!(ap_bonus_damage(1200.0, 3.3), 660.0);
}

#[test]
fn ap_bonus_with_1h_weapon() {
    // 1200 AP, 2.4 speed: (1200/6) * 2.4 = 200 * 2.4 = 480
    assert!((ap_bonus_damage(1200.0, 2.4) - 480.0).abs() < 0.01);
}

#[test]
fn ap_bonus_with_dagger() {
    // 1200 AP, 1.7 speed: (1200/6) * 1.7 = 200 * 1.7 = 340
    assert!((ap_bonus_damage(1200.0, 1.7) - 340.0).abs() < 0.01);
}

#[test]
fn ap_bonus_zero_ap() {
    assert_eq!(ap_bonus_damage(0.0, 2.6), 0.0);
}

#[test]
fn ap_bonus_scales_with_speed() {
    let slow = ap_bonus_damage(1000.0, 3.6);
    let fast = ap_bonus_damage(1000.0, 1.5);
    assert!(slow > fast, "slower weapons get more AP bonus per swing");
}

// --- Auto-attack damage tests ---

#[test]
fn auto_attack_min_roll() {
    // min weapon roll 100, 1200 AP, 3.3 speed → 100 + 660 = 760
    assert_eq!(auto_attack_damage(100.0, 1200.0, 3.3), 760.0);
}

#[test]
fn auto_attack_max_roll() {
    // max weapon roll 200, 1200 AP, 3.3 speed → 200 + 660 = 860
    assert_eq!(auto_attack_damage(200.0, 1200.0, 3.3), 860.0);
}

#[test]
fn auto_attack_zero_ap() {
    // No AP, just weapon damage
    assert_eq!(auto_attack_damage(150.0, 0.0, 2.6), 150.0);
}

#[test]
fn auto_attack_zero_weapon() {
    // Unarmed: 0 weapon roll, AP still contributes
    let bonus = ap_bonus_damage(1200.0, 2.0);
    assert_eq!(auto_attack_damage(0.0, 1200.0, 2.0), bonus);
}

// Remaining combat/integration tests moved to combat_tests.rs
