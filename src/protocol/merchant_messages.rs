//! Vendor messages (Retail `MerchantFrame`, AzerothCore `SMSG_LIST_INVENTORY` /
//! `CMSG_BUY_ITEM` / `CMSG_SELL_ITEM` / `CMSG_BUYBACK_ITEM` / `CMSG_REPAIR_ITEM`;
//! TrinityCore `CMSG_SELL_ALL_JUNK_ITEMS`).
//!
//! The server sends `VendorInventory` and `BuybackList` when it opens the vendor
//! role for an NPC, `VendorInventory` again when a limited-stock count changes, and
//! `BuybackList` after every sale and buyback. Requests name the vendor NPC by
//! server entity bits and are refused unless its vendor frame is open. Money, bags
//! and durability change through `Gold`, `InventoryDelta` and
//! `DurabilityStateUpdate`; refusals come back as `MerchantFailed`. All messages use
//! `MerchantChannel`.

use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

use super::ItemLocation;

/// Reliable ordered channel for vendor messages, bidirectional.
pub struct MerchantChannel;

/// Buyback slots per player (AzerothCore `BUYBACK_SLOT_START..BUYBACK_SLOT_END`).
pub const BUYBACK_SLOTS: usize = 12;

/// One vendor slot (Retail `MerchantItemInfo`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct VendorItem {
    /// Vendor slot index, the `slot` of `BuyItem`.
    pub slot: u32,
    pub item_id: u32,
    pub name: String,
    pub quality: u8,
    /// Copper per purchase of `stack_count` items.
    pub price: u32,
    /// Items one purchase yields (`VendorStackCount`).
    pub stack_count: u32,
    /// Largest stack the item forms (`Stackable`).
    pub max_stack: u32,
    /// Remaining limited stock; `None` for unlimited items.
    pub num_available: Option<u32>,
    /// The player meets the item's class and level requirements.
    pub usable: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct VendorInventory {
    pub npc: u64,
    /// The NPC repairs (`UNIT_NPC_FLAG_REPAIR`).
    pub can_repair: bool,
    pub items: Vec<VendorItem>,
}

/// One sold item that can be bought back.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BuybackItem {
    /// Buyback slot, the `slot` of `BuybackItemRequest`.
    pub slot: u8,
    pub item_id: u32,
    pub name: String,
    pub quality: u8,
    pub count: u32,
    /// Copper the sale paid, and the buyback costs.
    pub price: u32,
}

/// The player's filled buyback slots in slot order.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct BuybackList {
    pub items: Vec<BuybackItem>,
}

/// Buy `count` purchases of the item in vendor `slot`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BuyItem {
    pub npc: u64,
    pub slot: u32,
    pub item_id: u32,
    pub count: u32,
    /// Bag slot the purchase goes to (a merchant cursor dropped on it, TrinityCore
    /// `HandleBuyItemOpcode` ContainerGUID + Slot); `None` stores it anywhere.
    pub destination: Option<ItemLocation>,
}

/// Sell `count` of a bag item; 0 sells the whole stack.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SellItem {
    pub npc: u64,
    pub item_guid: u64,
    pub count: u32,
}

/// Sell every poor-quality bag item with a sell price (Retail
/// `C_MerchantFrame.SellAllJunkItems`, TrinityCore `HandleSellAllJunkItems`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SellAllJunkItems {
    pub npc: u64,
}

/// Buy back the item in buyback `slot`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BuybackItemRequest {
    pub npc: u64,
    pub slot: u8,
}

/// Repair one item, or every item when `item_guid` is `None`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RepairItem {
    pub npc: u64,
    pub item_guid: Option<u64>,
}

/// Why the server refused a vendor request; `message` is the Retail UI error text.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MerchantError {
    /// No vendor frame open for that NPC, or the NPC can't do it.
    NotInteracting,
    ItemNotFound,
    SoldOut,
    NotEnoughMoney,
    InventoryFull,
    CantCarryMore,
    NotInterested,
    TooMuchGold,
    /// The buy destination holds another item or a full stack (`EQUIP_ERR_CANT_STACK`).
    CantStack,
}

impl MerchantError {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotInteracting => "You are too far away.", // ERR_VENDOR_TOO_FAR
            Self::ItemNotFound => "The item was not found.", // ERR_ITEM_NOT_FOUND
            Self::SoldOut => "That item is currently sold out.", // ERR_VENDOR_SOLD_OUT
            Self::NotEnoughMoney => "You don't have enough money.", // ERR_NOT_ENOUGH_MONEY
            Self::InventoryFull => "Inventory is full.",     // ERR_INV_FULL
            Self::CantCarryMore => "You can't carry any more of those items.", // ERR_ITEM_MAX_COUNT
            Self::NotInterested => "The merchant doesn't want that item.", // ERR_VENDOR_NOT_INTERESTED
            Self::TooMuchGold => "At gold limit",                          // ERR_TOO_MUCH_GOLD
            Self::CantStack => "This item cannot stack.",                  // ERR_CANT_STACK
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MerchantFailed {
    pub npc: u64,
    pub error: MerchantError,
}

pub(super) fn register_merchant_protocol(app: &mut App) {
    app.add_channel::<MerchantChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_message::<VendorInventory>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<BuybackList>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<MerchantFailed>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<BuyItem>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<SellItem>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<SellAllJunkItems>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<BuybackItemRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<RepairItem>()
        .add_direction(NetworkDirection::ClientToServer);
}
