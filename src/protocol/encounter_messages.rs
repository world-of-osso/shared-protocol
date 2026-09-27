//! Dungeon encounter messages (TrinityCore `InstanceScript::SetBossState`,
//! `InstanceScript::SendEncounterUnit`; Retail `SMSG_ENCOUNTER_START`/`END`).
//!
//! A boss's encounter starts when it engages (`IN_PROGRESS`) and ends when it
//! dies (`DONE`, success) or its players wipe and it resets. While engaged, the
//! boss is a `boss1..5` unit for the client's boss frames (Retail
//! `INSTANCE_ENCOUNTER_ENGAGE_UNIT`).

use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

/// Reliable ordered channel for encounter messages, server to client.
pub struct EncounterChannel;

/// `SMSG_ENCOUNTER_START`: `DungeonEncounter.db2` ID `encounter_id` began.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EncounterStart {
    pub encounter_id: u32,
    pub difficulty_id: u32,
    pub group_size: u32,
}

/// `SMSG_ENCOUNTER_END`: the encounter ended, killed (`success`) or reset.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EncounterEnd {
    pub encounter_id: u32,
    pub difficulty_id: u32,
    pub group_size: u32,
    pub success: bool,
}

/// `SMSG_INSTANCE_ENCOUNTER_ENGAGE_UNIT`: `unit` (entity bits) takes the next
/// free boss frame; lower `target_frame_priority` sorts first.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EncounterEngageUnit {
    pub unit: u64,
    pub target_frame_priority: u8,
}

/// `SMSG_INSTANCE_ENCOUNTER_DISENGAGE_UNIT`: `unit` leaves its boss frame.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EncounterDisengageUnit {
    pub unit: u64,
}

pub(super) fn register_encounter_protocol(app: &mut App) {
    app.add_channel::<EncounterChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<EncounterStart>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<EncounterEnd>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<EncounterEngageUnit>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<EncounterDisengageUnit>()
        .add_direction(NetworkDirection::ServerToClient);
}
