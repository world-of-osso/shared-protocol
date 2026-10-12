use super::*;
use crate::battleground::{BgMatch, BgOutcome, temple_of_kotmogu};

const CENTER: [f32; 3] = [1783.41, 1333.56, 6.0];
const MEDIUM: [f32; 3] = [1823.41, 1333.56, 11.0];
const OUTSIDE: [f32; 3] = [1850.1666, 1250.118, 13.2056];

fn started() -> (BgInstance, BgTemplate, KotmoguBattle) {
    let template = temple_of_kotmogu();
    let mut instance = BgInstance::from_match(
        &BgMatch {
            bg_id: template.id,
            team_a: vec![1, 3],
            team_b: vec![2, 4],
        },
        &template,
    );
    instance.start();
    (instance, template, KotmoguBattle::new())
}

#[test]
fn kotmogu_only_one_orb_per_player_and_taken_orbs_cannot_be_stolen() {
    let mut battle = KotmoguBattle::new();
    assert!(battle.pickup(212094, 1, BgTeam::A).is_ok());
    assert!(battle.pickup(212091, 1, BgTeam::A).is_err());
    assert!(battle.pickup(212094, 2, BgTeam::B).is_err());
    assert!(battle.pickup(999999, 2, BgTeam::B).is_err());
    let (instance, _, _) = started();
    let states = battle.world_states(&instance);
    assert!(states.contains(&(6717, 0)));
    assert!(states.contains(&(6964, 1)));
    assert!(states.contains(&(6963, 0)));
    assert!(battle.drop_carrier(1));
    assert!(!battle.drop_carrier(1));
    assert!(battle.pickup(212094, 2, BgTeam::B).is_ok());
    assert!(battle.world_states(&instance).contains(&(6963, 1)));
}

#[test]
fn kotmogu_source_area_geometry_and_five_second_scores() {
    let (mut instance, template, mut battle) = started();
    battle.pickup(212094, 1, BgTeam::A).unwrap();
    assert!(
        battle
            .advance(4999, &[(1, OUTSIDE)], &mut instance, &template)
            .is_empty()
    );
    assert_eq!(
        battle.advance(1, &[(1, OUTSIDE)], &mut instance, &template),
        [(1, 2)]
    );
    assert_eq!(
        battle.advance(5000, &[(1, MEDIUM)], &mut instance, &template),
        [(1, 4)]
    );
    assert_eq!(
        battle.advance(5000, &[(1, CENTER)], &mut instance, &template),
        [(1, 6)]
    );
    assert_eq!(instance.score_a.score, 12);
    // TC Box extents 33/47, half-height 2.5; medium zero-height cylinder is
    // G3D's distance to a zero-length segment, thus a radius-50 sphere, not a flat disk.
    assert_eq!(victory_points([1816.41, 1333.56, 8.5]), 6);
    assert_eq!(victory_points([1816.42, 1333.56, 8.5]), 4);
    assert_eq!(victory_points([1783.41, 1333.56, 56.0]), 4);
    assert_eq!(victory_points([1783.41, 1333.56, 56.1]), 2);
}

#[test]
fn kotmogu_absent_dead_or_departed_carriers_return_the_orb_without_points() {
    let (mut instance, template, mut battle) = started();
    battle.pickup(212094, 1, BgTeam::A).unwrap();
    assert!(
        battle
            .advance(5000, &[], &mut instance, &template)
            .is_empty()
    );
    assert_eq!(instance.score_a.score, 0);
    assert!(battle.holders().is_empty());
    let states = battle.world_states(&instance);
    assert!(states.contains(&(6717, 1)));
    assert!(states.contains(&(6964, 0)));
}

#[test]
fn kotmogu_real_tick_victory_is_clamped_and_simultaneous_limit_is_a_draw() {
    let (mut instance, template, mut battle) = started();
    battle.pickup(212094, 1, BgTeam::A).unwrap();
    battle.advance(1_250_000, &[(1, CENTER)], &mut instance, &template);
    assert_eq!(instance.score_a.score, 1500);
    assert_eq!(instance.outcome, Some(BgOutcome::Winner(BgTeam::A)));
    assert!(
        battle
            .advance(5000, &[(1, CENTER)], &mut instance, &template)
            .is_empty()
    );
    let (mut instance, template, mut battle) = started();
    battle.pickup(212094, 1, BgTeam::A).unwrap();
    battle.pickup(212091, 2, BgTeam::B).unwrap();
    battle.advance(
        1_250_000,
        &[(1, CENTER), (2, CENTER)],
        &mut instance,
        &template,
    );
    assert_eq!(
        (instance.score_a.score, instance.score_b.score),
        (1500, 1500)
    );
    assert_eq!(instance.outcome, Some(BgOutcome::Draw));
}
