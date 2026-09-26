use super::*;

/// Hogger (448): ContentTuning 73, levels 1-30, no delta.
const HOGGER: LevelScaling = LevelScaling {
    content_tuning_id: 73,
    min_level: 1,
    max_level: 30,
    delta: 0,
};

#[test]
fn hogger_matches_a_level_10_player_and_caps_at_30_for_a_level_40_player() {
    assert_eq!(HOGGER.level_for_target(10), 10);
    assert_eq!(HOGGER.level_for_target(40), 30);
}

#[test]
fn creature_below_its_range_floor_stays_at_the_floor() {
    let guard = LevelScaling {
        content_tuning_id: 883,
        min_level: 90,
        max_level: 90,
        delta: 0,
    };
    assert_eq!(guard.level_for_target(5), 90);
}

#[test]
fn delta_shifts_the_scaled_and_native_levels() {
    let elite = LevelScaling { delta: 3, ..HOGGER };
    assert_eq!(elite.level_for_target(10), 13);
    assert_eq!(elite.level_for_target(40), 33);
    assert_eq!(elite.native_level(), 33);
    let weak = LevelScaling {
        delta: -4,
        ..HOGGER
    };
    assert_eq!(weak.level_for_target(2), 1, "never below level 1");
}

#[test]
fn native_level_is_the_top_of_the_range() {
    assert_eq!(HOGGER.native_level(), 30);
}

#[test]
fn untuned_units_show_their_own_level_to_every_viewer() {
    assert_eq!(level_for_viewer(UnitLevel(12), None, 40), 12);
    assert_eq!(level_for_viewer(UnitLevel(30), Some(&HOGGER), 7), 7);
}

#[test]
fn gray_level_follows_trinitycore() {
    assert_eq!(gray_level(6), 0);
    assert_eq!(gray_level(10), 4);
    assert_eq!(gray_level(20), 12);
    assert_eq!(gray_level(40), 30);
}

#[test]
fn level_colours_follow_the_level_difference() {
    use LevelDifficulty::*;
    let at_level_10 = |unit| LevelDifficulty::for_levels(10, unit);
    assert_eq!(at_level_10(15), Impossible);
    assert_eq!(at_level_10(13), VeryDifficult);
    assert_eq!(at_level_10(12), Difficult);
    assert_eq!(at_level_10(6), Difficult);
    assert_eq!(at_level_10(5), Standard);
    assert_eq!(at_level_10(4), Trivial);
}

#[test]
fn scaled_hogger_is_yellow_to_both_a_level_10_and_a_level_30_player() {
    for player in [10, 30] {
        let level = HOGGER.level_for_target(player);
        assert_eq!(
            LevelDifficulty::for_levels(player, level),
            LevelDifficulty::Difficult
        );
    }
    let level = HOGGER.level_for_target(45);
    assert_eq!(
        LevelDifficulty::for_levels(45, level),
        LevelDifficulty::Trivial
    );
}
