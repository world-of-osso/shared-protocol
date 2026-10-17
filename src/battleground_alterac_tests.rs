use super::*;

fn fixture() -> (AlteracBattle, BgInstance) {
    let template = alterac_valley();
    let mut instance = BgInstance::from_match(
        &BgMatch {
            bg_id: template.id,
            bracket: template.bracket,
            team_a: vec![1],
            team_b: vec![2],
        },
        &template,
    );
    assert!(instance.start());
    (AlteracBattle::default(), instance)
}

#[test]
fn alterac_neutral_snowfall_can_be_counter_assaulted_before_first_capture() {
    let (mut battle, mut instance) = fixture();
    assert_eq!(battle.interact(180418, BgTeam::A, &instance), Ok(63));
    assert_eq!(battle.banner_entry(3), Some(180419));
    assert_eq!(battle.interact(180419, BgTeam::B, &instance), Ok(63));
    battle.advance(239_999, &mut instance);
    assert_eq!(battle.banner_entry(3), Some(180420));
    battle.advance(1, &mut instance);
    assert_eq!(battle.banner_entry(3), Some(178364));
    assert!(battle.world_states(&instance).contains(&(1342, 1)));
    assert_eq!(instance.score_b.reinforcements, 700);
}

#[test]
fn alterac_tower_defense_cancels_burn_and_destroyed_tower_is_permanent() {
    let (mut battle, mut instance) = fixture();
    assert_eq!(battle.interact(178946, BgTeam::A, &instance), Ok(61));
    battle.advance(120_000, &mut instance);
    assert_eq!(battle.interact(178940, BgTeam::B, &instance), Ok(64));
    battle.advance(240_000, &mut instance);
    assert_eq!(instance.score_b.reinforcements, 700);
    assert_eq!(battle.banner_entry(11), Some(178946));
    assert_eq!(battle.interact(178946, BgTeam::A, &instance), Ok(61));
    assert!(battle.interact(178946, BgTeam::B, &instance).is_err());
    battle.advance(239_999, &mut instance);
    assert_eq!(instance.score_b.reinforcements, 700);
    battle.advance(1, &mut instance);
    assert_eq!(instance.score_b.reinforcements, 625);
    assert_eq!(battle.banner_entry(11), None);
    assert!(battle.world_states(&instance).contains(&(1368, 1)));
    assert!(battle.interact(178940, BgTeam::B, &instance).is_err());
    battle.advance(480_000, &mut instance);
    assert_eq!(instance.score_b.reinforcements, 625);
}

#[test]
fn alterac_graveyard_capture_and_defense_do_not_deduct_reinforcements() {
    let (mut battle, mut instance) = fixture();
    assert_eq!(battle.interact(179483, BgTeam::A, &instance), Ok(63));
    battle.advance(240_000, &mut instance);
    assert_eq!(battle.banner_entry(4), Some(179481));
    assert_eq!(battle.interact(179481, BgTeam::B, &instance), Ok(63));
    assert_eq!(battle.interact(179484, BgTeam::A, &instance), Ok(65));
    battle.advance(240_000, &mut instance);
    assert_eq!(battle.banner_entry(4), Some(179481));
    assert_eq!(instance.score_b.reinforcements, 700);
}

#[test]
fn alterac_captains_deduct_once_and_general_death_finishes_without_score_injection() {
    let (mut battle, mut instance) = fixture();
    assert!(battle.npc_killed(11949, Some(BgTeam::B), &mut instance));
    assert_eq!(instance.score_a.reinforcements, 600);
    assert!(!battle.npc_killed(11949, Some(BgTeam::B), &mut instance));
    assert_eq!(instance.score_a.reinforcements, 600);
    assert!(battle.world_states(&instance).contains(&(1351, 0)));
    assert!(!battle.npc_killed(99999, Some(BgTeam::A), &mut instance));
    assert!(battle.npc_killed(11946, Some(BgTeam::A), &mut instance));
    assert_eq!(instance.phase, BgPhase::Ended);
    assert_eq!(instance.outcome, Some(BgOutcome::Winner(BgTeam::A)));
    assert_eq!(instance.score_b.reinforcements, 700);
    assert!(battle.world_states(&instance).contains(&(601, 0)));
    assert!(!battle.npc_killed(11948, Some(BgTeam::B), &mut instance));
    assert_eq!(instance.outcome, Some(BgOutcome::Winner(BgTeam::A)));
}

#[test]
fn alterac_source_mine_leaders_change_owner_and_real_45second_ticks_add_resources() {
    let (mut battle, mut instance) = fixture();
    assert!(!battle.npc_killed(11657, None, &mut instance));
    assert!(battle.npc_killed(11657, Some(BgTeam::A), &mut instance));
    assert!(!battle.npc_killed(11657, Some(BgTeam::B), &mut instance));
    battle.advance(44_999, &mut instance);
    assert_eq!(instance.score_a.reinforcements, 700);
    battle.advance(1, &mut instance);
    assert_eq!(instance.score_a.reinforcements, 701);
    assert!(battle.npc_killed(13079, Some(BgTeam::B), &mut instance));
    assert!(battle.npc_killed(11677, Some(BgTeam::B), &mut instance));
    battle.advance(45_000, &mut instance);
    assert_eq!(instance.score_a.reinforcements, 701);
    assert_eq!(instance.score_b.reinforcements, 702);
    assert!(battle.world_states(&instance).contains(&(801, 2)));
    assert!(battle.world_states(&instance).contains(&(804, 2)));
}
