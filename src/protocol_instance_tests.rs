use super::*;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::AppMessageExt;

fn assert_wire_round_trip<T>(value: &T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(value, config).unwrap();
    let (decoded, read): (T, usize) = bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(&decoded, value);
}

#[test]
fn instance_messages_round_trip() {
    // Grim Batol (670) on Heroic (2), General Umbriss (DungeonEncounter 1051, Bit 3) killed.
    assert_wire_round_trip(&SetDungeonDifficulty { difficulty_id: 2 });
    assert_wire_round_trip(&DungeonDifficultySet { difficulty_id: 23 });
    assert_wire_round_trip(&WorldServerInfo {
        map_id: 670,
        difficulty_id: 2,
    });
    assert_wire_round_trip(&InstanceInfo {
        locks: vec![InstanceLockInfo {
            map_id: 670,
            difficulty_id: 2,
            instance_id: 3,
            time_remaining_secs: 51_840,
            completed_mask: 1 << 3,
            locked: true,
            extended: false,
        }],
    });
    assert_wire_round_trip(&InstanceResetFailed {
        map_id: 34,
        reason: InstanceResetFailedReason::PlayersInside,
    });
    assert_wire_round_trip(&SetSavedInstanceExtend {
        map_id: 670,
        difficulty_id: 2,
        extend: true,
    });
    assert_wire_round_trip(&RaidInstanceMessage {
        kind: RaidInstanceMessageType::Expired,
        map_id: 670,
        difficulty_id: 2,
    });
    assert_wire_round_trip(&TransferAborted {
        map_id: 670,
        reason: TransferAbortReason::Difficulty(2),
    });
}

#[test]
fn protocol_plugin_registers_instance_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<SetDungeonDifficulty>());
    assert!(app.is_message_registered::<DungeonDifficultySet>());
    assert!(app.is_message_registered::<WorldServerInfo>());
    assert!(app.is_message_registered::<RequestRaidInfo>());
    assert!(app.is_message_registered::<InstanceInfo>());
    assert!(app.is_message_registered::<InstanceSaveCreated>());
    assert!(app.is_message_registered::<ResetInstances>());
    assert!(app.is_message_registered::<InstanceReset>());
    assert!(app.is_message_registered::<InstanceResetFailed>());
    assert!(app.is_message_registered::<SetSavedInstanceExtend>());
    assert!(app.is_message_registered::<RaidInstanceMessage>());
}
