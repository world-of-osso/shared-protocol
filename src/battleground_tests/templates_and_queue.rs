use super::*;

#[test]
fn deephaul_source_dropped_crystal_is_usable() {
    let crystal = crate::protocol::GameObjectInfo {
        entry: 422412,
        go_type: 37,
        display_id: 91024,
        name: "Deephaul Crystal".into(),
        scale: 1.0,
    };
    assert!(crystal.is_battleground_flag());
    assert!(crystal.is_usable());
}

#[test]
fn deephaul_source_queue_flag_and_scoreboard_columns() {
    let template = template_by_id(1110).expect("Mainline Deephaul queue1110");
    assert_eq!(
        (
            template.map_id,
            template.min_players_per_team,
            template.team_size
        ),
        (2656, 5, 10)
    );
    assert_eq!(template.bracket, LevelBracket::new(19, 90));
    assert_eq!((template.win_score, template.max_duration_secs), (1500, 0));
    assert_eq!(objective_object_entries(2656), [422413]);
    let columns: Vec<_> = pvp_stat_columns(2656)
        .iter()
        .map(|column| (column.stat, column.title))
        .collect();
    assert_eq!(
        columns,
        [(1020, "Flag Captures"), (1021, "Carts Controlled")]
    );
}

#[test]
fn seething_source_queue_and_scoreboard_column() {
    let template = template_by_id(894).expect("Mainline Seething Shore queue 894");
    assert_eq!(
        (
            template.map_id,
            template.min_players_per_team,
            template.team_size
        ),
        (1803, 5, 10)
    );
    assert_eq!(template.bracket, LevelBracket::new(40, 90));
    assert_eq!((template.win_score, template.max_duration_secs), (1500, 0));
    let columns: Vec<_> = pvp_stat_columns(1803)
        .iter()
        .map(|column| (column.stat, column.title))
        .collect();
    assert_eq!(columns, [(744, "Azerite Collected")]);
}

#[test]
fn silvershard_source_queue_and_scoreboard_column() {
    let template = template_by_id(708).expect("Mainline Silvershard queue 708");
    assert_eq!(
        (
            template.map_id,
            template.min_players_per_team,
            template.team_size
        ),
        (727, 3, 10)
    );
    assert_eq!(template.bracket, LevelBracket::new(30, 90));
    assert_eq!((template.win_score, template.max_duration_secs), (1500, 0));
    let columns: Vec<_> = pvp_stat_columns(727)
        .iter()
        .map(|column| (column.stat, column.title))
        .collect();
    assert_eq!(columns, [(428, "Carts Controlled")]);
}

#[test]
fn kotmogu_source_queue_and_objective_columns() {
    let template = template_by_id(699).expect("Mainline Kotmogu queue 699");
    assert_eq!(
        (
            template.map_id,
            template.min_players_per_team,
            template.team_size
        ),
        (998, 2, 10)
    );
    assert_eq!(template.bracket, LevelBracket::new(40, 90));
    assert_eq!((template.win_score, template.max_duration_secs), (1500, 0));
    assert_eq!(
        objective_object_entries(998),
        [212094, 212091, 212093, 212092]
    );
    let columns: Vec<_> = pvp_stat_columns(998)
        .iter()
        .map(|column| (column.stat, column.title))
        .collect();
    assert_eq!(columns, [(418, "Orb Possessions"), (419, "Victory Points")]);
}

#[test]
fn wsg_template() {
    let wsg = warsong_gulch();
    assert_eq!(wsg.id, 1014);
    assert_eq!(wsg.map_id, 2106);
    assert_eq!(wsg.team_size, 10);
    assert_eq!(wsg.objective, BgObjective::FlagCapture);
    assert_eq!(wsg.win_score, 3);
    assert_eq!(wsg.max_duration_secs, 25 * 60);
}

#[test]
fn ab_template() {
    let ab = arathi_basin();
    assert_eq!(ab.id, 1018);
    assert_eq!(ab.map_id, 2107);
    assert_eq!(ab.bracket, LevelBracket::new(10, 90));
    assert_eq!(ab.team_size, 15);
    assert_eq!(ab.objective, BgObjective::NodeControl);
    assert_eq!(ab.win_score, 1500);
}

