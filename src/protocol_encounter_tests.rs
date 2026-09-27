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
fn encounter_messages_round_trip() {
    // DungeonEncounter 1144 is Hogger (instance_the_stockade.cpp), Normal (1), 5 players.
    assert_wire_round_trip(&EncounterStart {
        encounter_id: 1144,
        difficulty_id: 1,
        group_size: 5,
    });
    assert_wire_round_trip(&EncounterEnd {
        encounter_id: 1144,
        difficulty_id: 1,
        group_size: 5,
        success: true,
    });
    assert_wire_round_trip(&EncounterEngageUnit {
        unit: 0x0000_0001_0000_2a17,
        target_frame_priority: 0,
    });
    assert_wire_round_trip(&EncounterDisengageUnit {
        unit: 0x0000_0001_0000_2a17,
    });
}

#[test]
fn monster_chat_types_round_trip_with_their_speaker() {
    for channel in [
        ChatType::MonsterSay(7),
        ChatType::MonsterYell(7),
        ChatType::MonsterEmote(7),
        ChatType::RaidBossEmote(7),
    ] {
        assert_eq!(channel.monster_speaker(), Some(7));
        assert_wire_round_trip(&channel);
    }
    assert_eq!(ChatType::Yell.monster_speaker(), None);
}

#[test]
fn protocol_plugin_registers_encounter_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<EncounterStart>());
    assert!(app.is_message_registered::<EncounterEnd>());
    assert!(app.is_message_registered::<EncounterEngageUnit>());
    assert!(app.is_message_registered::<EncounterDisengageUnit>());
}
