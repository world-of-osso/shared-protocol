//! Group loot method. Loot generation lives on the server; the loot window
//! messages are in `protocol::loot_messages`.

use serde::{Deserialize, Serialize};

/// Group loot distribution mode.
///
/// Ref: AzerothCore `LootMethod` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LootMode {
    /// Free for All — anyone can loot anything.
    FreeForAll,
    /// Round Robin — common items rotate between group members.
    RoundRobin,
    /// Need Before Greed — rare+ items trigger a roll window.
    NeedBeforeGreed,
    /// Personal Loot — each player gets their own independent roll (retail default).
    PersonalLoot,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootRollEntry {
    pub player_id: u64,
    pub choice: LootRollChoice,
    pub roll_value: u8,
}

pub fn resolve_need_greed(entries: &[LootRollEntry]) -> Option<u64> {
    let best_need = entries
        .iter()
        .filter(|e| e.choice == LootRollChoice::Need)
        .max_by_key(|e| e.roll_value);
    if let Some(winner) = best_need {
        return Some(winner.player_id);
    }
    entries
        .iter()
        .filter(|e| e.choice == LootRollChoice::Greed)
        .max_by_key(|e| e.roll_value)
        .map(|winner| winner.player_id)
}

pub const ROLL_WINDOW_TIMEOUT: f32 = 60.0;

#[derive(Debug, Clone, PartialEq)]
pub struct RollWindow {
    pub item: LootDrop,
    pub pending_players: Vec<u64>,
    pub responses: Vec<LootRollEntry>,
    pub time_remaining: f32,
}

impl RollWindow {
    pub fn new(item: LootDrop, group_players: &[u64], can_equip: &[u64]) -> Self {
        let (pending_players, passed): (Vec<_>, Vec<_>) = group_players
            .iter()
            .copied()
            .partition(|player_id| can_equip.contains(player_id));
        let responses = passed
            .into_iter()
            .map(|player_id| LootRollEntry {
                player_id,
                choice: LootRollChoice::Pass,
                roll_value: 0,
            })
            .collect();
        Self {
            item,
            pending_players,
            responses,
            time_remaining: ROLL_WINDOW_TIMEOUT,
        }
    }

    pub fn respond(&mut self, player_id: u64, choice: LootRollChoice, roll_value: u8) -> bool {
        let Some(index) = self.pending_players.iter().position(|&p| p == player_id) else {
            return false;
        };
        self.pending_players.swap_remove(index);
        self.responses.push(LootRollEntry {
            player_id,
            choice,
            roll_value,
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
        resolve_need_greed(&self.responses)
    }
}
