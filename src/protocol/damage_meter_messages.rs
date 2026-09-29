//! Retail's built-in damage meter (`Blizzard_DamageMeter`, `C_DamageMeter`): the server
//! accumulates the damage done by the recipient and its group and sends each owner its
//! sessions. The Retail UI never reads combat log events; it only asks `C_DamageMeter`
//! for aggregated sessions (`GetCombatSessionFromType`, DamageMeterSessionWindow.lua
//! 561-579), so the aggregation is not client Lua.

use serde::{Deserialize, Serialize};

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

/// `DamageMeterCombatSource`: one player's damage in a session.
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
}

/// `DamageMeterCombatSpell`: one spell's damage by one source.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DamageMeterSpell {
    /// The damaging spell; 0 for melee swings.
    pub spell_id: u32,
    pub total_amount: u64,
    pub amount_per_second: f32,
}
