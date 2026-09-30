//! Far teleports between maps (TrinityCore `Player::TeleportTo` far branch,
//! `WorldSession::HandleMoveWorldportAck`).
//!
//! The server takes the player out of its map and sends `NewWorld`
//! (Retail `SMSG_NEW_WORLD`). The client unloads the old map, shows the loading
//! screen, loads `map_directory` and answers `WorldPortAck`
//! (`CMSG_WORLD_PORT_RESPONSE`); only then does the server add the player to the
//! destination map. A teleport the destination refuses is answered with
//! `TransferAborted` (`SMSG_TRANSFER_ABORTED`) and the player stays where it was.

use crate::protocol::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

/// Reliable ordered channel for map transfers, bidirectional.
pub struct TransferChannel;

/// The player now belongs to map `map_id` at `position` (Bevy space) facing `facing`
/// (radians). `map_directory` is the lowercase `Map.db2` Directory the client loads
/// terrain from (`world/maps/<directory>/<directory>.wdt`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NewWorld {
    pub map_id: u32,
    pub map_directory: String,
    pub position: [f32; 3],
    pub facing: f32,
}

/// The client finished loading the map of the last `NewWorld`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WorldPortAck;

/// Why a teleport to another map was refused (TrinityCore `TransferAbortReason`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TransferAbortReason {
    /// `TRANSFER_ABORT_MAX_PLAYERS`: "Transfer Aborted: instance is full".
    MaxPlayers,
    /// `TRANSFER_ABORT_NOT_FOUND`: "Transfer Aborted: instance not found".
    NotFound,
    /// `TRANSFER_ABORT_MAP_NOT_ALLOWED`: "Map cannot be entered at this time.".
    MapNotAllowed,
    /// `TRANSFER_ABORT_DIFFICULTY` with the requested `Difficulty.db2` ID: the map offers
    /// no difficulty it downscales to.
    Difficulty(u32),
    /// `TRANSFER_ABORT_LOCKED_TO_DIFFERENT_INSTANCE`: the player is saved to another copy.
    LockedToDifferentInstance,
    /// `TRANSFER_ABORT_ALREADY_COMPLETED_ENCOUNTER`.
    AlreadyCompletedEncounter,
    /// `TRANSFER_ABORT_DIFFICULTY` with a `MapDifficultyXConditionId`: the player fails
    /// that `PlayerCondition` (the dungeon's minimum level). Retail's client reads the
    /// failure text from its DB2; the server sends it resolved.
    Condition(String),
}

impl TransferAbortReason {
    /// Retail GlobalStrings text the client shows in UIErrorsFrame, `map_name` for `%s`.
    pub fn text(&self, map_name: &str) -> String {
        match self {
            Self::MaxPlayers => "Transfer Aborted: instance is full".into(),
            Self::NotFound => "Transfer Aborted: instance not found".into(),
            Self::MapNotAllowed => "Map cannot be entered at this time.".into(),
            // TRANSFER_ABORT_DIFFICULTY1..3 by the requested difficulty.
            Self::Difficulty(1) => format!("Normal difficulty mode is not available for {map_name}."),
            Self::Difficulty(2) => format!("Heroic difficulty mode is not available for {map_name}."),
            Self::Difficulty(_) => format!("This difficulty mode is not available for {map_name}."),
            Self::LockedToDifferentInstance => format!("You are already locked to {map_name}."),
            Self::AlreadyCompletedEncounter => "You are ineligible to participate in at least one encounter in this instance because you are already locked to an instance in which it has been defeated.".into(),
            Self::Condition(text) => text.clone(),
        }
    }
}

/// A teleport to `map_id` was refused for `reason`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransferAborted {
    pub map_id: u32,
    pub reason: TransferAbortReason,
}

pub(super) fn register_transfer_protocol(app: &mut App) {
    app.add_channel::<TransferChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_protocol_message::<NewWorld>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<WorldPortAck>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<TransferAborted>()
        .add_direction(NetworkDirection::ServerToClient);
}
