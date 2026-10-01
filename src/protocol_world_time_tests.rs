use super::*;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::AppMessageExt;

#[test]
fn login_set_time_speed_round_trips_and_gives_the_local_second_of_day() {
    // 2026-10-01 15:30:20 local (UTC-5 realm at 20:30:20 UTC).
    let message = LoginSetTimeSpeed {
        game_time_seconds: 1_790_868_620,
        new_speed: GAME_TIME_SPEED,
    };
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(message, config).unwrap();
    let (decoded, read): (LoginSetTimeSpeed, usize) =
        bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!((decoded, read), (message, bytes.len()));
    assert_eq!(message.second_of_day(), 15 * 3600 + 30 * 60 + 20);
    // Player.cpp:24989: TimeSpeed 0.01666667 game minutes per second.
    assert_eq!(GAME_TIME_SPEED, 0.016_666_67);
}

#[test]
fn protocol_plugin_registers_login_set_time_speed() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);
    assert!(app.is_message_registered::<LoginSetTimeSpeed>());
}
