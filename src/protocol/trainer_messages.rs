//! Trainer messages (Retail `ClassTrainerFrame`, TrinityCore `SMSG_TRAINER_LIST` /
//! `CMSG_TRAINER_BUY_SPELL` / `SMSG_TRAINER_BUY_FAILED`).
//!
//! The server sends `TrainerList` when it opens the trainer role for an NPC and
//! again after every purchase. `TrainerBuySpell` names the trainer NPC by server
//! entity bits and is refused unless its trainer frame is open. Money and learned
//! spells change through `Gold` and `ProfessionSnapshot`; refusals come back as
//! `TrainerBuyFailed`. All messages use `TrainerChannel`.

use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

/// Reliable ordered channel for trainer messages, bidirectional.
pub struct TrainerChannel;

/// TrinityCore `Trainer::SpellState`, shown by the frame's filter as
/// `AVAILABLE`, `UNAVAILABLE` and `USED` ("Already Known").
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrainerServiceState {
    Available,
    Unavailable,
    Known,
}

/// One trainer service (Retail `TrainerListSpell`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TrainerService {
    pub spell_id: u32,
    /// Copper.
    pub cost: u64,
    pub state: TrainerServiceState,
    pub req_level: u8,
    pub req_skill_line: u32,
    pub req_skill_rank: u16,
    /// Spells the player must know; 0 entries omitted.
    pub req_abilities: Vec<u32>,
    /// Learning it adds a primary profession (Retail `GetTrainerServiceCost`
    /// `isProfession`): the frame confirms it and needs a free profession slot.
    pub profession: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TrainerList {
    pub npc: u64,
    pub trainer_id: u32,
    pub greeting: String,
    pub services: Vec<TrainerService>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrainerBuySpell {
    pub npc: u64,
    pub spell_id: u32,
}

/// TrinityCore `Trainer::FailReason`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrainerFailReason {
    /// The service is not available: requirements unmet, already known, or no
    /// free primary profession slot.
    Unavailable,
    NotEnoughMoney,
}

impl TrainerFailReason {
    /// Retail error text. The Train button is disabled for unavailable services
    /// (`Blizzard_TrainerUI.lua:307`), so `Unavailable` has no string of its own.
    pub fn message(self) -> Option<&'static str> {
        match self {
            Self::Unavailable => None,
            Self::NotEnoughMoney => Some("You don't have enough money."), // ERR_NOT_ENOUGH_MONEY
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrainerBuyFailed {
    pub npc: u64,
    pub spell_id: u32,
    pub reason: TrainerFailReason,
}

pub(super) fn register_trainer_protocol(app: &mut App) {
    app.add_channel::<TrainerChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_message::<TrainerList>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<TrainerBuyFailed>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<TrainerBuySpell>()
        .add_direction(NetworkDirection::ClientToServer);
}