#[test]
fn epic_source_templates() {
    let av = template_by_id(1).expect("Mainline Alterac queue1");
    let isle = template_by_id(30).expect("Mainline Isle queue30");
    for (template, map, reinforcements) in [(av, 30, 700), (isle, 628, 400)] {
        assert_eq!(template.map_id, map);
        assert_eq!(template.team_size, 40);
        assert_eq!(template.min_players_per_team, 10);
        assert_eq!(template.bracket, LevelBracket::new(20, 90));
        assert_eq!(template.objective, BgObjective::Reinforcements);
        assert_eq!(template.starting_reinforcements, reinforcements);
        assert_eq!(template.max_duration_secs, 0);
        assert_eq!(template.win_score, 0);
    }
    assert!(template_by_id(3).is_none(), "queue3 is not Retail AV");
}

#[test]
fn epic_source_minimum_rosters_and_level_bracket() {
    for id in [1, 30] {
        let mut queue = BgQueue::default();
        assert_eq!(
            queue.join_solo(1, 19, id, 0),
            Err(BgQueueError::OutOfBracket)
        );
        for player in 1..20 {
            queue.join_solo(player, 20, id, 0).unwrap();
        }
        assert!(queue.try_match(id).is_none());
        queue.join_solo(20, 90, id, 0).unwrap();
        assert!(
            queue.try_match(id).is_none(),
            "level90 has its own source bracket"
        );
        queue.join_solo(21, 79, id, 0).unwrap();
        let matched = queue
            .try_match(id)
            .expect("ten per team admits source minimum");
        assert_eq!(matched.team_a.len(), 10);
        assert_eq!(matched.team_b.len(), 10);
        assert_eq!(matched.bracket, LevelBracket::new(20, 79));
        assert!(queue.is_queued(20));
    }
}

#[test]
fn all_templates_lists_source_queue_maps() {
    let pairs: Vec<_> = all_templates().iter().map(|t| (t.id, t.map_id)).collect();
    assert_eq!(
        pairs,
        [
            (1014, 2106),
            (1018, 2107),
            (120, 761),
            (108, 726),
            (7, 566),
            (699, 998),
            (708, 727),
            (894, 1803),
            (1110, 2656),
            (1, 30),
            (30, 628)
        ]
    );
}

#[test]
fn template_ids_unique() {
    let templates = all_templates();
    let mut ids: Vec<u32> = templates.iter().map(|t| t.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), templates.len());
}

#[test]
fn template_map_ids_unique() {
    let templates = all_templates();
    let mut maps: Vec<u32> = templates.iter().map(|t| t.map_id).collect();
    maps.sort();
    maps.dedup();
    assert_eq!(maps.len(), templates.len());
}

#[test]
fn lookup_by_id() {
    assert_eq!(template_by_id(1014).unwrap().name, "Warsong Gulch");
    assert_eq!(template_by_id(1018).unwrap().name, "Arathi Basin");
    assert_eq!(template_by_id(1).unwrap().name, "Alterac Valley");
    assert_eq!(template_by_id(30).unwrap().name, "Isle of Conquest");
    assert_eq!(template_by_id(120).unwrap().name, "The Battle for Gilneas");
    assert!(template_by_id(999).is_none());
}

#[test]
fn min_players_below_team_size() {
    for tmpl in all_templates() {
        assert!(
            tmpl.min_players_per_team <= tmpl.team_size,
            "{}: min {} > team_size {}",
            tmpl.name,
            tmpl.min_players_per_team,
            tmpl.team_size,
        );
    }
}

#[test]
fn level_bracket_contains() {
    let bracket = LevelBracket::new(10, 60);
    assert!(!bracket.contains(9));
    assert!(bracket.contains(10));
    assert!(bracket.contains(35));
    assert!(bracket.contains(60));
    assert!(!bracket.contains(61));
}

#[test]
fn all_templates_have_valid_brackets() {
    for tmpl in all_templates() {
        assert!(
            tmpl.bracket.min_level <= tmpl.bracket.max_level,
            "{}",
            tmpl.name
        );
        assert!(tmpl.bracket.min_level >= 1, "{}", tmpl.name);
    }
}

