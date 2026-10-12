use super::*;

fn running() -> (SeethingBattle, BgInstance, BgTemplate) {
    let template = seething_shore();
    let matched = BgMatch {
        bg_id: template.id,
        team_a: vec![1, 2, 3, 4, 5],
        team_b: vec![6, 7, 8, 9, 10],
    };
    let mut instance = BgInstance::from_match(&matched, &template);
    assert!(instance.start());
    (SeethingBattle::new(), instance, template)
}

#[test]
fn seething_initial_three_geysers_wait_forty_seconds_before_objects() {
    let (mut battle, _, _) = running();
    assert!(battle.advance(1999, &mut |_| 0).is_empty());
    let events = battle.advance(1, &mut |_| 0);
    let areas: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            AzeriteEvent::Activated { area, .. } => Some(*area),
            _ => None,
        })
        .collect();
    assert_eq!(areas, [9516, 9515, 9521]);
    assert!(battle.advance(39_999, &mut |_| 0).is_empty());
    let events = battle.advance(1, &mut |_| 0);
    let spawns: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            AzeriteEvent::Spawned { area, entry, .. } => Some((*area, *entry)),
            _ => None,
        })
        .collect();
    assert_eq!(spawns, [(9515, 281307), (9516, 281307), (9521, 281307)]);
}

#[test]
fn seething_capture_scores_once_removes_after_two_seconds_and_replenishes() {
    let (mut battle, mut instance, template) = running();
    let events = battle.advance(42_000, &mut |_| 0);
    let generation = events
        .iter()
        .find_map(|event| match event {
            AzeriteEvent::Spawned { generation, .. } => Some(*generation),
            _ => None,
        })
        .unwrap();
    battle
        .capture(generation, BgTeam::A, &mut instance, &template)
        .unwrap();
    assert_eq!((instance.score_a.score, instance.score_b.score), (100, 0));
    assert!(
        battle
            .capture(generation, BgTeam::B, &mut instance, &template)
            .is_err()
    );
    assert!(battle.advance(1999, &mut |_| 0).is_empty());
    let events = battle.advance(1, &mut |_| 0);
    assert!(events.contains(&AzeriteEvent::Removed { generation }));
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, AzeriteEvent::Activated { .. }))
            .count(),
        1
    );
    assert_eq!(
        battle.world_states(&instance),
        [(13845, 100), (13844, 0), (13846, 1500)]
    );
}

#[test]
fn seething_fifteen_real_resource_captures_end_at_source_limit() {
    let (mut battle, mut instance, template) = running();
    let mut captures = 0;
    for _ in 0..15 {
        let events = battle.advance(42_000, &mut |_| 0);
        for event in events {
            if let AzeriteEvent::Spawned { generation, .. } = event {
                if instance.phase == BgPhase::InProgress {
                    battle
                        .capture(generation, BgTeam::B, &mut instance, &template)
                        .unwrap();
                    captures += 1;
                }
            }
        }
        if instance.phase == BgPhase::Ended {
            break;
        }
    }
    assert_eq!(captures, 15);
    assert_eq!((instance.score_a.score, instance.score_b.score), (0, 1500));
    assert_eq!(instance.outcome, Some(BgOutcome::Winner(BgTeam::B)));
    assert!(
        battle
            .capture(1, BgTeam::A, &mut instance, &template)
            .is_err()
    );
}
