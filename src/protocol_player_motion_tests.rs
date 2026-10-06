use super::*;
use crate::components::PlayerMotion;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::ComponentRegistry;

/// Retail `MovementFlags` values (TrinityCore `MovementInfo.h`).
#[test]
fn player_motion_bits_are_trinitycore_movement_flags() {
    assert_eq!(PlayerMotion::FORWARD, 0x0000_0001);
    assert_eq!(PlayerMotion::BACKWARD, 0x0000_0002);
    assert_eq!(PlayerMotion::STRAFE_LEFT, 0x0000_0004);
    assert_eq!(PlayerMotion::STRAFE_RIGHT, 0x0000_0008);
    assert_eq!(PlayerMotion::WALKING, 0x0000_0100);
    assert_eq!(PlayerMotion::FALLING, 0x0000_0800);
    assert_eq!(PlayerMotion::JUMP_STARTED, 0x8000_0000_0000_0000);
    assert_eq!(PlayerMotion::SWIMMING, 0x0010_0000);
    assert_eq!(PlayerMotion::CAN_ADV_FLY, 0x0000_2000_0000_0000);
    assert_eq!(PlayerMotion::ADV_FLYING, 0x0000_4000_0000_0000);
    assert_eq!(PlayerMotion::default(), PlayerMotion(0));
}

#[test]
fn player_motion_contains_only_its_set_flags() {
    let strafing_jump = PlayerMotion(PlayerMotion::STRAFE_LEFT | PlayerMotion::FALLING);
    assert!(strafing_jump.contains(PlayerMotion::STRAFE_LEFT));
    assert!(strafing_jump.contains(PlayerMotion::FALLING));
    assert!(!strafing_jump.contains(PlayerMotion::STRAFE_RIGHT));
    assert!(!strafing_jump.contains(PlayerMotion::FORWARD | PlayerMotion::FALLING));
}

/// A backpedalling walker swimming and a strafing jumper survive both component encodings
/// (bitcode for replication, bincode serde for the wire) and replicate.
#[test]
fn player_motion_round_trips_and_replicates() {
    for motion in [
        PlayerMotion(PlayerMotion::BACKWARD | PlayerMotion::WALKING | PlayerMotion::SWIMMING),
        PlayerMotion(PlayerMotion::STRAFE_RIGHT | PlayerMotion::FALLING),
        PlayerMotion(PlayerMotion::FORWARD | PlayerMotion::FALLING | PlayerMotion::JUMP_STARTED),
        PlayerMotion(
            PlayerMotion::FORWARD
                | PlayerMotion::CAN_FLY
                | PlayerMotion::FLYING
                | PlayerMotion::CAN_ADV_FLY
                | PlayerMotion::ADV_FLYING,
        ),
        PlayerMotion::default(),
    ] {
        let decoded: PlayerMotion = bitcode::decode(&bitcode::encode(&motion)).unwrap();
        assert_eq!(decoded, motion);
        let config = bincode::config::standard();
        let bytes = bincode::serde::encode_to_vec(motion, config).unwrap();
        let (decoded, read): (PlayerMotion, usize) =
            bincode::serde::decode_from_slice(&bytes, config).unwrap();
        assert_eq!((decoded, read), (motion, bytes.len()));
    }

    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);
    assert!(
        app.world()
            .resource::<ComponentRegistry>()
            .is_registered::<PlayerMotion>()
    );
}
