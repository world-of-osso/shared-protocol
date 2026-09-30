//! Unit tooltip data the client cannot know on its own (game-engine
//! `docs/specs/unit-tooltip.md`), and the account's appearance collection.
//!
//! `CreatureTooltipQuery { entry }` asks for a creature template's tooltip data,
//! like Retail `CMSG_QUERY_CREATURE`; the server answers `CreatureTooltip` with
//! the template's subname and creature type (Retail `SMSG_QUERY_CREATURE_RESPONSE`)
//! plus two sections that deliberately deviate from Retail: the loot table's
//! drops with their chances and the vendor's items. The client caches the answer
//! per entry.
//!
//! `AppearanceCollectionUpdate` is the account-wide `transmog::AppearanceCollection`,
//! sent in full to the owning client when a character enters the world and after
//! every newly learned appearance. All messages use `TooltipChannel`.

use crate::protocol::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

/// Reliable ordered channel for tooltip queries and the appearance collection.
pub struct TooltipChannel;

/// Tooltip data request for one creature template entry (`Npc::template_id`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreatureTooltipQuery {
    pub entry: u32,
}

/// An item listed in a tooltip.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TooltipItem {
    pub item_id: u32,
    pub name: String,
    /// Retail `Enum.ItemQuality`.
    pub quality: u8,
    /// `ItemAppearance` id the item teaches the collection; `None` for items
    /// without an appearance (reagents, consumables, recipes, quest items, rings).
    pub appearance_id: Option<u32>,
}

/// A loot table item with the chance, in percent, that one kill drops it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TooltipDrop {
    pub item: TooltipItem,
    pub chance: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CreatureTooltip {
    pub entry: u32,
    /// `creature_template.subname` ("Weaponsmith"); empty when none.
    pub subname: String,
    /// `creature_template.type` (Retail `CreatureType`: 1 Beast … 7 Humanoid …).
    pub creature_type: u8,
    /// Drops by descending chance; poor-quality items are never listed.
    pub drops: Vec<TooltipDrop>,
    /// Vendor items in vendor slot order.
    pub vendor_items: Vec<TooltipItem>,
}

/// The account's learned `ItemAppearance` ids, sorted.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AppearanceCollectionUpdate {
    pub appearances: Vec<u32>,
}

pub(super) fn register_tooltip_protocol(app: &mut App) {
    app.add_channel::<TooltipChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_protocol_message::<CreatureTooltipQuery>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<CreatureTooltip>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<AppearanceCollectionUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}
