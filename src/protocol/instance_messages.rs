//! Dungeon difficulty and instance locks (TrinityCore `WorldSession::HandleSetDungeonDifficultyOpcode`,
//! `Player::SendDungeonDifficulty`, `Player::SendRaidInfo`, `InstanceMap::UpdateInstanceLock`,
//! `Player::ResetInstances`; Retail `SMSG_SET_DUNGEON_DIFFICULTY`, `SMSG_INSTANCE_INFO`,
//! `SMSG_INSTANCE_SAVE_CREATED`, `SMSG_INSTANCE_RESET`, `SMSG_RAID_INSTANCE_MESSAGE`).
//!
//! A player (or its group leader) picks the dungeon difficulty (`Difficulty.db2` ID:
//! 1 Normal, 2 Heroic, 23 Mythic) outside instances. A dungeon copy is created at that
//! difficulty, downscaled to one the map offers (`MapDifficulty.db2`). Killing a boss in a
//! difficulty with a reset schedule saves the copy's players to it until the daily or
//! weekly reset; the saved instances are listed by `InstanceInfo`.

use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

/// Reliable ordered channel for difficulty and instance lock messages, bidirectional.
pub struct InstanceChannel;

/// `CMSG_SET_DUNGEON_DIFFICULTY`: the player (a group's leader for its group) wants
/// dungeons at `difficulty_id`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetDungeonDifficulty {
    pub difficulty_id: u32,
}

/// `SMSG_SET_DUNGEON_DIFFICULTY`: the player's dungeon difficulty, at login and on every
/// change ("Dungeon Difficulty set to %s.").
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DungeonDifficultySet {
    pub difficulty_id: u32,
}

/// `SMSG_WORLD_SERVER_INFO` `DifficultyID`: the difficulty of the map copy the player was
/// just added to (0 on a continent); the client's `GetInstanceInfo`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldServerInfo {
    pub map_id: u32,
    pub difficulty_id: u32,
}

/// `CMSG_REQUEST_RAID_INFO`: send the saved instances (`RequestRaidInfo()`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestRaidInfo;

/// One saved instance (`WorldPackets::Instance::InstanceLock`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceLockInfo {
    pub map_id: u32,
    pub difficulty_id: u32,
    pub instance_id: u32,
    /// Seconds until the (extended) lock expires; 0 once expired.
    pub time_remaining_secs: u32,
    /// `DungeonEncounter.Bit` mask of the bosses killed.
    pub completed_mask: u32,
    /// Not expired.
    pub locked: bool,
    pub extended: bool,
}

/// `SMSG_INSTANCE_INFO`: every lock of the player, current and expired.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InstanceInfo {
    pub locks: Vec<InstanceLockInfo>,
}

/// `SMSG_INSTANCE_SAVE_CREATED`: "You are now saved to this instance".
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceSaveCreated;

/// `CMSG_RESET_INSTANCES`: reset the player's (the leader's group's) unlocked copies.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResetInstances;

/// `SMSG_INSTANCE_RESET`: "%s has been reset.".
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceReset {
    pub map_id: u32,
}

/// `SMSG_INSTANCE_RESET_FAILED` reasons (`ResetFailedReason`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstanceResetFailedReason {
    /// `INSTANCE_RESET_FAILED`: players are still inside.
    PlayersInside,
    /// `INSTANCE_RESET_FAILED_OFFLINE`.
    PlayersOffline,
    /// `INSTANCE_RESET_FAILED_ZONING`.
    PlayersZoning,
}

/// `SMSG_INSTANCE_RESET_FAILED`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceResetFailed {
    pub map_id: u32,
    pub reason: InstanceResetFailedReason,
}

/// `CMSG_SET_SAVED_INSTANCE_EXTEND`: extend (or stop extending) the lock on
/// (`map_id`, `difficulty_id`) by one reset period (`SetSavedInstanceExtend`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetSavedInstanceExtend {
    pub map_id: u32,
    pub difficulty_id: u32,
    pub extend: bool,
}

/// `SMSG_RAID_INSTANCE_MESSAGE` types the server sends (`RaidInstanceResetWarningType`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaidInstanceMessageType {
    /// `RAID_INSTANCE_EXPIRED`: "Your instance lock for %s has expired.".
    Expired,
}

/// `SMSG_RAID_INSTANCE_MESSAGE`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaidInstanceMessage {
    pub kind: RaidInstanceMessageType,
    pub map_id: u32,
    pub difficulty_id: u32,
}

pub(super) fn register_instance_protocol(app: &mut App) {
    app.add_channel::<InstanceChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_message::<SetDungeonDifficulty>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<RequestRaidInfo>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<ResetInstances>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<SetSavedInstanceExtend>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<DungeonDifficultySet>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<WorldServerInfo>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<InstanceInfo>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<InstanceSaveCreated>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<InstanceReset>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<InstanceResetFailed>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<RaidInstanceMessage>()
        .add_direction(NetworkDirection::ServerToClient);
}
