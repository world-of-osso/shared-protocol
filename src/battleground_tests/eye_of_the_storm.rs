use super::*;
use crate::battleground::{
    BgMatch, BgOutcome, LevelBracket, eye_of_the_storm, objective_object_entries, pvp_stat_columns,
    template_by_id,
};

// TDB1210 gameobject guids 166358/166359/166360/166361 and areatrigger spawn 38.
const BLOOD_ELF: [f32; 3] = [2050.49, 1372.24, 1194.56];
const FEL_REAVER: [f32; 3] = [2024.6, 1742.82, 1195.16];
const MAGE_TOWER: [f32; 3] = [2282.12, 1760.01, 1189.71];
const DRAENEI_RUINS: [f32; 3] = [2301.01, 1386.93, 1197.18];
const FEL_REAVER_TRIGGER: [f32; 3] = [2043.95, 1729.78, 1190.04];
const BLOOD_ELF_TRIGGER: [f32; 3] = [2048.47, 1393.59, 1194.54];

fn started() -> (BgTemplate, BgInstance, EyeBattle) {
    let template = eye_of_the_storm();
    let mut instance = BgInstance::from_match(
        &BgMatch {
            bg_id: 7,
            team_a: vec![1, 3],
            team_b: vec![2],
        },
        &template,
    );
    instance.start();
    (template, instance, EyeBattle::new(eye_rules(566).unwrap()))
}

fn has(battle: &EyeBattle, instance: &BgInstance, expected: &[(u32, i32)]) {
    let states = battle.world_states(instance);
    for value in expected {
        assert!(states.contains(value), "{value:?} missing from {states:?}");
    }
}

#[test]
fn eye_queue_identity_starts_objective_and_scoreboard_column_are_sourced() {
    let template = template_by_id(7).unwrap();
    assert_eq!(template, eye_of_the_storm());
    assert_eq!(
        (
            template.map_id,
            template.min_players_per_team,
            template.team_size,
            template.win_score
        ),
        (566, 8, 15, 1500)
    );
    assert_eq!(template.bracket, LevelBracket::new(20, 90));
    let rules = eye_rules(566).unwrap();
    assert_eq!(rules.starts, [1103, 1104]);
    assert_eq!(objective_object_entries(566), [208977]);
    let columns: Vec<_> = pvp_stat_columns(566)
        .iter()
        .map(|c| (c.stat, c.title))
        .collect();
    assert_eq!(columns, [(183, "Flag Captures")]);
}

#[test]
fn lone_player_takes_a_tower_on_the_37th_heartbeat_ticks_and_loses_it_to_one_enemy() {
    let (template, mut instance, mut battle) = started();
    let inside = [(BgTeam::A, BLOOD_ELF)];
    battle.advance(36_000, &inside, &mut instance, &template);
    has(&battle, &instance, &[(2722, 1), (2723, 0), (2752, 0)]);
    battle.advance(1_000, &inside, &mut instance, &template);
    has(
        &battle,
        &instance,
        &[(2722, 0), (2723, 1), (2724, 0), (2752, 1)],
    );
    assert_eq!(
        instance.score_a.score, 0,
        "no base was held at earlier ticks"
    );
    battle.advance(1_000, &[], &mut instance, &template);
    assert_eq!(instance.score_a.score, 1);
    has(&battle, &instance, &[(1776, 1), (1777, 0), (1780, 1500)]);
    battle.advance(1_000, &[(BgTeam::B, BLOOD_ELF)], &mut instance, &template);
    has(
        &battle,
        &instance,
        &[(2722, 1), (2723, 0), (2752, 0), (2753, 0)],
    );
}

