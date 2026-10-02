//! Owner-only quest giver and quest log messages (Retail `QuestHandler` semantics).
//!
//! Quest giver status depends on the viewing player, so it is not replicated: the
//! client sends `QuestGiverStatusQuery` for the NPCs it sees and the server answers
//! with `QuestGiverStatusMultiple`. After any change to that player's quests the
//! server re-sends the status of every NPC the connection has queried.
//!
//! Enter world sends `QuestLogSnapshot` (with objective POIs); later changes send
//! `QuestLogUpdate`. Talking to a giver: `QuestGiverHello` → `QuestGiverQuestList`,
//! `QuestGiverQueryQuest` → `QuestGiverQuestDetails`, `QuestGiverAcceptQuest`,
//! `QuestGiverCompleteQuest` → `QuestGiverRequestItems` (objectives incomplete) or
//! `QuestGiverOfferReward`, then `QuestGiverChooseReward` → `QuestGiverQuestComplete`.
//! Rejected requests answer with `QuestFailed`. NPCs are addressed by server entity
//! bits. All messages use `QuestChannel`.

use crate::protocol::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

use super::QuestEntrySnapshot;

/// Reliable ordered channel for quest giver and quest log messages, bidirectional.
pub struct QuestChannel;

/// Retail maximum number of quests in the log.
pub const MAX_QUEST_LOG_SIZE: usize = 35;

/// The kind of quest a giver marker stands for, lowest to highest priority within a
/// state (TrinityCore a352b1fa `QuestGiverStatus` bit order, QuestDef.h:153). Retail
/// draws each kind with its own `interface/buttons/talktome*` model.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuestMarkerClass {
    Normal,
    /// Daily or weekly (`Quest::IsDailyOrWeekly`).
    Repeatable,
    /// `QuestInfo.Modifiers & 0x800` (`Quest::IsMeta`).
    Meta,
    /// Covenant calling (`QuestInfo.Type`).
    Calling,
    /// A visible campaign quest line (`QuestMgr::IsCampaignQuestStatusVisibleForPlayer`).
    Campaign,
    /// `QUEST_FLAGS_EX_LEGENDARY`.
    Legendary,
    /// `QuestInfo.Modifiers & 0x400` (`Quest::IsImportant`).
    Important,
}

/// Quest marker over an NPC for one player, lowest to highest priority
/// (TrinityCore `QuestGiverStatus` bit order; the highest status among the NPC's
/// quests wins).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuestGiverStatus {
    None,
    /// A quest the player will be able to take within a few levels (`Future*`).
    Future(QuestMarkerClass),
    /// Available, but far below the player's level (`Trivial*`).
    Trivial(QuestMarkerClass),
    /// A quest in the log that ends here is not complete yet (`Reward`, `*Reward`).
    Incomplete(QuestMarkerClass),
    /// A quest the player can take (`Quest`, `*Quest`).
    Available(QuestMarkerClass),
    /// A completed quest can be turned in here (`*RewardComplete*`).
    Reward(QuestMarkerClass),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuestGiverStatusEntry {
    pub npc: u64,
    pub status: QuestGiverStatus,
}

/// Ask for the quest marker of each listed NPC.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestGiverStatusQuery {
    pub npcs: Vec<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestGiverStatusMultiple {
    pub statuses: Vec<QuestGiverStatusEntry>,
}

/// Open the quest list of a quest giver.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestGiverHello {
    pub npc: u64,
}

/// How a quest appears in a giver's list.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestGiverQuestState {
    Available,
    LowLevelAvailable,
    /// In the log, ends here, objectives not done.
    Incomplete,
    /// In the log, ends here, ready to turn in.
    Complete,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestGiverQuestEntry {
    pub quest_id: u32,
    pub title: String,
    /// Quest level; -1 scales to the player's level.
    pub level: i32,
    pub state: QuestGiverQuestState,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestGiverQuestList {
    pub npc: u64,
    pub quests: Vec<QuestGiverQuestEntry>,
}

/// Ask for the details (accept dialog) of a quest this giver offers.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestGiverQueryQuest {
    pub npc: u64,
    pub quest_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestRewardItem {
    pub item_id: u32,
    pub name: String,
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct QuestRewards {
    /// Copper paid on completion.
    pub money: u32,
    /// Always granted.
    pub items: Vec<QuestRewardItem>,
    /// The player picks exactly one when the list is non-empty.
    pub choice_items: Vec<QuestRewardItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestGiverQuestDetails {
    pub npc: u64,
    pub quest_id: u32,
    pub title: String,
    /// Story text (`QuestDescription`).
    pub description: String,
    /// Objective summary (`LogDescription`).
    pub objectives_text: String,
    pub level: i32,
    pub min_level: u8,
    pub suggested_group: u8,
    pub objectives: Vec<super::QuestObjectiveSnapshot>,
    pub rewards: QuestRewards,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestGiverAcceptQuest {
    pub npc: u64,
    pub quest_id: u32,
}

/// Start turning in a quest at its ender.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestGiverCompleteQuest {
    pub npc: u64,
    pub quest_id: u32,
}

/// Progress dialog for a quest whose objectives are not done.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestGiverRequestItems {
    pub npc: u64,
    pub quest_id: u32,
    pub title: String,
    pub completion_text: String,
    pub required_items: Vec<QuestRewardItem>,
    pub can_complete: bool,
}

/// Reward dialog for a completed quest.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestGiverOfferReward {
    pub npc: u64,
    pub quest_id: u32,
    pub title: String,
    pub reward_text: String,
    pub rewards: QuestRewards,
}

/// Turn in a completed quest; `choice_index` indexes `QuestRewards::choice_items`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestGiverChooseReward {
    pub npc: u64,
    pub quest_id: u32,
    pub choice_index: Option<u8>,
}

