use crate::components::{CharacterAppearance, EquipmentVisualSlot};
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

// -- Quest snapshots --

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuestLogSnapshot {
    pub entries: Vec<QuestEntrySnapshot>,
    pub watched_quest_ids: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuestEntrySnapshot {
    pub quest_id: u32,
    pub title: String,
    pub zone: String,
    pub completed: bool,
    pub repeatability: QuestRepeatability,
    pub objectives: Vec<QuestObjectiveSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuestObjectiveSnapshot {
    pub text: String,
    pub current: u32,
    pub required: u32,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum QuestRepeatability {
    Normal,
    Daily,
    Weekly,
}

// -- Group snapshots --

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GroupRosterSnapshot {
    pub is_raid: bool,
    pub ready_count: u16,
    pub total_count: u16,
    pub members: Vec<GroupMemberSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GroupMemberSnapshot {
    pub name: String,
    pub role: GroupRoleSnapshot,
    pub is_leader: bool,
    pub online: bool,
    pub subgroup: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GroupRoleSnapshot {
    Tank,
    Healer,
    Damage,
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GroupCommandResponse {
    pub message: String,
}

// -- Combat log snapshots --

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CombatLogSnapshot {
    pub entries: Vec<CombatLogEntrySnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CombatLogEntrySnapshot {
    pub kind: CombatLogEventKindSnapshot,
    pub source: String,
    pub target: String,
    pub spell: Option<String>,
    pub amount: Option<i32>,
    pub aura: Option<String>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CombatLogEventKindSnapshot {
    Damage,
    Heal,
    Interrupt,
    AuraApplied,
    Death,
}

// -- Collection snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CollectionSnapshot {
    pub mounts: Vec<CollectionMountSnapshot>,
    pub pets: Vec<CollectionPetSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CollectionMountSnapshot {
    pub mount_id: u32,
    pub name: String,
    pub known: bool,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CollectionPetSnapshot {
    pub pet_id: u32,
    pub name: String,
    pub known: bool,
    pub active: bool,
}

// -- Profession snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionSnapshot {
    pub skills: Vec<ProfessionSkillSnapshot>,
    pub recipes: Vec<ProfessionRecipeSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionSkillSnapshot {
    pub profession: String,
    pub current: u16,
    pub max: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionRecipeSnapshot {
    pub spell_id: u32,
    pub profession: String,
    pub name: String,
    pub craftable: bool,
    pub cooldown: Option<String>,
}

// -- Currency snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CurrencySnapshot {
    pub entries: Vec<CurrencyEntrySnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CurrencyEntrySnapshot {
    pub id: u32,
    pub name: String,
    pub amount: u64,
}

// -- Reputation snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ReputationSnapshot {
    pub entries: Vec<ReputationEntrySnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ReputationEntrySnapshot {
    pub faction_id: u32,
    pub faction_name: String,
    pub standing: String,
    pub value: i32,
}

// -- Achievement snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AchievementSnapshot {
    pub earned_ids: Vec<u32>,
    pub progress: Vec<AchievementProgressSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AchievementProgressSnapshot {
    pub achievement_id: u32,
    pub current: u32,
    pub required: u32,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AchievementToastSnapshot {
    pub achievement_id: u32,
    pub name: String,
    pub points: u32,
}

// -- World map snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WorldMapSnapshot {
    pub discovered_zone_ids: Vec<u32>,
}

// -- Resting snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RestAreaKindSnapshot {
    City,
    Inn,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RestSnapshot {
    pub in_rest_area: bool,
    pub rest_area_kind: Option<RestAreaKindSnapshot>,
    pub rested_xp: u32,
    pub rested_xp_max: u32,
}

// -- Friends snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FriendCharacterSnapshot {
    pub name: String,
    pub level: u16,
    pub class_name: String,
    pub area: String,
    pub presence: crate::components::PresenceStatus,
    pub note: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FriendsSnapshot {
    pub entries: Vec<FriendCharacterSnapshot>,
}

// -- Who snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WhoCharacterSnapshot {
    pub name: String,
    pub level: u16,
    pub class_name: String,
    pub area: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WhoSnapshot {
    pub query: String,
    pub entries: Vec<WhoCharacterSnapshot>,
}

// -- Calendar snapshots --

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalendarSignupStatusSnapshot {
    Confirmed,
    Tentative,
    Declined,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CalendarSignupSnapshot {
    pub character_name: String,
    pub status: CalendarSignupStatusSnapshot,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CalendarEventSnapshot {
    pub event_id: u64,
    pub title: String,
    pub organizer_name: String,
    pub starts_at_unix_secs: u64,
    pub max_signups: u8,
    pub is_raid: bool,
    pub signups: Vec<CalendarSignupSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CalendarSnapshot {
    pub events: Vec<CalendarEventSnapshot>,
}

// -- Ignore snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IgnoreListSnapshot {
    pub names: Vec<String>,
}

// -- LFG snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LfgRoleCheckSnapshot {
    pub dungeon_id: u32,
    pub dungeon_name: String,
    pub assigned_role: GroupRoleSnapshot,
    pub accepted_count: u8,
    pub total_count: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LfgMatchMemberSnapshot {
    pub name: String,
    pub role: GroupRoleSnapshot,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LfgMatchFoundSnapshot {
    pub dungeon_id: u32,
    pub dungeon_name: String,
    pub assigned_role: GroupRoleSnapshot,
    pub members: Vec<LfgMatchMemberSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LfgSnapshot {
    pub queued: bool,
    pub selected_role: Option<GroupRoleSnapshot>,
    pub dungeon_ids: Vec<u32>,
    pub queue_size: u16,
    pub average_wait_secs: u32,
    pub in_demand_roles: Vec<GroupRoleSnapshot>,
    pub role_check: Option<LfgRoleCheckSnapshot>,
    pub match_found: Option<LfgMatchFoundSnapshot>,
}

// -- PVP snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PvpBracketSnapshot {
    Arena2v2,
    Arena3v3,
    RatedBattleground,
    SoloShuffle,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PvpBracketStatsSnapshot {
    pub bracket: PvpBracketSnapshot,
    pub rating: u32,
    pub season_wins: u32,
    pub season_losses: u32,
    pub weekly_wins: u32,
    pub weekly_losses: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PvpQueueKindSnapshot {
    Battleground { battleground_id: u32, name: String },
    RatedBracket { bracket: PvpBracketSnapshot },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PvpQueueSnapshot {
    pub kind: PvpQueueKindSnapshot,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PvpSnapshot {
    pub honor: u32,
    pub honor_max: u32,
    pub conquest: u32,
    pub conquest_max: u32,
    pub brackets: Vec<PvpBracketStatsSnapshot>,
    pub queue: Option<PvpQueueSnapshot>,
}

// -- Barber shop snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BarberShopSnapshot {
    pub appearance: CharacterAppearance,
    pub gold: u32,
}

// -- Death snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DeathStateSnapshot {
    Alive,
    Dead,
    Ghost,
    Resurrecting,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DeathPositionSnapshot {
    pub map_id: u16,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DeathSnapshot {
    pub state: DeathStateSnapshot,
    pub corpse: Option<DeathPositionSnapshot>,
    pub graveyard: Option<DeathPositionSnapshot>,
    pub can_resurrect_at_corpse: bool,
    pub spirit_healer_available: bool,
}

// -- Durability snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DurabilitySlotSnapshot {
    pub slot: EquipmentVisualSlot,
    pub current: u32,
    pub max: u32,
    pub repair_cost: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DurabilitySnapshot {
    pub total_repair_cost: u32,
    pub slots: Vec<DurabilitySlotSnapshot>,
}

// -- Storage snapshots (guild vault, warbank) --

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageItemSnapshot {
    pub slot: u32,
    pub item_guid: u64,
    pub item_id: u32,
    pub name: String,
    pub stack_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuildVaultSnapshot {
    pub entries: Vec<StorageItemSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WarbankSnapshot {
    pub entries: Vec<StorageItemSnapshot>,
}

// -- Inventory search snapshot --

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InventorySearchResultSnapshot {
    pub entries: Vec<InventorySearchItemSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InventorySearchItemSnapshot {
    pub storage: String,
    pub slot: u32,
    pub item_guid: u64,
    pub item_id: u32,
    pub name: String,
    pub stack_count: u32,
}

pub fn register_snapshot_messages(app: &mut App) {
    app.register_message::<QuestLogSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<GroupRosterSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<GroupCommandResponse>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<CombatLogSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<CollectionSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<ProfessionSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<CurrencySnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<ReputationSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<AchievementSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<AchievementToastSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<WorldMapSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<FriendsSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<IgnoreListSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<LfgSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<PvpSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<BarberShopSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<DeathSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<DurabilitySnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<GuildVaultSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<WarbankSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<InventorySearchResultSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
}
