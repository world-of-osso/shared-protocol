use super::*;

// --- Rested XP tests ---

#[test]
fn rested_new_starts_empty() {
    let rested = RestedXp::new(10000);
    assert_eq!(rested.amount, 0);
    assert_eq!(rested.max, 15000); // 1.5 * 10000
    assert!(!rested.is_rested());
}

#[test]
fn rested_accumulate_in_rest_area() {
    let mut rested = RestedXp::new(10000);
    rested.accumulate(8.0, 10000); // 8 hours = 5% of level
    assert_eq!(rested.amount, 500); // 10000 * 0.05
    assert!(rested.is_rested());
}

#[test]
fn rested_caps_at_max() {
    let mut rested = RestedXp::new(10000);
    rested.accumulate(1000.0, 10000); // way more than needed
    assert_eq!(rested.amount, 15000); // capped at 1.5 levels
}

#[test]
fn rested_apply_bonus_doubles_xp() {
    let mut rested = RestedXp::new(10000);
    rested.amount = 500;
    let (total, consumed) = rested.apply_bonus(200);
    assert_eq!(total, 400); // 200 base + 200 bonus
    assert_eq!(consumed, 200);
    assert_eq!(rested.amount, 300); // 500 - 200
}

#[test]
fn rested_bonus_limited_by_pool() {
    let mut rested = RestedXp::new(10000);
    rested.amount = 100;
    let (total, consumed) = rested.apply_bonus(500);
    assert_eq!(total, 600); // 500 + 100 (only 100 available)
    assert_eq!(consumed, 100);
    assert_eq!(rested.amount, 0);
}

#[test]
fn rested_no_bonus_when_empty() {
    let mut rested = RestedXp::new(10000);
    let (total, consumed) = rested.apply_bonus(500);
    assert_eq!(total, 500);
    assert_eq!(consumed, 0);
}

#[test]
fn rested_levels_fraction() {
    let mut rested = RestedXp::new(10000);
    rested.amount = 5000;
    assert!((rested.rested_levels(10000) - 0.5).abs() < 0.001);
}

#[test]
fn rested_update_max_on_level_up() {
    let mut rested = RestedXp::new(10000);
    rested.amount = 15000; // at cap
    rested.update_max(20000); // level up
    assert_eq!(rested.max, 30000);
    assert_eq!(rested.amount, 15000); // still within new cap
}
