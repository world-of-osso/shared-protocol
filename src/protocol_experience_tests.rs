use super::*;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::AppMessageExt;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

fn assert_wire_round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(value, config).unwrap();
    let (decoded, read): (T, usize) = bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(&decoded, value);
}

#[test]
fn experience_messages_round_trip_on_the_wire() {
    // Level 10: 65 kill XP of 10,590 needed.
    assert_wire_round_trip(&PlayerXpUpdate {
        xp: 65,
        next_level_xp: 10_590,
        rested_xp: 0,
    });
    assert_wire_round_trip(&LogXpGain {
        victim: Some(42),
        original: 130,
        amount: 65,
        group_bonus: 1.0,
        reason: XpGainReason::Kill,
    });
    assert_wire_round_trip(&LogXpGain {
        victim: None,
        original: 100,
        amount: 100,
        group_bonus: 1.0,
        reason: XpGainReason::Quest,
    });
}

#[test]
fn protocol_plugin_registers_experience_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<PlayerXpUpdate>());
    assert!(app.is_message_registered::<LogXpGain>());
}