/// A quest was turned in; lists what was granted.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestGiverQuestComplete {
    pub quest_id: u32,
    pub money: u32,
    pub items: Vec<QuestRewardItem>,
    /// Experience granted (0 at the level cap).
    pub xp: u32,
}

/// Remove a quest from the log.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AbandonQuest {
    pub quest_id: u32,
}

/// Track (`watched`) or untrack a quest in the objective tracker.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SetQuestWatched {
    pub quest_id: u32,
    pub watched: bool,
}

/// Quest log change after the enter-world `QuestLogSnapshot`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct QuestLogUpdate {
    /// Added or changed entries, full contents.
    pub changed: Vec<QuestEntrySnapshot>,
    /// Quest ids that left the log (turned in or abandoned).
    pub removed: Vec<u32>,
    /// Full watch list after the change.
    pub watched_quest_ids: Vec<u32>,
}

/// Why a quest request was rejected (Retail `QuestFailedReasons` and quest
/// giver errors).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestFailedReason {
    /// Unknown quest id, or the NPC does not start/end it.
    NotAvailable,
    /// NPC missing or out of interaction range.
    TooFar,
    LowLevel,
    HighLevel,
    WrongClass,
    WrongRace,
    /// Prerequisite quest missing, or an exclusive alternative was taken.
    DontHaveRequirement,
    AlreadyOn,
    AlreadyDone,
    QuestLogFull,
    NotInLog,
    ObjectivesIncomplete,
    NotEnoughMoney,
    InventoryFull,
    InvalidRewardChoice,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuestFailed {
    pub quest_id: u32,
    pub reason: QuestFailedReason,
}

/// What an objective counts.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum QuestObjectiveKind {
    /// Creature kills (`object_id` = creature template).
    #[default]
    Monster,
    /// Game object use (`object_id` = game object template).
    GameObject,
    /// Items held in bags (`object_id` = item id).
    Item,
}

/// One quest POI blob: an objective area on a map.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestPoiSnapshot {
    /// Objective index; -1 marks the turn-in location.
    pub objective_index: i32,
    pub map_id: u32,
    /// Wrath `WorldMapArea` id from the imported data (not a Retail `UiMap` id).
    pub world_map_area_id: u32,
    pub floor: u32,
    pub priority: u32,
    pub flags: u32,
    /// Polygon outline in world coordinates (yards).
    pub points: Vec<QuestPoiPoint>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuestPoiPoint {
    pub x: i32,
    pub y: i32,
}

pub(super) fn register_quest_protocol(app: &mut App) {
    app.add_channel::<QuestChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    register_client_messages(app);
    register_server_messages(app);
}

fn register_client_messages(app: &mut App) {
    app.register_protocol_message::<QuestGiverStatusQuery>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<QuestGiverHello>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<QuestGiverQueryQuest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<QuestGiverAcceptQuest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<QuestGiverCompleteQuest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<QuestGiverChooseReward>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<AbandonQuest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<SetQuestWatched>()
        .add_direction(NetworkDirection::ClientToServer);
}

fn register_server_messages(app: &mut App) {
    app.register_protocol_message::<QuestGiverStatusMultiple>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<QuestGiverQuestList>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<QuestGiverQuestDetails>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<QuestGiverRequestItems>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<QuestGiverOfferReward>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<QuestGiverQuestComplete>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<QuestLogUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<QuestFailed>()
        .add_direction(NetworkDirection::ServerToClient);
}
