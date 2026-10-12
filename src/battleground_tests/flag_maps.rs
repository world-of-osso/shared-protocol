use super::*;
use crate::battleground::{
    BgMatch, BgOutcome, BgPhase, LevelBracket, pvp_stat_columns, template_by_id, twin_peaks,
};

fn started(template: &BgTemplate) -> BgInstance {
    let mut instance = BgInstance::from_match(
        &BgMatch {
            bg_id: template.id,
            team_a: vec![1],
            team_b: vec![2],
        },
        template,
    );
    instance.start();
    instance
}

fn capture(battle: &mut FlagBattle, team: BgTeam, carrier: u64, instance: &mut BgInstance) {
    let template = template_by_id(instance.bg_id).unwrap();
    assert!(battle.flags.pickup(team, carrier));
    assert!(battle.capture(team, carrier, instance, &template));
}

#[test]
fn twin_peaks_queue_identity_and_scoreboard_columns_are_sourced() {
    let template = template_by_id(108).unwrap();
    assert_eq!(template, twin_peaks());
    assert_eq!(
        (
            template.map_id,
            template.min_players_per_team,
            template.team_size
        ),
        (726, 5, 10)
    );
    assert_eq!(template.bracket, LevelBracket::new(30, 90));
    let rules = flag_rules(726).unwrap();
    assert_eq!(rules.starts, [1726, 1727]);
    assert_eq!((rules.capture_stat, rules.return_stat), (290, 291));
    let titles: Vec<_> = pvp_stat_columns(726)
        .iter()
        .map(|c| (c.stat, c.title))
        .collect();
    assert_eq!(titles, [(290, "Flag Captures"), (291, "Flag Returns")]);
    let warsong: Vec<_> = pvp_stat_columns(2106).iter().map(|c| c.stat).collect();
    assert_eq!(
        warsong,
        [929, 928],
        "PvpScoreboardLayout orders WSG returns first"
    );
}

#[test]
fn twin_peaks_third_capture_wins_and_worldstates_track_carried_flags() {
    let template = twin_peaks();
    let mut instance = started(&template);
    let mut battle = FlagBattle::new(flag_rules(726).unwrap());
    assert!(battle.flags.pickup(BgTeam::A, 1));
    let carried = battle.world_states(&instance);
    assert!(
        carried.contains(&(1546, 2)),
        "horde flag taken: {carried:?}"
    );
    assert!(carried.contains(&(2339, 2)));
    assert!(carried.contains(&(1545, 1)));
    assert!(battle.capture(BgTeam::A, 1, &mut instance, &template));
    capture(&mut battle, BgTeam::A, 1, &mut instance);
    let states = battle.world_states(&instance);
    assert!(states.contains(&(1581, 2)) && states.contains(&(1546, 1)));
    capture(&mut battle, BgTeam::A, 1, &mut instance);
    assert_eq!(instance.outcome, Some(BgOutcome::Winner(BgTeam::A)));
    assert!(battle.world_states(&instance).contains(&(4247, 0)));
}

#[test]
fn flag_time_limit_applies_source_tie_breaks_after_fifteen_minutes() {
    let template = twin_peaks();
    let mut leading = started(&template);
    let mut battle = FlagBattle::new(flag_rules(726).unwrap());
    capture(&mut battle, BgTeam::B, 2, &mut leading);
    battle.advance(FLAG_MATCH_MS - 1, &mut leading);
    assert_eq!(leading.phase, BgPhase::InProgress);
    battle.advance(1, &mut leading);
    assert_eq!(leading.outcome, Some(BgOutcome::Winner(BgTeam::B)));

    let mut tied = started(&template);
    let mut battle = FlagBattle::new(flag_rules(726).unwrap());
    capture(&mut battle, BgTeam::B, 2, &mut tied);
    capture(&mut battle, BgTeam::A, 1, &mut tied);
    battle.advance(FLAG_MATCH_MS, &mut tied);
    assert_eq!(
        tied.outcome,
        Some(BgOutcome::Winner(BgTeam::A)),
        "last capture breaks ties"
    );

    let mut scoreless = started(&crate::battleground::warsong_gulch());
    let mut battle = FlagBattle::new(flag_rules(2106).unwrap());
    battle.advance(FLAG_MATCH_MS, &mut scoreless);
    assert_eq!(scoreless.outcome, Some(BgOutcome::Draw));
}
