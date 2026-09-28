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
fn mirror_timer_types_are_trinitycore_mirror_timer_type() {
    // Player.h MirrorTimerType: FATIGUE_TIMER 0, BREATH_TIMER 1, FIRE_TIMER 2 (feign death).
    assert_eq!(
        (
            MIRROR_TIMER_FATIGUE,
            MIRROR_TIMER_BREATH,
            MIRROR_TIMER_FEIGN_DEATH
        ),
        (0, 1, 2)
    );
}

#[test]
fn mirror_timer_messages_round_trip_on_the_wire() {
    // Player::HandleDrowning: a fresh breath timer of 3 minutes draining (scale -1), then a
    // surfaced one refilling at 10 ms per ms from 42.5 s.
    let draining = MirrorTimerStart {
        timer: MIRROR_TIMER_BREATH,
        value_ms: 180_000,
        max_value_ms: 180_000,
        scale: -1.0,
        paused: false,
        spell_id: 0,
    };
    assert_wire_round_trip(&draining);
    assert_wire_round_trip(&MirrorTimerStart {
        value_ms: 42_500,
        scale: 10.0,
        ..draining
    });
    assert_wire_round_trip(&MirrorTimerStart {
        timer: MIRROR_TIMER_FATIGUE,
        value_ms: 60_000,
        max_value_ms: 60_000,
        scale: -1.0,
        paused: true,
        spell_id: 0,
    });
    assert_wire_round_trip(&MirrorTimerPause {
        timer: MIRROR_TIMER_BREATH,
        paused: true,
    });
    assert_wire_round_trip(&MirrorTimerStop {
        timer: MIRROR_TIMER_FEIGN_DEATH,
    });
}

#[test]
fn protocol_plugin_registers_mirror_timer_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<MirrorTimerStart>());
    assert!(app.is_message_registered::<MirrorTimerPause>());
    assert!(app.is_message_registered::<MirrorTimerStop>());
}
