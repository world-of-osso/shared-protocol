//! Authoritative pet battle state and decisions. Slot APIs are one-based like Retail.
use serde::{Deserialize, Serialize};

/// Retail Find Battle and queue-ready popup decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PetBattleQueueRequest {
    Join,
    Leave,
    Accept { proposal_id: u64 },
    Decline { proposal_id: u64 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PetBattleQueueState {
    #[default]
    Idle,
    Queued,
    Proposal {
        proposal_id: u64,
        accepted: bool,
    },
    InBattle,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PetBattleQueueUpdate {
    pub state: PetBattleQueueState,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetBattlePetLoadout {
    pub slots: [Option<u64>; 3],
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartWildPetBattle {
    pub creature: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WildPetBattleAction {
    Ability(u8),
    Swap(u8),
    Pass,
    Forfeit,
    Trap,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WildPetBattleActionRequest {
    pub battle_id: u64,
    pub round: u32,
    pub action: WildPetBattleAction,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleAbilitySnapshot {
    pub id: u32,
    pub name: String,
    pub icon: u32,
    /// Zero-based ability family for the Retail effectiveness badge.
    pub family: u8,
    pub cooldown: u32,
    pub usable: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleAuraSnapshot {
    pub ability_id: u32,
    pub icon: u32,
    pub rounds_remaining: i32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattlePetSnapshot {
    pub instance_id: Option<u64>,
    pub species_id: u32,
    pub name: String,
    pub display_id: u32,
    pub icon: u32,
    pub quality: u8,
    pub xp: u32,
    pub next_level_xp: u32,
    /// Zero-based DB2 family, independent of art skin.
    pub family: u8,
    pub level: u8,
    pub health: i32,
    pub max_health: i32,
    pub power: i32,
    pub speed: i32,
    pub abilities: [BattleAbilitySnapshot; 3],
    pub auras: Vec<BattleAuraSnapshot>,
}
/// Server event amounts, not health deltas: preserves multi-hit, healing and misses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleCombatFeedback {
    pub team: u8,
    pub slot: u8,
    pub text: String,
    pub healing: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WildPetBattleSnapshot {
    pub battle_id: u64,
    pub round: u32,
    pub teams: [Vec<BattlePetSnapshot>; 2],
    /// Zero-based active indices into each team.
    pub active: [u8; 2],
    pub wild_creature: u64,
    pub can_trap: bool,
    /// Zero means untimed, including wild PvE.
    pub turn_time_ms: u32,
    /// PvP pre-battle selection: both owners must choose a living front pet.
    pub initial_selection_required: bool,
    pub replacement_required: bool,
    pub feedback: Vec<BattleCombatFeedback>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WildPetBattleOutcome {
    Won,
    Lost,
    Draw,
    Forfeited,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattlePetXpReward {
    pub instance_id: u64,
    pub xp_gained: u32,
    pub level: u8,
    pub xp: u32,
    pub next_level_xp: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WildPetBattleUpdate {
    Start(WildPetBattleSnapshot),
    State(WildPetBattleSnapshot),
    Round {
        state: WildPetBattleSnapshot,
        combat_text: Vec<String>,
    },
    End {
        battle_id: u64,
        outcome: WildPetBattleOutcome,
        rewards: Vec<BattlePetXpReward>,
        captured_pet_id: Option<u64>,
        combat_text: Vec<String>,
    },
    Rejected(String),
}
