//! Retail's built-in damage meter (`Blizzard_DamageMeter`, `C_DamageMeter`): the server
//! accumulates combat category totals for the recipient and its group and sends each owner its
//! sessions. The Retail UI never reads combat log events; it only asks `C_DamageMeter`
//! for aggregated sessions (`GetCombatSessionFromType`, DamageMeterSessionWindow.lua
//! 561-579), so the aggregation is not client Lua.

use serde::{Deserialize, Serialize};

use super::CombatLogEvent;

/// Export limits shared by the server and client. Older recaps are discarded, not totals.
pub const DAMAGE_METER_MAX_SOURCES: usize = 40;
pub const DAMAGE_METER_MAX_SPELLS: usize = 64;
/// Each action category retains 16 cast/affected spell pairs per member.
/// Together with damage and recaps, both sessions fit the 256 KiB payload bound.
pub const DAMAGE_METER_MAX_ACTION_SPELLS: usize = 16;
pub const DAMAGE_METER_MAX_NAME_BYTES: usize = 48;
pub const DAMAGE_METER_MAX_DEATH_RECAPS: usize = 2;
pub const DAMAGE_METER_MAX_RECAP_EVENTS: usize = 8;
/// Conservative upper bound for the standard bincode snapshot payload (before framing).
pub const DAMAGE_METER_MAX_PAYLOAD_BYTES: usize = 256 * 1024;

/// Owner-only snapshot of the two session types a window can show
/// (`Enum.DamageMeterSessionType`: `Overall` 0, `Current` 1).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DamageMeterSnapshot {
    /// `Current`: the active combat, else the last finished one; none before the first.
    pub current: Option<DamageMeterSession>,
    /// `Overall`: every session since the owner entered the world.
    pub overall: DamageMeterSession,
}

/// `DamageMeterCombatSession` plus `GetSessionDurationSeconds`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DamageMeterSession {
    /// `DamageMeterAvailableCombatSession.sessionID`, counting from 1; 0 for overall.
    pub session_id: u32,
    /// Seconds in combat: the session's, or for overall the sum of all sessions'.
    pub duration_secs: f32,
    /// The session's combat is still running.
    pub active: bool,
    pub total_amount: u64,
    /// Highest `total_amount` first.
    pub sources: Vec<DamageMeterSource>,
}

/// `DamageMeterCombatSource`: one player's category totals in a session.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DamageMeterSource {
    /// Server entity bits (`sourceGUID`).
    pub unit: u64,
    pub name: String,
    /// `ChrClasses` id (`classFilename`).
    pub class_id: u8,
    pub is_local_player: bool,
    pub total_amount: u64,
    /// `total_amount` over the session's `duration_secs`.
    pub amount_per_second: f32,
    /// `DamageMeterCombatSessionSource.combatSpells`, highest `total_amount` first.
    pub spells: Vec<DamageMeterSpell>,
    /// Effective healing: logged amount minus overheal, clamped to zero.
    pub healing_done: u64,
    pub overhealing: u64,
    /// Shield points consumed, credited to the shield caster (not damage absorbed by this unit).
    pub absorbs: u64,
    pub interrupts: u64,
    /// Successful interrupts by casting spell and interrupted spell, highest count first.
    pub interrupt_spells: Vec<DamageMeterActionSpell>,
    /// Successfully removed auras, not attempted dispel casts.
    pub dispels: u64,
    /// Successful dispels by casting spell and removed aura, highest count first.
    pub dispel_spells: Vec<DamageMeterActionSpell>,
    pub deaths: u64,
    /// Latest deaths, chronological; bounded independently of the lifetime death count.
    pub death_recaps: Vec<DamageMeterDeathRecap>,
}

/// Counted action spells (`combatSpells`) plus the combat log's affected spell identity.
/// Detail is bounded independently of category totals; absent affected identity stays absent.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DamageMeterActionSpell {
    pub spell_id: u32,
    pub affected_spell_id: Option<u32>,
    pub total_amount: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DamageMeterDeathRecap {
    pub timestamp_unix_ms: u64,
    /// Last events targeting the victim, chronological, excluding the Death line itself.
    pub events: Vec<CombatLogEvent>,
}

/// `DamageMeterCombatSpell`: one spell's damage by one source.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DamageMeterSpell {
    /// The damaging spell; 0 for melee swings.
    pub spell_id: u32,
    pub total_amount: u64,
    pub amount_per_second: f32,
}