#[test]
fn serialization_round_trip() {
    let wsg = warsong_gulch();
    let json = serde_json::to_string(&wsg).unwrap();
    let restored: BgTemplate = serde_json::from_str(&json).unwrap();
    assert_eq!(wsg, restored);
}

#[test]
fn join_solo_and_is_queued() {
    let mut q = BgQueue::default();
    assert!(q.join_solo(1, 30, warsong_gulch().id, 100).is_ok());
    assert!(q.is_queued(1));
    assert_eq!(q.len(), 1);
}

#[test]
fn join_solo_already_queued() {
    let mut q = BgQueue::default();
    q.join_solo(1, 30, warsong_gulch().id, 100).unwrap();
    assert_eq!(
        q.join_solo(1, 30, warsong_gulch().id, 101),
        Err(BgQueueError::AlreadyQueued)
    );
}

#[test]
fn join_unknown_bg() {
    let mut q = BgQueue::default();
    assert_eq!(q.join_solo(1, 30, 999, 100), Err(BgQueueError::UnknownBg));
}

#[test]
fn join_out_of_bracket() {
    let mut q = BgQueue::default();
    assert_eq!(
        q.join_solo(1, 5, warsong_gulch().id, 100),
        Err(BgQueueError::OutOfBracket)
    );
    assert_eq!(
        q.join_solo(1, 91, warsong_gulch().id, 100),
        Err(BgQueueError::OutOfBracket)
    );
}

#[test]
fn join_group() {
    let mut q = BgQueue::default();
    q.join_group(1, vec![1, 2, 3], vec![30, 30, 30], warsong_gulch().id, 100)
        .unwrap();
    assert!(q.is_queued(1));
    assert!(q.is_queued(2));
    assert!(q.is_queued(3));
    assert_eq!(q.len(), 1);
}

#[test]
fn join_empty_group() {
    let mut q = BgQueue::default();
    assert_eq!(
        q.join_group(1, vec![], vec![], warsong_gulch().id, 100),
        Err(BgQueueError::EmptyGroup)
    );
}

#[test]
fn join_group_rejects_level_count_mismatch() {
    let mut q = BgQueue::default();
    assert_eq!(
        q.join_group(1, vec![1, 2, 3], vec![30, 30], warsong_gulch().id, 100),
        Err(BgQueueError::LevelCountMismatch)
    );
}

#[test]
fn join_group_rejects_duplicate_members() {
    let mut q = BgQueue::default();
    assert_eq!(
        q.join_group(1, vec![1, 2, 2], vec![30, 30, 30], warsong_gulch().id, 100),
        Err(BgQueueError::DuplicatePlayer)
    );
}

#[test]
fn join_group_rejects_member_already_queued_elsewhere() {
    let mut q = BgQueue::default();
    q.join_group(1, vec![1, 2, 3], vec![30, 30, 30], warsong_gulch().id, 100)
        .unwrap();
    assert_eq!(
        q.join_group(4, vec![3, 4, 5], vec![30, 30, 30], warsong_gulch().id, 101),
        Err(BgQueueError::AlreadyQueued)
    );
}

#[test]
fn leave_queue() {
    let mut q = BgQueue::default();
    q.join_solo(1, 30, warsong_gulch().id, 100).unwrap();
    assert!(q.leave(1));
    assert!(!q.is_queued(1));
    assert!(q.is_empty());
}

#[test]
fn leave_not_queued() {
    let mut q = BgQueue::default();
    assert!(!q.leave(999));
}

#[test]
fn match_not_enough_players() {
    let mut q = BgQueue::default();
    for i in 1..=8 {
        q.join_solo(i, 30, warsong_gulch().id, 100).unwrap();
    }
    assert!(q.try_match(warsong_gulch().id).is_none());
    assert_eq!(q.len(), 8);
}

