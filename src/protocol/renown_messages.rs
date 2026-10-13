//! Realm-backed Mainline Journeys renown (Retail 12.1.0.69933 Covenant/RenownRewards).
//! Unevaluated unlock conditions are explicit, never encoded as unlocked.
use super::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RenownRequest {
    Query,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RenownReward {
    pub id: u32,
    pub level: u32,
    pub ui_order: u32,
    pub flags: u32,
    pub icon: u32,
    pub name: String,
    pub description: String,
    pub toast: String,
    pub item_id: u32,
    pub spell_id: u32,
    pub mount_id: u32,
    pub transmog_id: u32,
    pub transmog_set_id: u32,
    pub transmog_illusion_id: u32,
    pub title_id: u32,
    pub quest_id: u32,
    pub player_condition_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RenownUnlock {
    /// The faction has no PlayerCondition gate.
    Ungated,
    /// The realm cannot evaluate this PlayerCondition; `description` is its authored text.
    Unsupported {
        condition_id: u32,
        description: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MajorFactionSnapshot {
    /// Covenant ID, the key of RenownRewards.
    pub id: u32,
    pub faction_id: u32,
    pub name: String,
    pub description: String,
    pub expansion: u8,
    pub max_level: u32,
    pub level_threshold: u32,
    pub level: u32,
    /// Reputation toward the next level, below `level_threshold`.
    pub standing: u32,
    pub unlock: RenownUnlock,
    pub rewards: Vec<RenownReward>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RenownSnapshot {
    pub factions: Vec<MajorFactionSnapshot>,
    /// Missing source catalog, not fabricated empty content.
    pub source_error: Option<String>,
}

pub(super) fn register(app: &mut App) {
    app.register_protocol_message::<RenownRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<RenownSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
}
