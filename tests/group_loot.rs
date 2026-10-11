use shared::loot::{LootDrop, LootRollChoice, RollWindow};

#[test]
fn transmog_outranks_greed_but_not_need() {
    let mut roll = RollWindow::new(
        LootDrop {
            item_id: 2379,
            count: 1,
        },
        &[11, 22, 33],
        &[11],
        &[],
    );
    roll.can_transmog = vec![22];
    assert!(roll.respond(33, LootRollChoice::Greed, 100));
    assert!(roll.respond(22, LootRollChoice::Transmog, 1));
    assert_eq!(roll.resolve(), None);
    assert!(roll.respond(11, LootRollChoice::Pass, 0));
    assert_eq!(roll.resolve(), Some(22));
}

#[test]
fn appropriate_class_transmog_beats_higher_nonpreferred_transmog() {
    let mut roll = RollWindow::new(
        LootDrop {
            item_id: 15379,
            count: 1,
        },
        &[11, 22],
        &[22],
        &[22],
    );
    roll.can_transmog = vec![11, 22];
    assert!(roll.respond(11, LootRollChoice::Transmog, 99));
    assert!(roll.respond(22, LootRollChoice::Transmog, 1));
    assert_eq!(roll.resolve(), Some(22));
}

#[test]
fn ties_are_independent_of_response_order_and_responses_are_single_use() {
    let mut roll = RollWindow::new(
        LootDrop {
            item_id: 1206,
            count: 1,
        },
        &[11, 22],
        &[11, 22],
        &[],
    );
    assert!(!roll.respond(33, LootRollChoice::Greed, 100));
    assert!(!roll.respond(11, LootRollChoice::Need, 101));
    assert!(!roll.respond(11, LootRollChoice::Transmog, 100));
    assert!(roll.respond(22, LootRollChoice::Need, 50));
    assert!(!roll.respond(22, LootRollChoice::Need, 100));
    assert!(roll.respond(11, LootRollChoice::Need, 50));
    assert_eq!(roll.resolve(), Some(11));
}

#[test]
fn non_need_member_can_still_greed_and_win() {
    let mut roll = RollWindow::new(
        LootDrop {
            item_id: 1206,
            count: 1,
        },
        &[11, 22],
        &[11],
        &[],
    );
    assert!(roll.respond(22, LootRollChoice::Greed, 99));
    assert!(roll.respond(11, LootRollChoice::Pass, 0));
    assert_eq!(roll.resolve(), Some(22));
}

#[test]
fn need_is_rejected_without_consuming_the_members_response() {
    let mut roll = RollWindow::new(
        LootDrop {
            item_id: 1206,
            count: 1,
        },
        &[11, 22],
        &[11],
        &[],
    );
    assert!(!roll.respond(22, LootRollChoice::Need, 100));
    assert!(roll.respond(22, LootRollChoice::Greed, 100));
    assert!(roll.respond(11, LootRollChoice::Need, 1));
    assert_eq!(roll.resolve(), Some(11));
}

#[test]
fn unsubmitted_responses_become_pass_at_timeout() {
    let mut roll = RollWindow::new(
        LootDrop {
            item_id: 1206,
            count: 1,
        },
        &[11, 22],
        &[11, 22],
        &[],
    );
    assert!(roll.respond(11, LootRollChoice::Greed, 42));
    assert!(!roll.tick(59.0));
    assert!(roll.tick(1.0));
    assert_eq!(roll.resolve(), Some(11));
    assert_eq!(roll.responses.len(), 2);
    assert!(!roll.respond(22, LootRollChoice::Need, 100));
}
