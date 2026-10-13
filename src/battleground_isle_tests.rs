use super::*;

fn fixture() -> (IsleBattle, BgInstance) {
    let template = isle_of_conquest();
    let mut instance = BgInstance::from_match(
        &BgMatch {
            bg_id: template.id,
            team_a: vec![1],
            team_b: vec![2],
        },
        &template,
    );
    assert!(instance.start());
    (IsleBattle::default(), instance)
}

fn state(battle: &IsleBattle, instance: &BgInstance, id: u32) -> i32 {
    battle
        .world_states(instance)
        .into_iter()
        .find(|(state, _)| *state == id)
        .unwrap_or_else(|| panic!("world state {id} missing"))
        .1
}

#[test]
fn isle_initial_world_states_follow_tdb_defaults() {
    let (battle, instance) = fixture();
    // TDB1210 world_state DefaultValue rows for map 628.
    for (id, value) in [
        (4221, 1),
        (4222, 1),
        (4226, 400),
        (4227, 400),
        (17377, 400),
        (4311, 1), // Refinery uncontrolled
        (4306, 1), // Quarry uncontrolled
        (4301, 1), // Docks uncontrolled
        (4296, 1), // Hangar uncontrolled
        (4294, 1), // Workshop uncontrolled
        (4339, 1), // Alliance Keep Alliance controlled
        (4341, 0),
        (4345, 1), // Horde Keep Horde controlled
        (4346, 0),
    ] {
        assert_eq!(state(&battle, &instance, id), value, "world state {id}");
    }
}

#[test]
fn isle_quarry_assault_captures_after_one_minute_and_ticks_reinforcements() {
    let (mut battle, mut instance) = fixture();
    // Neutral Quarry banner 195338: assault credits PvpStat 245 Bases Assaulted.
    assert_eq!(battle.interact(195338, BgTeam::A, &instance), Ok(245));
    assert_eq!(state(&battle, &instance, 4310), 1);
    assert_eq!(state(&battle, &instance, 4306), 0);
    // A contested node yields nothing at the 45 s resource tick.
    battle.advance(45_000, &mut instance);
    assert_eq!(instance.score_a.reinforcements, 400);
    battle.advance(14_999, &mut instance);
    assert_eq!(state(&battle, &instance, 4309), 0);
    battle.advance(1, &mut instance);
    assert_eq!(state(&battle, &instance, 4309), 1);
    assert_eq!(state(&battle, &instance, 4310), 0);
    // Next tick at 90 s from the match start.
    battle.advance(30_000, &mut instance);
    assert_eq!(instance.score_a.reinforcements, 401);
    assert_eq!(state(&battle, &instance, 4226), 401);
    battle.advance(45_000, &mut instance);
    assert_eq!(instance.score_a.reinforcements, 402);
    assert_eq!(instance.score_b.reinforcements, 400);
}

#[test]
fn isle_defense_restores_control_immediately_and_stale_banners_are_refused() {
    let (mut battle, mut instance) = fixture();
    battle.interact(195343, BgTeam::B, &instance).unwrap();
    battle.advance(60_000, &mut instance);
    assert_eq!(state(&battle, &instance, 4313), 1);
    // The neutral banner left with its spawn group; own controlled banner is refused.
    assert!(battle.interact(195343, BgTeam::A, &instance).is_err());
    assert!(battle.interact(195341, BgTeam::B, &instance).is_err());
    assert_eq!(battle.interact(195341, BgTeam::A, &instance), Ok(245));
    assert_eq!(state(&battle, &instance, 4315), 1);
    // The assaulter cannot use its own contested banner; the last owner defends (246).
    assert!(battle.interact(195340, BgTeam::A, &instance).is_err());
    assert_eq!(battle.interact(195340, BgTeam::B, &instance), Ok(246));
    assert_eq!(state(&battle, &instance, 4313), 1);
    assert_eq!(state(&battle, &instance, 4315), 0);
    battle.advance(60_000, &mut instance);
    assert_eq!(state(&battle, &instance, 4313), 1);
    assert_eq!(state(&battle, &instance, 4314), 0);
}

#[test]
fn isle_neutral_assault_can_be_counter_assaulted_and_restarts_capture() {
    let (mut battle, mut instance) = fixture();
    battle.interact(195157, BgTeam::A, &instance).unwrap();
    battle.advance(50_000, &mut instance);
    // TC ConflictA from neutral: Horde assaults (no last owner to defend).
    assert_eq!(battle.interact(195150, BgTeam::B, &instance), Ok(245));
    assert_eq!(state(&battle, &instance, 4302), 1);
    battle.advance(59_999, &mut instance);
    assert_eq!(state(&battle, &instance, 4303), 0);
    battle.advance(1, &mut instance);
    assert_eq!(state(&battle, &instance, 4303), 1);
}

#[test]
fn isle_keep_graveyard_can_be_taken_from_its_initial_owner() {
    let (mut battle, mut instance) = fixture();
    assert!(battle.interact(195396, BgTeam::A, &instance).is_err());
    assert_eq!(battle.interact(195396, BgTeam::B, &instance), Ok(245));
    assert_eq!(state(&battle, &instance, 4343), 1);
    battle.advance(60_000, &mut instance);
    assert_eq!(state(&battle, &instance, 4340), 1);
    assert_eq!(state(&battle, &instance, 4339), 0);
}

#[test]
fn isle_boss_death_ends_with_the_opposing_team_winning() {
    let (mut battle, mut instance) = fixture();
    assert!(!battle.npc_killed(12345, &mut instance));
    // High Commander Halford Wyrmbane 34924 is the Alliance boss.
    assert!(battle.npc_killed(34924, &mut instance));
    assert_eq!(instance.phase, BgPhase::Ended);
    assert_eq!(instance.outcome, Some(BgOutcome::Winner(BgTeam::B)));
    let (mut battle, mut instance) = fixture();
    // Overlord Agmar 34922 is the Horde boss.
    assert!(battle.npc_killed(34922, &mut instance));
    assert_eq!(instance.outcome, Some(BgOutcome::Winner(BgTeam::A)));
}

#[test]
fn isle_scoreboard_columns_follow_source_layout() {
    let columns: Vec<_> = pvp_stat_columns(628)
        .iter()
        .map(|column| (column.stat, column.title))
        .collect();
    assert_eq!(columns, [(245, "Bases Assaulted"), (246, "Bases Defended")]);
    assert!(objective_object_entries(628).contains(&195338));
    assert!(is_isle_banner_entry(195399));
}
