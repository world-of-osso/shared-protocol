//! Source conversation actor/line payload; privately routed by the server.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ActorObject {
    WorldObject(u64),
    NoObject { creature_id: u32, display_id: u32 },
    ActivePlayer,
    TalkingHead { creature_id: u32, display_id: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ActorPayload {
    pub id: i32,
    pub index: u8,
    pub name: Option<String>,
    pub object: ActorObject,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BroadcastTextPayload {
    pub male_text: String,
    pub female_text: String,
    pub sound_kit_id: u32,
    pub language_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LinePayload {
    pub id: u32,
    pub broadcast_text_id: u32,
    pub ui_camera_id: u32,
    pub actor_index: u8,
    pub flags: u8,
    pub chat_type: u8,
    pub start_ms: i64,
    pub source_duration_ms: Option<i32>,
    pub additional_duration_ms: i32,
    pub visual_kit_id: u32,
    pub animation_kit_id: u32,
    pub start_animation: u16,
    pub end_animation: u16,
    pub text: Option<BroadcastTextPayload>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ConversationPayload {
    pub id: u64,
    pub entry: u32,
    pub creator: u64,
    pub private_owner: u64,
    pub texture_kit_id: u32,
    pub flags: u8,
    pub actors: Vec<ActorPayload>,
    pub lines: Vec<LinePayload>,
    pub last_line_end_ms: i64,
    pub duration_ms: u64,
    pub elapsed_ms: u64,
}

/// Full authoritative set for the owning player, sent over EncounterChannel.
/// Empty sets remove expired/cancelled conversations; elapsed time supports late join.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ConversationUpdate {
    pub map_id: u32,
    pub instance_id: u32,
    pub conversations: Vec<ConversationPayload>,
}
