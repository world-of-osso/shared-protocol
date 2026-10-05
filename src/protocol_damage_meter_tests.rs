use super::*;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::AppMessageExt;

fn frostbolt_session(session_id: u32, duration_secs: f32, active: bool) -> DamageMeterSession {
    DamageMeterSession {
        session_id,
        duration_secs,
        active,
        total_amount: 186,
        sources: vec![DamageMeterSource {
            unit: 4_294_967_302,
            name: "Fbmage".into(),
            class_id: 8,
            is_local_player: true,
            total_amount: 186,
            amount_per_second: 186.0 / duration_secs,
            spells: vec![DamageMeterSpell {
                spell_id: 228_597,
                total_amount: 186,
                amount_per_second: 186.0 / duration_secs,
            }],
        }],
    }
}

#[test]
fn damage_meter_snapshot_round_trips_on_the_wire() {
    let snapshot = DamageMeterSnapshot {
        current: Some(frostbolt_session(2, 7.5, true)),
        overall: DamageMeterSession {
            session_id: 0,
            ..frostbolt_session(0, 19.5, false)
        },
    };
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&snapshot, config).unwrap();
    let (decoded, read): (DamageMeterSnapshot, usize) =
        bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(decoded, snapshot);

    let empty = DamageMeterSnapshot {
        current: None,
        overall: DamageMeterSession {
            session_id: 0,
            duration_secs: 0.0,
            active: false,
            total_amount: 0,
            sources: Vec::new(),
        },
    };
    let bytes = bincode::serde::encode_to_vec(&empty, config).unwrap();
    let (decoded, _): (DamageMeterSnapshot, usize) =
        bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(decoded, empty);
}

#[test]
fn threat_update_and_clear_round_trip() {
    let update = ThreatUpdate {
        creature: 42,
        victim: Some(17),
        entries: vec![ThreatUnit {
            unit: 17,
            name: "Tank".into(),
            class_id: 2,
            raw_threat: 143.0,
            status: 3,
            raw_percent: 100.0,
            scaled_percent: 100.0,
        }],
    };
    let clear = ThreatUpdate {
        creature: 42,
        victim: None,
        entries: vec![],
    };
    for message in [update, clear] {
        let config = bincode::config::standard();
        let bytes = bincode::serde::encode_to_vec(&message, config).unwrap();
        let (decoded, read): (ThreatUpdate, usize) =
            bincode::serde::decode_from_slice(&bytes, config).unwrap();
        assert_eq!(decoded, message);
        assert_eq!(read, bytes.len());
    }
}

#[test]
fn protocol_plugin_registers_the_damage_meter_snapshot() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<DamageMeterSnapshot>());
    assert!(app.is_message_registered::<ThreatUpdate>());
}
