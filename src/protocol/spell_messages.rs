//! Spell system messages.
//!
//! Channels: spell runtime (cooldowns, charges, cast failures, combat log, aura cancel) uses
//! `CombatChannel`, ordered with `SpellCastIntent`. Known spells, action bars, traits and
//! specialization use `TalentChannel`.

use serde::{Deserialize, Serialize};

use crate::spell_data::CastFailReason;

/// Full known-spell list for the owning player, sent on enter world.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KnownSpellsSnapshot {
    pub spells: Vec<u32>,
}

/// Spells added to the owning player's known list.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SpellsLearned {
    pub spells: Vec<u32>,
}

/// Spells removed from the owning player's known list.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SpellsUnlearned {
    pub spells: Vec<u32>,
}

/// Cooldown started or changed for a spell (or the GCD when `is_gcd`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SpellCooldownUpdate {
    pub spell_id: u32,
    /// Shared cooldown category; 0 when none.
    pub category: u32,
    pub duration_ms: u32,
    /// Remaining time when the server sent this value.
    pub remaining_ms: u32,
    pub is_gcd: bool,
}

/// Charge state of a charge-based spell.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SpellChargesUpdate {
    pub spell_id: u32,
    pub current: u8,
    pub max: u8,
    /// Time to regain one charge.
    pub recharge_ms: u32,
    /// Remaining time until the next charge when the server sent this value.
    pub remaining_ms: u32,
}

/// Server rejected a cast requested by the owning client.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CastFailed {
    pub spell_id: u32,
    pub reason: CastFailReason,
    /// Optional human-readable detail for error text.
    pub detail: Option<String>,
}

/// A cast resolved (TrinityCore `SMSG_SPELL_GO`): broadcast to every client that
/// replicates the caster, so each one plays the caster's cast animation and the spell's
/// visual kits. `target` is the explicit unit target (`None` for untargeted casts);
/// `hit_targets` are the units the spell's effects landed on (`SpellGo.HitTargets`),
/// caster included for self-buffs, each once.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SpellGo {
    pub caster: u64,
    pub target: Option<u64>,
    pub hit_targets: Vec<u64>,
    pub spell_id: u32,
}

/// A cast or channel ended without resolving (TrinityCore `SMSG_SPELL_FAILURE`, sent by
/// `Spell::SendInterrupted` with `SendMessageToSet`): every client that replicates the
/// caster learns why, so cast bars show "Interrupted" or "Failed". `failed_by` is the
/// unit whose interrupt effect stopped it (`SpellFailure.FailedBy`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SpellFailure {
    pub caster: u64,
    pub spell_id: u32,
    pub reason: CastFailReason,
    pub failed_by: Option<u64>,
}

/// Avoidance/negation outcome of a combat log `Miss`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissKind {
    Miss,
    Dodge,
    Parry,
    Block,
    Resist,
    Immune,
    Evade,
    Absorb,
    Deflect,
    Reflect,
}

/// What a combat log event records.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatLogKind {
    Damage,
    Heal,
    Energize,
    Miss(MissKind),
    AuraApplied,
    AuraRemoved,
    AuraRefreshed,
    Interrupt,
    Dispel,
    CastStart,
    CastSuccess,
    Death,
}

/// One combat log entry; entities are server entity bits.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CombatLogEvent {
    pub source: Option<u64>,
    pub target: Option<u64>,
    pub spell_id: Option<u32>,
    /// Bit set of spell schools (1 physical, 2 holy, 4 fire, 8 nature, 16 frost, 32 shadow, 64 arcane).
    pub school_mask: u32,
    pub amount: i32,
    /// Overkill for damage, overheal for heals.
    pub overflow: i32,
    pub absorbed: i32,
    pub resisted: i32,
    pub blocked: i32,
    pub crit: bool,
    /// A glancing melee swing (`HITINFO_GLANCING`; CLEU SWING_DAMAGE `glancing`): a player's
    /// swing at a creature 4 or more levels above it. Crushing blows are not carried: whether
    /// Retail 12.x still produces them is unverified (TrinityCore master's formula never
    /// fires, Unit.cpp:2475).
    pub glancing: bool,
    pub periodic: bool,
    pub kind: CombatLogKind,
}

/// What an action bar slot triggers.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionRef {
    Spell(u32),
    Item(u32),
    Macro(u32),
}

/// All occupied action bar slots of the owning player.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ActionBarSnapshot {
    pub slots: Vec<(u8, ActionRef)>,
}

/// Client places (`Some`) or clears (`None`) an action bar slot.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SetActionButton {
    pub slot: u8,
    pub action: Option<ActionRef>,
}

/// Client cancels one of its own auras.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CancelAura {
    pub spell_id: u32,
}

/// One chosen trait entry (DB2 `TraitNode`/`TraitNodeEntry` ids).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TraitEntrySelection {
    pub node_id: u32,
    pub entry_id: u32,
    pub rank: u8,
}

/// Active trait configuration of the owning player.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TraitConfigSnapshot {
    pub spec_id: u32,
    pub tree_id: u32,
    pub entries: Vec<TraitEntrySelection>,
    /// Unspent points as (`TraitCurrency` id, amount).
    pub unspent: Vec<(u32, i32)>,
}

/// Client replaces the trait configuration for a specialization.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CommitTraitConfig {
    pub spec_id: u32,
    pub entries: Vec<TraitEntrySelection>,
}

/// Server answer to `CommitTraitConfig`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TraitCommitResult {
    pub ok: bool,
    pub reason: Option<String>,
}

/// Client requests a specialization change (`ChrSpecialization` id).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SetSpecialization {
    pub spec_id: u32,
}

/// Owning player's active specialization changed.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SpecializationChanged {
    pub spec_id: u32,
}
