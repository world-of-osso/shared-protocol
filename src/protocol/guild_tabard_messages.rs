//! Retail GuildTabardVendor operations on ordered-reliable GuildChannel.
use super::ProtocolRegistrationExt;
use crate::guild_creation::GuildTabardDesign;
use bevy::prelude::*;
use lightyear::prelude::NetworkDirection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GuildTabardRequest {
    Open { npc: u64 },
    Save { npc: u64, design: GuildTabardDesign },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildTabardVendorSnapshot {
    pub npc: u64,
    pub npc_name: String,
    pub own_rank: Option<u8>,
    pub design: Option<GuildTabardDesign>,
    pub cost: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GuildTabardReply {
    Vendor(GuildTabardVendorSnapshot),
    Saved { design: GuildTabardDesign },
    Failed { message: String },
}

pub(super) fn register_guild_tabard_protocol(app: &mut App) {
    app.register_protocol_message::<GuildTabardRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GuildTabardReply>()
        .add_direction(NetworkDirection::ServerToClient);
}