#[test]
fn superiority_shortens_captures_equal_presence_holds_and_horde_takes_mage_tower() {
    let (template, mut instance, mut battle) = started();
    let contested = [(BgTeam::A, DRAENEI_RUINS), (BgTeam::B, DRAENEI_RUINS)];
    battle.advance(300_000, &contested, &mut instance, &template);
    has(&battle, &instance, &[(2731, 1)]);

    let three = [(BgTeam::A, FEL_REAVER); 3];
    battle.advance(27_000, &three, &mut instance, &template);
    has(&battle, &instance, &[(2726, 0)]);
    battle.advance(1_000, &three, &mut instance, &template);
    has(&battle, &instance, &[(2725, 0), (2726, 1)]);

    let mut crowd = vec![(BgTeam::B, MAGE_TOWER); 7];
    crowd.push((BgTeam::A, MAGE_TOWER));
    battle.advance(18_000, &crowd, &mut instance, &template);
    has(&battle, &instance, &[(2728, 1), (2729, 0)]);
    battle.advance(1_000, &crowd, &mut instance, &template);
    has(
        &battle,
        &instance,
        &[(2728, 0), (2729, 1), (2730, 0), (2753, 1)],
    );
}

#[test]
fn flag_scores_only_at_a_held_tower_trigger_then_respawns_after_twenty_seconds() {
    let (template, mut instance, mut battle) = started();
    battle.advance(37_000, &[(BgTeam::A, FEL_REAVER)], &mut instance, &template);
    assert!(battle.pickup(BgTeam::A, 1));
    assert!(
        !battle.pickup(BgTeam::B, 2),
        "a carried flag cannot be taken"
    );
    has(&battle, &instance, &[(8863, 2), (9808, 2), (9809, 0)]);
    assert_eq!(battle.flag_site(), None);
    let template = &template;
    assert!(!battle.capture(1, BgTeam::A, BLOOD_ELF_TRIGGER, &mut instance, template));
    assert!(!battle.capture(3, BgTeam::A, FEL_REAVER_TRIGGER, &mut instance, template));
    let before = instance.score_a.score;
    assert!(battle.capture(1, BgTeam::A, FEL_REAVER_TRIGGER, &mut instance, template));
    assert_eq!(instance.score_a.score - before, 75, "one base held");
    has(&battle, &instance, &[(8863, 4), (9808, 0), (9809, 0)]);
    battle.advance(19_999, &[], &mut instance, template);
    assert_eq!(battle.flag_site(), None);
    battle.advance(1, &[], &mut instance, template);
    assert_eq!(battle.flag_site(), Some(FlagSite::Home));
    has(&battle, &instance, &[(8863, 1)]);
}

#[test]
fn dropped_flag_can_be_taken_by_either_team_and_returns_home_after_ten_seconds() {
    let (template, mut instance, mut battle) = started();
    let spot = [2174.0, 1600.0, 1160.0];
    assert!(battle.pickup(BgTeam::A, 1));
    battle.drop_flag(spot);
    has(&battle, &instance, &[(8863, 3), (9808, 1), (9809, 1)]);
    assert_eq!(battle.flag_site(), Some(FlagSite::Dropped(spot)));
    assert!(battle.pickup(BgTeam::B, 2));
    has(&battle, &instance, &[(8863, 2), (9808, 1), (9809, 2)]);
    battle.drop_flag(spot);
    battle.advance(9_999, &[], &mut instance, &template);
    assert_eq!(battle.flag_site(), Some(FlagSite::Dropped(spot)));
    battle.advance(1, &[], &mut instance, &template);
    assert_eq!(battle.flag_site(), Some(FlagSite::Home));
    has(&battle, &instance, &[(8863, 1), (9808, 1), (9809, 1)]);
}

#[test]
fn four_bases_tick_ten_and_a_flag_worth_500_reaches_the_1500_victory() {
    let (template, mut instance, mut battle) = started();
    let everywhere = [BLOOD_ELF, FEL_REAVER, MAGE_TOWER, DRAENEI_RUINS].map(|at| (BgTeam::A, at));
    battle.advance(37_000, &everywhere, &mut instance, &template);
    has(&battle, &instance, &[(2752, 4)]);
    battle.advance(1_000, &[], &mut instance, &template);
    assert_eq!(instance.score_a.score, 10);
    instance.score_a.score = 1000;
    assert!(battle.pickup(BgTeam::A, 1));
    assert!(battle.capture(1, BgTeam::A, FEL_REAVER_TRIGGER, &mut instance, &template));
    assert_eq!(instance.score_a.score, 1500);
    assert_eq!(instance.outcome, Some(BgOutcome::Winner(BgTeam::A)));
    has(&battle, &instance, &[(1776, 1500)]);
}
