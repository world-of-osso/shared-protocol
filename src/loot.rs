//! Group loot method. Loot generation lives on the server; the loot window
//! messages are in `protocol::loot_messages`.

use serde::{Deserialize, Serialize};

/// Group loot distribution mode.
///
/// Ref: AzerothCore `LootMethod` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LootMode {
    /// Legacy enum value; not offered by the Mainline client.
    /// Free for All — anyone can loot anything.
    FreeForAll,
    /// Round Robin — common items rotate between group members.
    RoundRobin,
    /// Need Before Greed — rare+ items trigger a roll window.
    NeedBeforeGreed,
    /// Personal Loot — each player gets their own independent roll (retail default).
    PersonalLoot,
    /// Mainline group rolls on items at or above the group's quality threshold.
    GroupLoot,
}

// Migrated from game-server/crates/shared/src/loot.rs; server integration uses
// these shared roll/window primitives rather than a second resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootDrop {
    pub item_id: u32,
    pub count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LootRollChoice {
    Need,
    Greed,
    Pass,
    Transmog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootRollEntry {
    pub player_id: u64,
    pub choice: LootRollChoice,
    pub roll_value: u8,
}

pub fn resolve_need_greed(entries: &[LootRollEntry], preferred_transmog: &[u64]) -> Option<u64> {
    entries
        .iter()
        .filter(|entry| entry.choice != LootRollChoice::Pass)
        .max_by_key(|entry| {
            (
                entry.choice.priority(),
                entry.choice == LootRollChoice::Transmog
                    && preferred_transmog.contains(&entry.player_id),
                entry.roll_value,
                std::cmp::Reverse(entry.player_id),
            )
        })
        .map(|winner| winner.player_id)
}

impl LootRollChoice {
    pub fn priority(self) -> u8 {
        match self {
            Self::Pass => 0,
            Self::Greed => 1,
            Self::Transmog => 2,
            Self::Need => 3,
        }
    }
}

impl LootMode {
    pub fn is_mainline(self) -> bool {
        matches!(self, Self::GroupLoot | Self::PersonalLoot)
    }
}

pub const ROLL_WINDOW_TIMEOUT: f32 = 60.0;

#[derive(Debug, Clone, PartialEq)]
pub struct RollWindow {
    pub item: LootDrop,
    pub pending_players: Vec<u64>,
    pub responses: Vec<LootRollEntry>,
    pub can_need: Vec<u64>,
    pub can_transmog: Vec<u64>,
    /// Warbands: appropriate classes outrank other Transmog collectors.
    /// This server-owned preference is independent of item-specific Need refusal.
    pub preferred_transmog: Vec<u64>,
    pub time_remaining: f32,
}

impl RollWindow {
    pub fn new(
        item: LootDrop,
        group_players: &[u64],
        can_need: &[u64],
        preferred_transmog: &[u64],
    ) -> Self {
        Self {
            item,
            pending_players: group_players.to_vec(),
            responses: Vec::new(),
            can_need: can_need.to_vec(),
            can_transmog: Vec::new(),
            preferred_transmog: preferred_transmog.to_vec(),
            time_remaining: ROLL_WINDOW_TIMEOUT,
        }
    }

    pub fn respond(&mut self, player_id: u64, choice: LootRollChoice, roll_value: u8) -> bool {
        let invalid_need = choice == LootRollChoice::Need && !self.can_need.contains(&player_id);
        let invalid_transmog =
            choice == LootRollChoice::Transmog && !self.can_transmog.contains(&player_id);
        let invalid_value = choice != LootRollChoice::Pass && !(1..=100).contains(&roll_value);
        if invalid_need || invalid_transmog || invalid_value {
            return false;
        }
        let Some(index) = self.pending_players.iter().position(|&p| p == player_id) else {
            return false;
        };
        self.pending_players.swap_remove(index);
        self.responses.push(LootRollEntry {
            player_id,
            choice,
            roll_value: if choice == LootRollChoice::Pass {
                0
            } else {
                roll_value
            },
        });
        true
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if self.pending_players.is_empty() {
            return true;
        }
        self.time_remaining -= dt;
        if self.time_remaining <= 0.0 {
            self.responses.extend(
                self.pending_players
                    .drain(..)
                    .map(|player_id| LootRollEntry {
                        player_id,
                        choice: LootRollChoice::Pass,
                        roll_value: 0,
                    }),
            );
            return true;
        }
        false
    }

    pub fn is_complete(&self) -> bool {
        self.pending_players.is_empty()
    }

    pub fn resolve(&self) -> Option<u64> {
        if self.is_complete() {
            resolve_need_greed(&self.responses, &self.preferred_transmog)
        } else {
            None
        }
    }
}
