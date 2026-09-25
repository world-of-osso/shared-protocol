use super::*;

// --- Raid tests ---

#[test]
fn raid_new_puts_leader_in_first_subgroup() {
    let raid = Raid::new(1);
    assert_eq!(raid.leader, 1);
    assert_eq!(raid.total_members(), 1);
    assert_eq!(raid.subgroup_of(1), Some(0));
    assert_eq!(raid.loot_mode, LootMode::PersonalLoot);
}

#[test]
fn collapse_to_party_gathers_members_into_first_subgroup() {
    let mut raid = Raid::new(1);
    raid.add_member(2).unwrap();
    raid.add_member(3).unwrap();
    raid.move_to_subgroup(2, 4).unwrap();
    raid.move_to_subgroup(3, 7).unwrap();
    raid.collapse_to_party().unwrap();
    assert_eq!(raid.subgroups[0], vec![1, 2, 3]);
    assert_eq!(raid.total_members(), 3);
}

#[test]
fn collapse_to_party_rejects_six_members() {
    let mut raid = Raid::new(1);
    for i in 2..=6 {
        raid.add_member(i).unwrap();
    }
    assert_eq!(raid.collapse_to_party(), Err(PartyError::Full));
    assert_eq!(raid.subgroup_of(6), Some(1));
}

#[test]
fn raid_add_fills_subgroups() {
    let mut raid = Raid::new(1);
    // Fill subgroup 0 (already has 1 member)
    for i in 2..=5 {
        raid.add_member(i).unwrap();
    }
    assert_eq!(raid.subgroups[0].len(), 5);
    // Next member goes to subgroup 1
    let group = raid.add_member(6).unwrap();
    assert_eq!(group, 1);
    assert_eq!(raid.subgroup_of(6), Some(1));
}

#[test]
fn raid_full_at_40() {
    let mut raid = Raid::new(1);
    for i in 2..=40 {
        raid.add_member(i).unwrap();
    }
    assert_eq!(raid.total_members(), 40);
    assert_eq!(raid.add_member(41), Err(PartyError::Full));
}

#[test]
fn raid_move_subgroup() {
    let mut raid = Raid::new(1);
    raid.add_member(2).unwrap();
    raid.move_to_subgroup(2, 3).unwrap();
    assert_eq!(raid.subgroup_of(2), Some(3));
    assert_eq!(raid.subgroups[0].len(), 1);
}

#[test]
fn raid_move_to_full_subgroup_fails() {
    let mut raid = Raid::new(1);
    for i in 2..=5 {
        raid.add_member(i).unwrap();
    }
    raid.add_member(6).unwrap(); // goes to group 1
    assert_eq!(raid.move_to_subgroup(6, 0), Err(PartyError::Full));
}

#[test]
fn raid_leave_promotes_leader() {
    let mut raid = Raid::new(1);
    raid.add_member(2).unwrap();
    raid.leave(1);
    assert_eq!(raid.leader, 2);
    assert!(!raid.contains(1));
}

#[test]
fn raid_reject_duplicate() {
    let mut raid = Raid::new(1);
    assert_eq!(raid.add_member(1), Err(PartyError::AlreadyMember));
}

// --- Role assignment tests ---

#[test]
fn role_assign_and_query() {
    let mut roles = RoleAssignments::default();
    roles.assign(1, GroupRole::Tank);
    roles.assign(2, GroupRole::Healer);
    roles.assign(3, GroupRole::Dps);
    assert_eq!(roles.role_of(1), Some(GroupRole::Tank));
    assert_eq!(roles.role_of(2), Some(GroupRole::Healer));
    assert_eq!(roles.role_of(99), None);
}

#[test]
fn role_overwrite() {
    let mut roles = RoleAssignments::default();
    roles.assign(1, GroupRole::Dps);
    roles.assign(1, GroupRole::Tank);
    assert_eq!(roles.role_of(1), Some(GroupRole::Tank));
}

#[test]
fn role_remove() {
    let mut roles = RoleAssignments::default();
    roles.assign(1, GroupRole::Tank);
    roles.remove(1);
    assert_eq!(roles.role_of(1), None);
}

#[test]
fn role_counts() {
    let mut roles = RoleAssignments::default();
    roles.assign(1, GroupRole::Tank);
    roles.assign(2, GroupRole::Healer);
    roles.assign(3, GroupRole::Dps);
    roles.assign(4, GroupRole::Dps);
    roles.assign(5, GroupRole::Dps);
    assert_eq!(roles.counts(), (1, 1, 3));
}

