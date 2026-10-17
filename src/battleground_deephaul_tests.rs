use super::*;

fn running() -> (DeephaulBattle, CartPaths, BgInstance, BgTemplate) {
    let template = deephaul_ravine();
    let matched = BgMatch {
        bg_id: template.id,
        bracket: template.bracket,
        team_a: vec![1, 2, 3, 4, 5],
        team_b: vec![6, 7, 8, 9, 10],
    };
    let mut instance = BgInstance::from_match(&matched, &template);
    assert!(instance.start());
    let paths = serde_json::from_str(include_str!("fixtures/deephaul_cart_paths.json")).unwrap();
    (DeephaulBattle::new(), paths, instance, template)
}

#[test]
fn deephaul_source_faction_carts_and_fifteen_second_crystal_enable() {
    let (mut battle, paths, mut instance, template) = running();
    assert_eq!(
        battle.sites(),
        [
            Some([3875.0, -3150.0, 240.2913]),
            Some([4250.3647, -2751.066, 239.46973])
        ]
    );
    assert!(battle.world_states(&instance).contains(&(25415, 1)));
    assert!(battle.world_states(&instance).contains(&(25420, 1)));
    assert_eq!(battle.flag_site(), None);
    assert!(!battle.pickup(BgTeam::A, 1));
    battle.advance(14_999, &[], &paths, &mut instance, &template);
    assert_eq!(battle.flag_site(), None);
    battle.advance(1, &[], &paths, &mut instance, &template);
    assert_eq!(battle.flag_site(), Some(FlagSite::Home));
    assert!(battle.world_states(&instance).contains(&(25412, 3)));
    assert_eq!((instance.score_a.score, instance.score_b.score), (21, 21));
}

#[test]
fn deephaul_presence_changes_west_control_once_and_scores_three_per_cart() {
    let (mut battle, paths, mut instance, template) = running();
    let mut controlled = Vec::new();
    for _ in 0..26 {
        let at = battle.sites()[1].unwrap();
        controlled.extend(battle.advance(
            1000,
            &[(1, BgTeam::A, at)],
            &paths,
            &mut instance,
            &template,
        ));
    }
    assert_eq!(
        controlled
            .iter()
            .filter(|event| **event == CartEvent::Controlled(1))
            .count(),
        1
    );
    assert!(battle.world_states(&instance).contains(&(25421, 1)));
    assert!(battle.world_states(&instance).contains(&(25420, 0)));
    assert_eq!((instance.score_a.score, instance.score_b.score), (42, 36));
}

#[test]
fn deephaul_crystal_team_trigger_drop_expiry_and_thirty_second_respawn() {
    let (mut battle, paths, mut instance, template) = running();
    battle.advance(15_000, &[], &paths, &mut instance, &template);
    assert!(battle.pickup(BgTeam::A, 1));
    assert!(!battle.pickup(BgTeam::B, 6));
    assert!(!battle.capture(
        1,
        BgTeam::A,
        DEEPHAUL_CAPTURE_TRIGGERS[1],
        &mut instance,
        &template
    ));
    let at = [4062.62, -2949.4, 204.736];
    battle.drop_flag(at);
    assert_eq!(battle.flag_site(), Some(FlagSite::Dropped(at)));
    battle.advance(9_999, &[], &paths, &mut instance, &template);
    assert_eq!(battle.flag_site(), Some(FlagSite::Dropped(at)));
    battle.advance(1, &[], &paths, &mut instance, &template);
    assert_eq!(battle.flag_site(), Some(FlagSite::Home));
    assert!(battle.pickup(BgTeam::A, 1));
    let before = instance.score_a.score;
    assert!(battle.capture(
        1,
        BgTeam::A,
        DEEPHAUL_CAPTURE_TRIGGERS[0],
        &mut instance,
        &template
    ));
    assert_eq!(instance.score_a.score, before + 100);
    assert_eq!(battle.flag_site(), None);
    battle.advance(29_999, &[], &paths, &mut instance, &template);
    assert_eq!(battle.flag_site(), None);
    battle.advance(1, &[], &paths, &mut instance, &template);
    assert_eq!(battle.flag_site(), Some(FlagSite::Home));
}

#[test]
fn deephaul_both_cart_arrivals_deliver_together_then_respawn_after_two_seconds() {
    let (mut battle, paths, mut instance, template) = running();
    let mut delivered = Vec::new();
    for _ in 0..700 {
        delivered = battle.advance(1000, &[], &paths, &mut instance, &template);
        if delivered
            .iter()
            .any(|event| matches!(event, CartEvent::Delivered { .. }))
        {
            break;
        }
    }
    assert_eq!(
        delivered
            .iter()
            .filter(|event| matches!(event, CartEvent::Delivered { .. }))
            .count(),
        2
    );
    assert_eq!(battle.sites(), [None, None]);
    assert_eq!(instance.score_a.score, instance.score_b.score);
    battle.advance(1999, &[], &paths, &mut instance, &template);
    assert_eq!(battle.sites(), [None, None]);
    battle.advance(1, &[], &paths, &mut instance, &template);
    assert!(battle.sites().into_iter().all(|site| site.is_some()));
}

#[test]
fn deephaul_equal_natural_cart_income_reaches_a_simultaneous_draw() {
    let (mut battle, paths, mut instance, template) = running();
    battle.advance(1_500_000, &[], &paths, &mut instance, &template);
    assert_eq!(instance.phase, BgPhase::Ended);
    assert_eq!(
        (instance.score_a.score, instance.score_b.score),
        (1500, 1500)
    );
    assert_eq!(instance.outcome, Some(BgOutcome::Draw));
}
