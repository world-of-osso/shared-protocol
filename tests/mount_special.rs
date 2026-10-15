use shared::components::Mounted;
use shared::protocol::{EmoteEvent, EmoteIntent, EmoteKind};

#[test]
fn mount_special_round_trips_through_the_existing_emote_wire() {
    // RED on the old protocol: unknown variant, before touching production code.
    let special: EmoteKind = serde_json::from_str(r#""MountSpecial""#).unwrap();
    let intent = EmoteIntent {
        emote: special,
        target_entity: None,
    };
    let bytes = bincode::serde::encode_to_vec(&intent, bincode::config::standard()).unwrap();
    let (received, _): (EmoteIntent, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(received.emote, special);
    let event = EmoteEvent {
        player_entity: 9123,
        sender: "Rider".into(),
        emote: received.emote,
    };
    let bytes = bincode::serde::encode_to_vec(&event, bincode::config::standard()).unwrap();
    let (observed, _): (EmoteEvent, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(observed.player_entity, 9123);
    assert_eq!(observed.emote, special);
}

#[test]
fn mounted_identity_survives_replication_without_guessing_from_display() {
    let source = serde_json::json!({
        "mount_id": 1797,
        "mount_display_id": 112432,
        "vehicle_id": 0,
        "seats": [null, null, null, null, null, null, null, null]
    });
    let mounted: Mounted = serde_json::from_value(source.clone()).unwrap();
    let bytes = bincode::serde::encode_to_vec(&mounted, bincode::config::standard()).unwrap();
    let (observed, _): (Mounted, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(serde_json::to_value(observed).unwrap(), source);
}
