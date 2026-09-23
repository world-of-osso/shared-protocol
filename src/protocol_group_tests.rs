use super::*;
use crate::loot::LootMode;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::AppMessageExt;

#[test]
fn protocol_plugin_registers_group_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<RespondGroupInvite>());
    assert!(app.is_message_registered::<LeaveGroup>());
    assert!(app.is_message_registered::<PromoteGroupLeader>());
    assert!(app.is_message_registered::<ConvertGroupToRaid>());
    assert!(app.is_message_registered::<SetRaidSubgroup>());
    assert!(app.is_message_registered::<SetGroupRole>());
    assert!(app.is_message_registered::<SetGroupLootMethod>());
    assert!(app.is_message_registered::<StartReadyCheck>());
    assert!(app.is_message_registered::<RespondReadyCheck>());
    assert!(app.is_message_registered::<GroupInvitePrompt>());
    assert!(app.is_message_registered::<ReadyCheckUpdate>());
    assert!(app.is_message_registered::<GroupRosterSnapshot>());
    assert!(app.is_message_registered::<GroupCommandResponse>());
}

#[test]
fn group_roster_round_trips_on_the_wire() {
    let roster = GroupRosterSnapshot {
        is_raid: true,
        ready_count: 1,
        total_count: 2,
        members: vec![GroupMemberSnapshot {
            name: "Alice".into(),
            role: GroupRoleSnapshot::Tank,
            is_leader: true,
            online: true,
            subgroup: 3,
            class: 2,
            level: 80,
            entity: Some(42),
        }],
        loot_method: LootMode::RoundRobin,
    };
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&roster, config).unwrap();
    let (decoded, _): (GroupRosterSnapshot, usize) =
        bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(decoded, roster);
}

#[test]
fn group_message_codes_use_retail_global_strings() {
    assert_eq!(
        GroupMessageCode::GroupFull.global_string_key(),
        "ERR_GROUP_FULL"
    );
    assert_eq!(
        GroupMessageCode::GroupFull.format(""),
        "Your party is full."
    );
    assert_eq!(
        GroupMessageCode::DeclineGroup.format("Bob"),
        "Bob declines your group invitation."
    );
    assert_eq!(
        GroupMessageCode::BadPlayerName.format("Nobody"),
        "Cannot find player 'Nobody'."
    );
}
