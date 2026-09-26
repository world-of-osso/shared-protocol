//! Owner-only bag and equipment messages (Retail inventory semantics).
//!
//! The server sends `InventorySnapshot` + `EquipmentSnapshot` on enter world and an
//! `InventoryDelta` for every later change. Client requests address items by
//! `ItemLocation`; rejected requests answer with `InventoryError`. All messages use
//! `InventoryChannel`.

use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

use crate::components::EquipmentVisualSlot;

/// Reliable ordered channel for inventory and equipment operations, bidirectional.
pub struct InventoryChannel;

/// Backpack container index (Retail `BACKPACK_CONTAINER`).
pub const BACKPACK_BAG: u8 = 0;
/// Containers addressed by `ItemLocation::Bag`: backpack plus four bag slots.
pub const BAG_COUNT: u8 = 5;
/// Backpack capacity (Retail `BACKPACK_SLOTS`).
pub const BACKPACK_SLOTS: u8 = 16;

/// Retail character equipment slots, in `INVSLOT_*` order (1-19).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EquipmentSlot {
    Head,
    Neck,
    Shoulder,
    Shirt,
    Chest,
    Waist,
    Legs,
    Feet,
    Wrist,
    Hands,
    Finger1,
    Finger2,
    Trinket1,
    Trinket2,
    Back,
    MainHand,
    OffHand,
    Ranged,
    Tabard,
}

impl EquipmentSlot {
    pub const ALL: [Self; 19] = [
        Self::Head,
        Self::Neck,
        Self::Shoulder,
        Self::Shirt,
        Self::Chest,
        Self::Waist,
        Self::Legs,
        Self::Feet,
        Self::Wrist,
        Self::Hands,
        Self::Finger1,
        Self::Finger2,
        Self::Trinket1,
        Self::Trinket2,
        Self::Back,
        Self::MainHand,
        Self::OffHand,
        Self::Ranged,
        Self::Tabard,
    ];

    /// Retail `INVSLOT_*` id (Head = 1 … Tabard = 19).
    pub fn inv_slot_id(self) -> u8 {
        self as u8 + 1
    }

    /// Slots an item of Retail `Enum.InventoryType` may occupy, preferred first.
    /// Empty for non-equippable types (0 = non-equip, 18 bag, 24 ammo, 27 quiver).
    /// Ranged weapons (15 bow, 25 thrown, 26 gun/wand) use the main hand since MoP.
    pub fn for_inventory_type(inventory_type: u8) -> &'static [Self] {
        use EquipmentSlot::*;
        match inventory_type {
            1 => &[Head],
            2 => &[Neck],
            3 => &[Shoulder],
            4 => &[Shirt],
            5 | 20 => &[Chest],
            6 => &[Waist],
            7 => &[Legs],
            8 => &[Feet],
            9 => &[Wrist],
            10 => &[Hands],
            11 => &[Finger1, Finger2],
            12 => &[Trinket1, Trinket2],
            13 => &[MainHand, OffHand],
            14 | 22 | 23 => &[OffHand],
            15 | 17 | 21 | 25 | 26 => &[MainHand],
            16 => &[Back],
            19 => &[Tabard],
            28 => &[Ranged],
            _ => &[],
        }
    }

    /// Visual slot shown on the character model; `None` for jewelry, trinkets and relics.
    pub fn visual_slot(self) -> Option<EquipmentVisualSlot> {
        use EquipmentVisualSlot as V;
        Some(match self {
            Self::Head => V::Head,
            Self::Shoulder => V::Shoulder,
            Self::Shirt => V::Shirt,
            Self::Chest => V::Chest,
            Self::Waist => V::Waist,
            Self::Legs => V::Legs,
            Self::Feet => V::Feet,
            Self::Wrist => V::Wrist,
            Self::Hands => V::Hands,
            Self::Back => V::Back,
            Self::MainHand => V::MainHand,
            Self::OffHand => V::OffHand,
            Self::Tabard => V::Tabard,
            Self::Neck
            | Self::Finger1
            | Self::Finger2
            | Self::Trinket1
            | Self::Trinket2
            | Self::Ranged => return None,
        })
    }

    pub fn from_visual_slot(slot: EquipmentVisualSlot) -> Self {
        use EquipmentVisualSlot as V;
        match slot {
            V::Head => Self::Head,
            V::Shoulder => Self::Shoulder,
            V::Back => Self::Back,
            V::Chest => Self::Chest,
            V::Shirt => Self::Shirt,
            V::Tabard => Self::Tabard,
            V::Wrist => Self::Wrist,
            V::Hands => Self::Hands,
            V::Waist => Self::Waist,
            V::Legs => Self::Legs,
            V::Feet => Self::Feet,
            V::MainHand => Self::MainHand,
            V::OffHand => Self::OffHand,
        }
    }
}

/// Where an item sits: a container slot or an equipment slot.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ItemLocation {
    /// `bag` 0 is the backpack, 1-4 the equipped bags; `slot` is 0-based.
    Bag {
        bag: u8,
        slot: u8,
    },
    Equipment(EquipmentSlot),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemDurability {
    pub current: u32,
    pub max: u32,
}

