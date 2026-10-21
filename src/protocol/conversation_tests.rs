use super::*;
use bevy::prelude::*;
use bevy_replicon::shared::{
    protocol::ProtocolHasher,
    replication::{registry::ReplicationRegistry, rules::ReplicationRules},
};
use lightyear::prelude::AppMessageExt;

#[test]
fn conversation_wire_native_orator_payload_and_empty_snapshot_round_trip() {
    let value = ConversationUpdate {
        map_id: 2669,
        instance_id: 41,
        conversations: vec![ConversationPayload {
            id: 700,
            entry: 24642,
            creator: 600,
            private_owner: 400,
            texture_kit_id: 0,
            flags: 0,
            actors: vec![ActorPayload {
                id: 51642,
                index: 0,
                name: Some("Orator Krix'vizk".into()),
                object: ActorObject::TalkingHead {
                    creature_id: 220769,
                    display_id: 116692,
                },
            }],
            lines: vec![LinePayload {
                id: 67085,
                broadcast_text_id: 259911,
                ui_camera_id: 1751,
                actor_index: 0,
                flags: 0,
                chat_type: 0,
                start_ms: 0,
                source_duration_ms: None,
                additional_duration_ms: 0,
                visual_kit_id: 0,
                animation_kit_id: 0,
                start_animation: 60,
                end_animation: 60,
                text: Some(BroadcastTextPayload {
                    male_text: "Native source dialogue".into(),
                    female_text: String::new(),
                    sound_kit_id: 250774,
                    language_id: 0,
                }),
            }],
            last_line_end_ms: 0,
            duration_ms: 10000,
            elapsed_ms: 750,
        }],
    };
    for payload in [
        value,
        ConversationUpdate {
            map_id: 2669,
            instance_id: 41,
            conversations: vec![],
        },
    ] {
        let config = bincode::config::standard();
        let bytes = bincode::serde::encode_to_vec(&payload, config).unwrap();
        let (decoded, read): (ConversationUpdate, usize) =
            bincode::serde::decode_from_slice(&bytes, config).unwrap();
        assert_eq!(read, bytes.len());
        assert_eq!(decoded, payload);
    }
    for actor in [
        ActorObject::WorldObject(4300),
        ActorObject::NoObject {
            creature_id: 216619,
            display_id: 116692,
        },
        ActorObject::ActivePlayer,
    ] {
        let bytes = bincode::serde::encode_to_vec(&actor, bincode::config::standard()).unwrap();
        let (decoded, _): (ActorObject, usize) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
        assert_eq!(decoded, actor);
    }
}

#[test]
fn conversation_wire_update_is_registered_by_the_production_protocol() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);
    assert!(app.is_message_registered::<ConversationUpdate>());
}
