//! Loot window messages (Retail `LootFrame`, AzerothCore `CMSG_LOOT_UNIT` /
//! `CMSG_LOOT_ITEM` / `CMSG_LOOT_MONEY` / `CMSG_LOOT_RELEASE`).
//!
//! Loot is personal: every qualifying player of a kill has their own slots and
//! coins on the corpse. The server sends `CorpseLootable` to each of them when the
//! corpse gets loot for them and again when their loot is empty or the corpse
//! decays. `LootUnit` opens the window with `LootResponse`; `LootSlotRequest`
//! takes one slot and answers with `LootSlotRemoved`, or `LootFailed` when the item
//! does not fit. The window ends with `LootClosed`. Items and money reach the
//! player through `InventoryDelta` and `Gold`. All messages use `LootChannel`.

#[cfg(feature = "transport")]
use crate::protocol::ProtocolRegistrationExt;
#[cfg(feature = "transport")]
use bevy::prelude::*;
#[cfg(feature = "transport")]
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

/// Reliable ordered channel for loot messages, bidirectional.
pub struct LootChannel;

/// Right-click on a corpse. `auto` takes every slot at once (Retail
/// `autoLootDefault` toggled by the `AUTOLOOTTOGGLE` modifier, Shift).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LootUnit {
    pub corpse: u64,
    pub auto: bool,
}

/// Take one slot of the open loot window (`LootSlot(i)`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LootSlotRequest {
    pub corpse: u64,
    pub slot: u8,
}

/// The player closed the loot window (`CloseLoot()`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LootRelease {
    pub corpse: u64,
}

/// What a loot slot holds (Retail `GetLootSlotType`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum LootContent {
    Money {
        copper: u64,
    },
    Item {
        item_id: u32,
        name: String,
        quality: u8,
        count: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LootSlot {
    pub slot: u8,
    pub content: LootContent,
}

/// The loot window opened (`LOOT_OPENED isAutoLoot`) with the slots still on the
/// corpse, in slot order.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LootResponse {
    pub corpse: u64,
    pub auto: bool,
    pub slots: Vec<LootSlot>,
}

/// A slot was taken (`LOOT_SLOT_CLEARED`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LootSlotRemoved {
    pub corpse: u64,
    pub slot: u8,
}

/// The loot window ended server-side (`LOOT_CLOSED`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LootClosed {
    pub corpse: u64,
}

/// The corpse has loot for this player (sparkle, loot cursor), or no longer has.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CorpseLootable {
    pub corpse: u64,
    pub lootable: bool,
}

/// Why the server refused a loot request; `message` is the Retail UI error text.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LootError {
    TooFar,
    NoPermission,
    PlayerDead,
    InventoryFull,
    CantCarryMore,
    TooMuchGold,
}

impl LootError {
    pub fn message(self) -> &'static str {
        match self {
            Self::TooFar => "You are too far away to loot that corpse.", // ERR_LOOT_TOO_FAR
            Self::NoPermission => "You don't have permission to loot that corpse.", // ERR_LOOT_DIDNT_KILL
            Self::PlayerDead => "You can't do that when you're dead.", // ERR_PLAYER_DEAD
            Self::InventoryFull => "Inventory is full.",               // ERR_INV_FULL
            Self::CantCarryMore => "You can't carry any more of those items.", // ERR_ITEM_MAX_COUNT
            Self::TooMuchGold => "At gold limit",                      // ERR_TOO_MUCH_GOLD
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LootFailed {
    pub corpse: u64,
    pub error: LootError,
}

/// Need/Greed/Transmog/Pass intent; roll values are generated only by the server.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupLootChoice {
    pub roll_id: u64,
    pub choice: crate::loot::LootRollChoice,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupLootHistoryRequest;

/// Eligibility and result for one participant, captured at drop time.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GroupLootPlayer {
    pub character_id: u64,
    pub name: String,
    pub can_need: bool,
    pub need_reason: Option<String>,
    pub can_transmog: bool,
    pub response: Option<crate::loot::LootRollEntry>,
}

/// Authoritative roll/history entry. Sent to eligible members on start, vote and
/// resolution, and replayed after a history request. Complete entries never roll again.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GroupLootRoll {
    pub roll_id: u64,
    pub corpse: u64,
    pub item: LootContent,
    pub seconds_remaining: f32,
    pub players: Vec<GroupLootPlayer>,
    pub winner: Option<u64>,
    pub complete: bool,
    pub delivered: bool,
}

#[cfg(feature = "transport")]
pub(super) fn register_loot_protocol(app: &mut App) {
    app.add_channel::<LootChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_protocol_message::<GroupLootChoice>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GroupLootHistoryRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GroupLootRoll>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<LootUnit>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<LootSlotRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<LootRelease>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<LootResponse>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<LootSlotRemoved>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<LootClosed>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<CorpseLootable>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<LootFailed>()
        .add_direction(NetworkDirection::ServerToClient);
}
