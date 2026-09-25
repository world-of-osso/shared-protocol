//! Character and Warband bank messages (Retail `BankFrame`, `C_Bank`).
//!
//! Retail combines two banks in one frame opened at a banker NPC: the per-character
//! bank (`Enum.BankType.Character`) and the account-wide Warband bank
//! (`Enum.BankType.Account`). Each bank is a list of purchased tabs of
//! `BANK_TAB_SLOTS` slots. When the banker role opens (`InteractionOpened` with
//! `NpcRole::Banker`) the server sends `BankContents` for both banks, and again for
//! a bank after every change. Requests name the banker by server entity bits and
//! are refused unless its bank frame is open. Bags and money change through
//! `InventoryDelta` and `Gold`; refusals come back as `BankFailed`. All messages use
//! `BankChannel`.

use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

use super::ItemStack;

/// Reliable ordered channel for bank messages, bidirectional.
pub struct BankChannel;

/// Slots per bank tab (Retail `C_Container.GetContainerNumSlots` of a bank tab).
pub const BANK_TAB_SLOTS: usize = 98;

/// Retail `Enum.BankType` of the banks a banker opens.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BankType {
    Character,
    Account,
}

/// Retail `Enum.BagSlotFlags` a tab's deposit settings use.
pub mod bank_tab_flags {
    pub const DISABLE_AUTO_SORT: u32 = 0x1;
    pub const EQUIPMENT: u32 = 0x2;
    pub const CONSUMABLES: u32 = 0x4;
    pub const PROFESSION_GOODS: u32 = 0x8;
    pub const JUNK: u32 = 0x10;
    pub const REAGENTS: u32 = 0x80;
    pub const EXPANSION_CURRENT: u32 = 0x100;
    pub const EXPANSION_LEGACY: u32 = 0x200;
}

/// One purchased tab (Retail `BankTabData` plus its container slots).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BankTabView {
    pub name: String,
    /// Icon FileDataID.
    pub icon: u32,
    /// `bank_tab_flags` bits.
    pub deposit_flags: u32,
    /// `BANK_TAB_SLOTS` entries in slot order.
    pub slots: Vec<Option<ItemStack>>,
}

/// Full contents of one bank.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BankContents {
    pub bank: BankType,
    pub tabs: Vec<BankTabView>,
    /// Copper price of the next tab; `None` once every tab is bought.
    pub next_tab_cost: Option<u64>,
    /// Deposited copper; `Some` only for banks that hold money (the Warband bank).
    pub money: Option<u64>,
}

/// Move a bag stack into the bank: stacks of the same item in `tab` are topped up,
/// then the first empty slot of `tab`, then of the other tabs in order.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BankDeposit {
    pub npc: u64,
    pub bank: BankType,
    pub tab: u8,
    pub item_guid: u64,
}

/// Move the stack in a bank slot into the bags.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BankWithdraw {
    pub npc: u64,
    pub bank: BankType,
    pub tab: u8,
    pub slot: u8,
}

/// Buy the next tab (`C_Bank.PurchaseBankTab`); the player pays.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BankPurchaseTab {
    pub npc: u64,
    pub bank: BankType,
}

/// `C_Bank.DepositMoney` / `C_Bank.WithdrawMoney`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BankMoneyTransfer {
    pub npc: u64,
    pub bank: BankType,
    pub copper: u64,
    pub deposit: bool,
}

/// `C_Bank.AutoDepositItemsIntoBank`: Character deposits every reagent; Account
/// deposits every Warbound item, plus tradeable reagents when `include_reagents`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BankAutoDeposit {
    pub npc: u64,
    pub bank: BankType,
    pub include_reagents: bool,
}

/// `C_Bank.UpdateBankTabSettings`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BankUpdateTabSettings {
    pub npc: u64,
    pub bank: BankType,
    pub tab: u8,
    pub name: String,
    pub icon: u32,
    pub deposit_flags: u32,
}

/// Why the server refused a bank request; `message` is the Retail UI error text.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BankError {
    /// No bank frame open for that NPC.
    NotInteracting,
    ItemNotFound,
    BankFull,
    InventoryFull,
    TabLocked,
    MaxTabs,
    CantAfford,
    NotEnoughMoney,
    TooMuchGold,
    SoulboundInAccountBank,
    /// The bank type holds no money (Retail `DoesBankTypeSupportMoneyTransfer`).
    NoMoneyTransfer,
    InvalidTabName,
}

impl BankError {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotInteracting => "You are too far away from a bank.", // ERR_NO_BANK_HERE
            Self::ItemNotFound => "The item was not found.",             // ERR_ITEM_NOT_FOUND
            Self::BankFull => "Your bank is full",                       // ERR_BANK_FULL
            Self::InventoryFull => "Inventory is full.",                 // ERR_INV_FULL
            Self::TabLocked => "The bank tab is locked.",                // BANK_TAB_NOT_UNLOCKED
            Self::MaxTabs => "You've reached your limit of bag slots!", // ERR_BANKSLOT_FAILED_TOO_MANY
            Self::CantAfford => "You can't afford that.", // ERR_BANKSLOT_INSUFFICIENT_FUNDS
            Self::NotEnoughMoney => "You don't have enough money.", // ERR_NOT_ENOUGH_MONEY
            Self::TooMuchGold => "At gold limit",         // ERR_TOO_MUCH_GOLD
            Self::SoulboundInAccountBank => {
                "Soulbound items cannot be stored in the Warband Bank." // ERR_NO_SOULBOUND_ITEM_IN_ACCOUNT_BANK
            }
            Self::NoMoneyTransfer => "This bank is currently unavailable.", // BANK_LOCKED_REASON_BANK_DISABLED
            Self::InvalidTabName => "Bank tab name is invalid.",            // BANK_TAB_INVALID_NAME
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BankFailed {
    pub npc: u64,
    pub error: BankError,
}

pub(super) fn register_bank_protocol(app: &mut App) {
    app.add_channel::<BankChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_message::<BankContents>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<BankFailed>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<BankDeposit>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<BankWithdraw>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<BankPurchaseTab>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<BankMoneyTransfer>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<BankAutoDeposit>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<BankUpdateTabSettings>()
        .add_direction(NetworkDirection::ClientToServer);
}
