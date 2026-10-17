use shared::protocol::{EmoteEvent, EmoteIntent, EmoteKind};

#[test]
fn hug247_round_trips_targeted_intent_and_event() {
    let hug: EmoteKind = serde_json::from_str(r#""Hug""#).unwrap();
    let intent = EmoteIntent {
        emote: hug,
        target_entity: Some(9123),
    };
    let bytes = bincode::serde::encode_to_vec(&intent, bincode::config::standard()).unwrap();
    let (received, _): (EmoteIntent, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(received.emote, hug);
    assert_eq!(received.target_entity, Some(9123));
    let event = EmoteEvent {
        player_entity: 5123,
        sender: "Hugger".into(),
        emote: received.emote,
    };
    let bytes = bincode::serde::encode_to_vec(&event, bincode::config::standard()).unwrap();
    let (observed, _): (EmoteEvent, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(observed.player_entity, 5123);
    assert_eq!(observed.emote, hug);
}
