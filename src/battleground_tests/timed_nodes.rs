use super::*;
use crate::battleground::{BgMatch, BgOutcome, BgPhase, arathi_basin};

fn instance(template: &BgTemplate) -> BgInstance {
    let mut instance = BgInstance::from_match(
        &BgMatch {
            bg_id: template.id,
            bracket: template.bracket,
            team_a: vec![1],
            team_b: vec![2],
        },
        template,
    );
    instance.start();
    instance
}

#[test]
fn arathi_capture_deadline_resources_contest_defense_and_victory() {
    let template = arathi_basin();
    let mut instance = instance(&template);
    let mut nodes = NodeBattle::new(node_rules(2107).unwrap());
    assert_eq!(nodes.assault(227420, BgTeam::A), Ok(926));
    nodes.advance(59_999, &mut instance, &template);
    assert_eq!(instance.score_a.score, 0);
    assert!(nodes.world_states(&instance).contains(&(5834, 3)));
    nodes.advance(1, &mut instance, &template);
    assert!(nodes.world_states(&instance).contains(&(5834, 5)));
    assert!(nodes.world_states(&instance).contains(&(17322, 2)));
    assert_eq!(instance.score_a.score, 2);
    assert_eq!(nodes.assault(227420, BgTeam::B), Ok(926));
    nodes.advance(2000, &mut instance, &template);
    assert_eq!(
        instance.score_a.score, 2,
        "contested points yield no resources"
    );
    assert_eq!(nodes.assault(227420, BgTeam::A), Ok(927));
    assert!(nodes.world_states(&instance).contains(&(5834, 5)));
    for entry in [227522, 227536, 227538, 227544] {
        assert_eq!(nodes.assault(entry, BgTeam::A), Ok(926));
    }
    nodes.advance(60_000, &mut instance, &template);
    nodes.advance(100_000, &mut instance, &template);
    assert_eq!(instance.phase, BgPhase::Ended);
    assert_eq!(instance.outcome, Some(BgOutcome::Winner(BgTeam::A)));
    assert_eq!(instance.score_a.score, 1500);
    let ended = nodes.world_states(&instance);
    nodes.advance(100_000, &mut instance, &template);
    assert_eq!(nodes.world_states(&instance), ended);
}

#[test]
fn gilneas_uses_three_nodes_one_second_tick_and_its_own_worldstates() {
    let template = crate::battleground::battle_for_gilneas();
    let mut instance = instance(&template);
    let rules = node_rules(761).unwrap();
    let mut nodes = NodeBattle::new(rules);
    for entry in [228050, 228052, 228053] {
        assert_eq!(nodes.assault(entry, BgTeam::B), Ok(370));
    }
    nodes.advance(60_000, &mut instance, &template);
    assert_eq!(instance.score_b.score, 30);
    assert!(nodes.world_states(&instance).contains(&(8851, 4)));
    assert!(nodes.world_states(&instance).contains(&(17355, 2)));
    assert_eq!(nodes.assault(228050, BgTeam::A), Ok(370));
    assert_eq!(nodes.assault(228050, BgTeam::B), Ok(371));
    nodes.advance(49_000, &mut instance, &template);
    assert_eq!(instance.score_b.score, 1500);
    assert_eq!(instance.outcome, Some(BgOutcome::Winner(BgTeam::B)));
}
