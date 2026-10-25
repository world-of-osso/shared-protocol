//! Dungeon encounter messages (TrinityCore `InstanceScript::SetBossState`,
//! `InstanceScript::SendEncounterUnit`; Retail `SMSG_ENCOUNTER_START`/`END`).
//!
//! A boss's encounter starts when it engages (`IN_PROGRESS`) and ends when it
//! dies (`DONE`, success) or its players wipe and it resets. While engaged, the
//! boss is a `boss1..5` unit for the client's boss frames (Retail
//! `INSTANCE_ENCOUNTER_ENGAGE_UNIT`).

#[cfg(feature = "transport")]
use crate::protocol::ProtocolRegistrationExt;
#[cfg(feature = "transport")]
use bevy::prelude::*;
#[cfg(feature = "transport")]
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

/// Retail EncounterEventSeverity. No severity is inferred from spell damage.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncounterSeverity {
    Low,
    Medium,
    High,
}

/// An actual selected scheduler timer, not an estimated ability cooldown.
/// Conditional timers may skip on a chance roll, target or phase condition.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EncounterTimer {
    pub event_id: u32,
    pub spell_id: u32,
    pub remaining_ms: u32,
    pub duration_ms: u32,
    pub conditional: bool,
}

/// Ordered with encounter lifecycle. Snapshot replaces this caster's timers;
/// empty snapshots cancel them. Casts are sent only after server acceptance.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum EncounterTimelineEvent {
    Timers {
        caster: u64,
        /// UTC sample time; receivers subtract delivery age, not restart timers.
        sample_unix_ms: u64,
        timers: Vec<EncounterTimer>,
    },
    Cast {
        caster: u64,
        spell_id: u32,
        severity: EncounterSeverity,
        /// Fully rendered addon enUS text, with pinned SpellName substitutions.
        warning_text: String,
    },
    Phase {
        caster: u64,
        phase: u32,
    },
}

/// Read the same UTC millisecond domain used by authoritative group countdowns.
pub fn read_encounter_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("encounter clock predates Unix epoch")
        .as_millis() as u64
}

#[cfg(feature = "transport")]
pub(super) fn register_encounter_protocol(app: &mut App) {
    app.add_channel::<EncounterChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<EncounterTimelineEvent>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<EncounterStart>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<EncounterEnd>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<EncounterEngageUnit>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<EncounterDisengageUnit>()
        .add_direction(NetworkDirection::ServerToClient);
}
