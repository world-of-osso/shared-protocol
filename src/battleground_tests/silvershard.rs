use super::*;
use crate::battleground::{BgMatch, BgOutcome, silvershard_mines};

fn source_paths() -> CartPaths {
    serde_json::from_str(include_str!("fixtures/silvershard-paths.json")).unwrap()
}

fn started() -> (BgInstance, BgTemplate, SilvershardBattle) {
    let template = silvershard_mines();
    let mut instance = BgInstance::from_match(
        &BgMatch {
            bg_id: template.id,
            team_a: vec![1, 3, 5],
            team_b: vec![2, 4, 6],
        },
        &template,
    );
    instance.start();
    (instance, template, SilvershardBattle::new())
}

fn escorts(battle: &SilvershardBattle) -> Vec<(u64, BgTeam, [f32; 3])> {
    battle
        .sites()
        .into_iter()
        .enumerate()
        .filter_map(|(index, at)| at.map(|at| (index as u64 * 2 + 1, BgTeam::A, at)))
        .collect()
}

#[test]
fn silvershard_presence_controls_after_one_heartbeat_and_scores_every_two_seconds() {
    let (mut instance, template, mut battle) = started();
    let paths = source_paths();
    let south = [739.29517, 203.76389, 319.54398];
    let players = [(1, BgTeam::A, south)];
    assert!(
        battle
            .advance(999, &players, &paths, &mut instance, &template)
            .is_empty()
    );
    assert_eq!(instance.score_a.score, 0);
    // Spawn stands are all within the source radius 25: one player controls all three.
    assert_eq!(
        battle.advance(1, &players, &paths, &mut instance, &template),
        vec![CartEvent::Controlled(1); 3]
    );
    assert!(battle.world_states(&instance).contains(&(6881, 1)));
    battle.advance(1000, &players, &paths, &mut instance, &template);
    assert_eq!(instance.score_a.score, 3);
    assert!(battle.world_states(&instance).contains(&(6437, 3)));
}

#[test]
fn silvershard_equal_presence_does_not_capture_and_enemy_recontrol_credits_inside_players() {
    let (mut instance, template, mut battle) = started();
    let paths = source_paths();
    let south = [739.29517, 203.76389, 319.54398];
    assert!(
        battle
            .advance(
                1000,
                &[(1, BgTeam::A, south), (2, BgTeam::B, south)],
                &paths,
                &mut instance,
                &template
            )
            .is_empty()
    );
    assert!(battle.world_states(&instance).contains(&(6881, 0)));
    assert_eq!(
        battle.advance(
            1000,
            &[(1, BgTeam::A, south)],
            &paths,
            &mut instance,
            &template
        ),
        vec![CartEvent::Controlled(1); 3]
    );
    let events = battle.advance(
        2000,
        &[(2, BgTeam::B, south), (4, BgTeam::B, south)],
        &paths,
        &mut instance,
        &template,
    );
    assert!(
        events.contains(&CartEvent::Controlled(2)) && events.contains(&CartEvent::Controlled(4))
    );
    assert!(battle.world_states(&instance).contains(&(6882, 1)));
}

#[test]
fn silvershard_source_south_route_delivers_150_and_respawns_after_five_seconds() {
    let (mut instance, template, mut battle) = started();
    let paths = source_paths();
    let mut delivered = None;
    for _ in 0..500 {
        let players = battle.sites()[0]
            .map(|at| vec![(1, BgTeam::A, at)])
            .unwrap_or_default();
        let before = instance.score_a.score;
        let events = battle.advance(1000, &players, &paths, &mut instance, &template);
        for event in events {
            if let CartEvent::Delivered {
                cart: 0,
                team,
                position,
            } = event
            {
                assert_eq!(team, Some(BgTeam::A));
                assert_eq!(position, [563.509, 337.615, 347.005]);
                assert!(instance.score_a.score >= before + 150);
                delivered = Some(position);
            }
        }
        if delivered.is_some() {
            break;
        }
    }
    assert!(delivered.is_some(), "source route must reach the depot");
    assert!(battle.sites()[0].is_none());
    battle.advance(3999, &[], &paths, &mut instance, &template);
    assert!(battle.sites()[0].is_none());
    // Delivery may have happened within the preceding 1s frame, so the last
    // fraction of the exact five-second timer can be consumed by that frame.
    battle.advance(1001, &[], &paths, &mut instance, &template);
    assert!(battle.sites()[0].is_some());
    assert!(battle.world_states(&instance).contains(&(6881, 0)));
}

#[test]
fn silvershard_eastern_track_switch_changes_the_actual_authored_depot() {
    let paths = source_paths();
    for (switch, expected) in [
        (false, [615.415, 79.6458, 298.101]),
        (true, [896.797, 25.6181, 363.99]),
    ] {
        let (mut instance, template, mut battle) = started();
        if switch {
            battle.switch_track(6467).unwrap();
        }
        let mut arrived = None;
        for _ in 0..500 {
            let players = battle.sites()[2]
                .map(|at| vec![(1, BgTeam::A, at)])
                .unwrap_or_default();
            for event in battle.advance(1000, &players, &paths, &mut instance, &template) {
                if let CartEvent::Delivered {
                    cart: 2, position, ..
                } = event
                {
                    arrived = Some(position);
                }
            }
            if arrived.is_some() {
                break;
            }
        }
        assert_eq!(arrived, Some(expected));
    }
}

#[test]
fn silvershard_real_cart_cycles_and_resource_ticks_reach_victory_then_freeze() {
    let (mut instance, template, mut battle) = started();
    let paths = source_paths();
    for _ in 0..1000 {
        let players = escorts(&battle);
        battle.advance(1000, &players, &paths, &mut instance, &template);
        if instance.phase == BgPhase::Ended {
            break;
        }
    }
    assert_eq!(instance.score_a.score, 1500);
    assert_eq!(instance.outcome, Some(BgOutcome::Winner(BgTeam::A)));
    assert!(
        battle
            .advance(10000, &[], &paths, &mut instance, &template)
            .is_empty()
    );
    assert_eq!(instance.score_a.score, 1500);
}
