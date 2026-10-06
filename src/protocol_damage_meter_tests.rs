use super::*;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::AppMessageExt;

fn recap_event() -> CombatLogEvent {
    CombatLogEvent {
        source: Some(u64::MAX),
        target: Some(u64::MAX),
        spell_id: Some(u32::MAX),
        school_mask: u32::MAX,
        amount: i32::MAX,
        overflow: i32::MAX,
        absorbed: i32::MAX,
        resisted: i32::MAX,
        blocked: i32::MAX,
        crit: true,
        glancing: true,
        periodic: true,
        extra_spell_id: Some(u32::MAX),
        timestamp_unix_ms: u64::MAX,
        kind: CombatLogKind::Environmental(EnvironmentalKind::Falling),
    }
}

#[test]
fn damage_meter_maximum_snapshot_fits_payload_bound() {
    let mut session = frostbolt_session(u32::MAX, f32::MAX, true);
    let mut source = session.sources[0].clone();
    source.unit = u64::MAX;
    source.name = "x".repeat(DAMAGE_METER_MAX_NAME_BYTES);
    source.total_amount = u64::MAX;
    source.healing_done = u64::MAX;
    source.overhealing = u64::MAX;
    source.absorbs = u64::MAX;
    source.interrupts = u64::MAX;
    source.dispels = u64::MAX;
    source.deaths = u64::MAX;
    source.spells = vec![
        DamageMeterSpell {
            spell_id: u32::MAX,
            total_amount: u64::MAX,
            amount_per_second: f32::MAX,
        };
        DAMAGE_METER_MAX_SPELLS
    ];
    source.death_recaps = vec![
        DamageMeterDeathRecap {
            timestamp_unix_ms: u64::MAX,
            events: vec![recap_event(); DAMAGE_METER_MAX_RECAP_EVENTS],
        };
        DAMAGE_METER_MAX_DEATH_RECAPS
    ];
    session.sources = vec![source; DAMAGE_METER_MAX_SOURCES];
    session.total_amount = u64::MAX;
    let snapshot = DamageMeterSnapshot {
        current: Some(session.clone()),
        overall: session,
    };
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&snapshot, config).unwrap();
    assert!(
        bytes.len() <= DAMAGE_METER_MAX_PAYLOAD_BYTES,
        "{} bytes",
        bytes.len()
    );
    let (decoded, read): (DamageMeterSnapshot, usize) =
        bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(decoded, snapshot);
}

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
            healing_done: 100,
            overhealing: 40,
            absorbs: 25,
            interrupts: 2,
            dispels: 3,
            deaths: 1,
            death_recaps: vec![DamageMeterDeathRecap {
                timestamp_unix_ms: 1234,
                events: vec![recap_event()],
            }],
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