#[test]
fn role_standard_composition() {
    let mut roles = RoleAssignments::default();
    roles.assign(1, GroupRole::Tank);
    roles.assign(2, GroupRole::Healer);
    roles.assign(3, GroupRole::Dps);
    assert!(roles.has_standard_composition());
}

#[test]
fn role_no_tank_not_standard() {
    let mut roles = RoleAssignments::default();
    roles.assign(1, GroupRole::Healer);
    roles.assign(2, GroupRole::Dps);
    assert!(!roles.has_standard_composition());
}

// --- Ready check tests ---

#[test]
fn ready_check_all_ready() {
    let mut check = ReadyCheck::new(&[1, 2, 3]);
    assert!(!check.all_responded());
    check.respond(1, ReadyResponse::Ready);
    check.respond(2, ReadyResponse::Ready);
    check.respond(3, ReadyResponse::Ready);
    assert!(check.all_responded());
    assert!(check.all_ready());
}

#[test]
fn ready_check_not_ready() {
    let mut check = ReadyCheck::new(&[1, 2]);
    check.respond(1, ReadyResponse::Ready);
    check.respond(2, ReadyResponse::NotReady);
    assert!(check.all_responded());
    assert!(!check.all_ready());
}

#[test]
fn ready_check_timeout() {
    let mut check = ReadyCheck::new(&[1, 2]);
    check.respond(1, ReadyResponse::Ready);
    assert!(!check.tick(20.0));
    assert!(check.tick(15.0)); // 35s > 30s timeout
}

#[test]
fn ready_check_counts() {
    let mut check = ReadyCheck::new(&[1, 2, 3, 4]);
    check.respond(1, ReadyResponse::Ready);
    check.respond(2, ReadyResponse::NotReady);
    let (pending, ready, not_ready) = check.counts();
    assert_eq!(pending, 2);
    assert_eq!(ready, 1);
    assert_eq!(not_ready, 1);
}

#[test]
fn ready_check_unknown_player() {
    let mut check = ReadyCheck::new(&[1, 2]);
    assert!(!check.respond(99, ReadyResponse::Ready));
}

// --- Shared threat tests ---

#[test]
fn group_damage_threat_shared_table() {
    let mut table = ThreatTable::default();
    // Two party members both damage the same mob
    apply_group_damage_threat(&mut table, 1, 500.0, 1.0);
    apply_group_damage_threat(&mut table, 2, 300.0, 1.0);
    assert_eq!(table.threat_for(1), 500.0);
    assert_eq!(table.threat_for(2), 300.0);
    assert_eq!(table.top_threat().unwrap().entity, 1);
}

#[test]
fn group_heal_threat_split_across_mobs() {
    let mut table1 = ThreatTable::default();
    let mut table2 = ThreatTable::default();
    // Healer heals 1000, split across 2 mobs
    apply_group_heal_threat(&mut [&mut table1, &mut table2], 3, 1000.0, 1.0);
    // Each mob gets 500 heal → 500 * 0.5 = 250 threat
    assert!((table1.threat_for(3) - 250.0).abs() < 0.01);
    assert!((table2.threat_for(3) - 250.0).abs() < 0.01);
}

#[test]
fn group_heal_no_mobs_no_crash() {
    apply_group_heal_threat(&mut [], 1, 1000.0, 1.0);
}

#[test]
fn engaged_mobs_filters_by_group() {
    let mut t1 = ThreatTable::default();
    t1.add_damage_threat(1, 100.0, 1.0); // player 1 in group

    let mut t2 = ThreatTable::default();
    t2.add_damage_threat(99, 100.0, 1.0); // player 99 NOT in group

    let mobs: Vec<(u64, &ThreatTable)> = vec![(10, &t1), (20, &t2)];
    let group_members = vec![1, 2, 3];
    let engaged = engaged_mobs(&mobs, &group_members);
    assert_eq!(engaged, vec![10]); // only mob 10 is engaged by the group
}

#[test]
fn heal_threat_with_tank_modifier() {
    let mut table = ThreatTable::default();
    // Tank heals with 1.43x threat modifier
    apply_group_heal_threat(&mut [&mut table], 1, 1000.0, 1.43);
    // 1000 * 0.5 * 1.43 = 715
    assert!((table.threat_for(1) - 715.0).abs() < 0.01);
}