/// One item instance (stack) as the owner sees it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ItemStack {
    pub item_guid: u64,
    pub item_id: u32,
    pub count: u32,
    /// `None` when the item has no durability or the server does not track it.
    pub durability: Option<ItemDurability>,
    pub soulbound: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BagSlotItem {
    pub slot: u8,
    pub item: ItemStack,
}

/// One container: capacity plus its occupied slots.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BagContents {
    pub bag: u8,
    pub size: u8,
    pub items: Vec<BagSlotItem>,
}

/// Full bag contents (containers 0-4), owner only.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InventorySnapshot {
    pub bags: Vec<BagContents>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EquippedItem {
    pub slot: EquipmentSlot,
    pub item: ItemStack,
}

/// Occupied equipment slots, owner only. Absent slots are empty.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EquipmentSnapshot {
    pub items: Vec<EquippedItem>,
}

/// New content of one location; `None` means the location is now empty.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InventorySlotChange {
    pub location: ItemLocation,
    pub item: Option<ItemStack>,
}

/// Bag and equipment slots that changed since the last snapshot or delta.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InventoryDelta {
    pub changes: Vec<InventorySlotChange>,
}

/// Move the item at `from` to `to`; equips, unequips or swaps depending on the slots.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SwapItem {
    pub from: ItemLocation,
    pub to: ItemLocation,
}

/// Move `count` items from the stack at `from` into the empty location `to`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SplitItem {
    pub from: ItemLocation,
    pub to: ItemLocation,
    pub count: u32,
}

/// Destroy `count` items at `location`; 0 or at least the stack size destroys the stack.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DestroyItem {
    pub location: ItemLocation,
    pub count: u32,
}

/// Use the item at `location` (its on-use spell) on an optional target entity.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UseItem {
    pub location: ItemLocation,
    pub target: Option<u64>,
}

/// Equip the item at `from` into the first slot its inventory type allows.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EquipItem {
    pub from: ItemLocation,
}

/// Request server-side bag sorting.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SortBags;

/// Retail `EQUIP_ERR_*` results the server reports.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryErrorReason {
    /// `EQUIP_ERR_CANT_EQUIP_LEVEL_I` with the item's required level.
    CantEquipLevelI { level: u16 },
    /// `EQUIP_ERR_WRONG_SLOT`
    WrongSlot,
    /// `EQUIP_ERR_BAG_FULL`
    BagFull,
    /// `EQUIP_ERR_INV_FULL`
    InvFull,
    /// `EQUIP_ERR_CANT_STACK`
    CantStack,
    /// `EQUIP_ERR_CANT_DUAL_WIELD`
    CantDualWield,
    /// `EQUIP_ERR_NOT_EQUIPPABLE`
    NotEquippable,
    /// `EQUIP_ERR_SLOT_EMPTY`
    SlotEmpty,
    /// `EQUIP_ERR_ITEM_NOT_FOUND`
    ItemNotFound,
    /// `EQUIP_ERR_TOO_FEW_TO_SPLIT`
    TooFewToSplit,
    /// `EQUIP_ERR_SPLIT_FAILED`
    SplitFailed,
    /// `EQUIP_ERR_CANT_DO_RIGHT_NOW`
    CantDoRightNow,
    /// `EQUIP_ERR_INTERNAL_BAG_ERROR`
    InternalBagError,
    /// `EQUIP_ERR_CANT_EQUIP_EVER` (class or race restriction)
    CantEquipEver,
}

impl InventoryErrorReason {
    /// Retail UI error text (`ERR_*` GlobalStrings; TrinityCore ItemDefines.h comments).
    pub fn message(self) -> String {
        match self {
            Self::CantEquipLevelI { level } => {
                format!("You must reach level {level} to use that item.")
            }
            Self::WrongSlot => "That item does not go in that slot.".into(),
            Self::BagFull => "That bag is full.".into(),
            Self::InvFull => "Inventory is full.".into(),
            Self::CantStack => "This item cannot stack.".into(),
            Self::CantDualWield => "You cannot dual-wield".into(), // ERR_2HSKILLNOTFOUND
            Self::NotEquippable => "This item cannot be equipped.".into(),
            Self::SlotEmpty => "That slot is empty.".into(),
            Self::ItemNotFound => "The item was not found.".into(),
            Self::TooFewToSplit => "Tried to split more than number in stack.".into(),
            Self::SplitFailed => "Couldn't split those items.".into(),
            Self::CantDoRightNow => "You can't do that right now.".into(), // ERR_CANT_DO_THAT_RIGHT_NOW
            Self::InternalBagError => "Internal Bag Error".into(),
            Self::CantEquipEver => "You can never use that item.".into(),
        }
    }
}

/// Server rejected an inventory request from the owning client.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InventoryError {
    pub reason: InventoryErrorReason,
    /// Optional detail (e.g. which server data is missing).
    pub detail: Option<String>,
}

pub(super) fn register_inventory_protocol(app: &mut App) {
    app.add_channel::<InventoryChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_message::<InventorySnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<EquipmentSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<InventoryDelta>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<InventoryError>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<SwapItem>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<SplitItem>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<DestroyItem>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<UseItem>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<EquipItem>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<SortBags>()
        .add_direction(NetworkDirection::ClientToServer);
}
