//! Guild bank messages (Retail `Blizzard_GuildBankUI`, AzerothCore
//! `WorldSession::HandleGuildBank*`).
//!
//! The guild bank opens at a Guild Vault game object (`UseGameObject`, answered
//! with `InteractionOpened` / `NpcRole::GuildBanker`). The server then sends
//! `GuildBankContents`, and again to every member with the vault open after any
//! change. Tabs the player's rank can't view are sent without slots. Logs are sent
//! on `GuildBankQueryLog`. Requests name the vault by server entity bits and are
//! refused unless its guild bank frame is open; refusals come back as
//! `GuildBankFailed`. All messages use `GuildBankChannel`.

use crate::protocol::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

use super::ItemStack;

/// Reliable ordered channel for guild bank messages, bidirectional.
pub struct GuildBankChannel;

/// Retail `MAX_GUILDBANK_SLOTS_PER_TAB`.
pub const GUILD_BANK_TAB_SLOTS: usize = 98;
/// Retail `MAX_GUILDBANK_TABS`.
pub const GUILD_BANK_MAX_TABS: usize = 8;

/// One purchased tab as the player's rank sees it (Retail `GetGuildBankTabInfo`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildBankTabView {
    pub name: String,
    /// Icon FileDataID.
    pub icon: u32,
    pub viewable: bool,
    pub can_deposit: bool,
    /// Stacks per day the rank may withdraw; `None` is unlimited.
    pub withdrawals_per_day: Option<u32>,
    /// Stacks left today; `None` is unlimited.
    pub remaining_withdrawals: Option<u32>,
    /// Tab info text (Retail `GetGuildBankText`).
    pub text: String,
    /// `GUILD_BANK_TAB_SLOTS` entries in slot order; empty when not viewable.
    pub slots: Vec<Option<ItemStack>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildBankContents {
    pub object: u64,
    pub guild_name: String,
    pub tabs: Vec<GuildBankTabView>,
    /// Guild money in copper (`GetGuildBankMoney`).
    pub money: u64,
    /// Copper the player may still withdraw today; `None` is unlimited
    /// (`GetGuildBankWithdrawMoney` -1).
    pub withdraw_money_remaining: Option<u64>,
    /// Copper price of the next tab, when one is purchasable.
    pub next_tab_cost: Option<u64>,
    /// Guild Master (`IsGuildLeader`): may buy tabs and edit tab info.
    pub is_leader: bool,
}

/// Deposit a bag stack into `tab`: same-item stacks topped up first, then the
/// first empty slot.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildBankDeposit {
    pub object: u64,
    pub tab: u8,
    pub item_guid: u64,
}

/// Withdraw the stack in a tab slot into the bags.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildBankWithdraw {
    pub object: u64,
    pub tab: u8,
    pub slot: u8,
}

/// `DepositGuildBankMoney` / `WithdrawGuildBankMoney`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildBankMoneyTransfer {
    pub object: u64,
    pub copper: u64,
    pub deposit: bool,
}

/// `BuyGuildBankTab`: the Guild Master pays.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildBankBuyTab {
    pub object: u64,
}

/// `SetGuildBankTabInfo` (Guild Master).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildBankSetTabInfo {
    pub object: u64,
    pub tab: u8,
    pub name: String,
    pub icon: u32,
}

/// `SetGuildBankText` (Guild Master).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildBankSetTabText {
    pub object: u64,
    pub tab: u8,
    pub text: String,
}

/// `QueryGuildBankLog`: `tab` `None` is the money log.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildBankQueryLog {
    pub object: u64,
    pub tab: Option<u8>,
}

/// Retail `GetGuildBankTransaction` / `GetGuildBankMoneyTransaction` types.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuildBankLogKind {
    DepositItem,
    WithdrawItem,
    DepositMoney,
    WithdrawMoney,
    BuyTab,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildBankLogEntry {
    pub kind: GuildBankLogKind,
    pub actor: String,
    pub item_id: u32,
    pub item_name: String,
    pub count: u32,
    pub copper: u64,
    /// Seconds between the transaction and the server sending the log.
    pub seconds_ago: u64,
}

/// One log, oldest entry first.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildBankLog {
    pub tab: Option<u8>,
    pub entries: Vec<GuildBankLogEntry>,
}

/// Why the server refused a guild bank request; `message` is the Retail UI text.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuildBankError {
    NotInteracting,
    NotInGuild,
    Permissions,
    WrongTab,
    TabFull,
    BoundItem,
    QuestItem,
    ItemNotFound,
    InventoryFull,
    WithdrawLimit,
    NotEnoughGuildMoney,
    NotEnoughMoney,
    TooMuchMoney,
}

impl GuildBankError {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotInteracting => "You are too far away.", // ERR_USE_TOO_FAR
            Self::NotInGuild => "You are not in a guild.",   // ERR_GUILD_PLAYER_NOT_IN_GUILD
            Self::Permissions => "You don't have permission to do that.", // ERR_GUILD_PERMISSIONS
            Self::WrongTab => "Incorrect bank tab",          // ERR_GUILD_BANK_WRONG_TAB
            Self::TabFull => "This guild bank tab is full",  // ERR_GUILD_BANK_FULL
            Self::BoundItem => "You cannot store soulbound items in the guild bank", // ERR_GUILD_BANK_BOUND_ITEM
            Self::QuestItem => "You cannot store quest items in the guild bank", // ERR_GUILD_BANK_QUEST_ITEM
            Self::ItemNotFound => "The item was not found.", // ERR_ITEM_NOT_FOUND
            Self::InventoryFull => "Inventory is full.",     // ERR_INV_FULL
            Self::WithdrawLimit => "You cannot withdraw that much from the guild bank.", // ERR_GUILD_WITHDRAW_LIMIT
            Self::NotEnoughGuildMoney => "The guild bank does not have enough money", // ERR_GUILD_NOT_ENOUGH_MONEY
            Self::NotEnoughMoney => "You don't have enough money.", // ERR_NOT_ENOUGH_MONEY
            Self::TooMuchMoney => "The guild bank is at gold limit", // ERR_GUILD_TOO_MUCH_MONEY
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuildBankFailed {
    pub object: u64,
    pub error: GuildBankError,
}

pub(super) fn register_guild_bank_protocol(app: &mut App) {
    app.add_channel::<GuildBankChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_protocol_message::<GuildBankContents>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<GuildBankLog>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<GuildBankFailed>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<GuildBankDeposit>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GuildBankWithdraw>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GuildBankMoneyTransfer>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GuildBankBuyTab>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GuildBankSetTabInfo>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GuildBankSetTabText>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GuildBankQueryLog>()
        .add_direction(NetworkDirection::ClientToServer);
}