#[test]
fn match_forms_two_teams() {
    let mut q = BgQueue::default();
    for i in 1..=10 {
        q.join_solo(i, 30, warsong_gulch().id, 100).unwrap();
    }
    let m = q.try_match(warsong_gulch().id).unwrap();
    assert_eq!(m.bg_id, 1014);
    assert_eq!(m.team_a.len(), 5);
    assert_eq!(m.team_b.len(), 5);

    let mut all: Vec<u64> = m.team_a.iter().chain(&m.team_b).copied().collect();
    all.sort();
    assert_eq!(all, (1..=10).collect::<Vec<u64>>());
    assert!(q.is_empty());
}

#[test]
fn match_with_groups() {
    let mut q = BgQueue::default();
    q.join_group(1, vec![1, 2, 3], vec![30, 30, 30], warsong_gulch().id, 100)
        .unwrap();
    q.join_group(4, vec![4, 5, 6], vec![30, 30, 30], warsong_gulch().id, 100)
        .unwrap();
    for i in 7..=10 {
        q.join_solo(i, 30, warsong_gulch().id, 100).unwrap();
    }
    let m = q.try_match(warsong_gulch().id).unwrap();
    let total = m.team_a.len() + m.team_b.len();
    assert_eq!(total, 10);
}

#[test]
fn match_different_bg_types_independent() {
    let mut q = BgQueue::default();
    for i in 1..=10 {
        q.join_solo(i, 30, warsong_gulch().id, 100).unwrap();
    }
    for i in 11..=18 {
        q.join_solo(i, 30, arathi_basin().id, 100).unwrap();
    }
    assert!(q.try_match(warsong_gulch().id).is_some());
    assert!(q.try_match(arathi_basin().id).is_none());
}

#[test]
fn source_bracket_preserves_authored_epic_and_max_level_boundaries() {
    for map in [30, 628] {
        assert_eq!(
            source_level_bracket(map, 20),
            Some(LevelBracket::new(20, 79))
        );
        assert_eq!(
            source_level_bracket(map, 79),
            Some(LevelBracket::new(20, 79))
        );
        assert_eq!(
            source_level_bracket(map, 80),
            Some(LevelBracket::new(80, 80))
        );
        assert_eq!(
            source_level_bracket(map, 81),
            Some(LevelBracket::new(81, 89))
        );
        assert_eq!(
            source_level_bracket(map, 90),
            Some(LevelBracket::new(90, 90))
        );
    }
    assert_eq!(source_level_bracket(2656, 79), None);
    let mut queue = BgQueue::default();
    assert_eq!(
        queue.join_solo(1, 79, 1110, 0),
        Err(BgQueueError::OutOfBracket)
    );
    assert!(queue.join_solo(1, 80, 1110, 0).is_ok());
    assert_eq!(source_level_bracket(999999, 30), None);
}

#[test]
fn source_bracket_rejects_a_group_crossing_the_39_to_40_boundary() {
    let mut queue = BgQueue::default();
    assert_eq!(
        queue.join_group(1, vec![1, 2], vec![39, 40], 1014, 0),
        Err(BgQueueError::OutOfBracket)
    );
    assert!(queue.is_empty(), "rejected group leaves no queued members");
}

#[test]
fn source_bracket_queue_keeps_29_separate_without_blocking_a_full_30_to_39_match() {
    let mut queue = BgQueue::default();
    queue.join_solo(1, 29, 1014, 0).unwrap();
    for player in 2..=11 {
        let level = if player % 2 == 0 { 30 } else { 39 };
        queue.join_solo(player, level, 1014, 0).unwrap();
    }
    let matched = queue.try_match(1014).unwrap();
    assert_eq!(matched.bracket, LevelBracket::new(30, 39));
    let instance = BgInstance::from_match(&matched, &warsong_gulch());
    assert_eq!(instance.bracket, LevelBracket::new(30, 39));
    let mut players: Vec<_> = matched
        .team_a
        .iter()
        .chain(&matched.team_b)
        .copied()
        .collect();
    players.sort();
    assert_eq!(players, (2..=11).collect::<Vec<_>>());
    assert!(
        queue.is_queued(1),
        "older incomplete bracket remains queued"
    );
    assert!(queue.try_match(1014).is_none());
}

#[test]
fn match_unknown_bg_returns_none() {
    let mut q = BgQueue::default();
    assert!(q.try_match(999).is_none());
}
