use crate::components::{CharacterAppearance, EquipmentVisualSlot};
use crate::protocol::{ArenaQueueMode, ProtocolRegistrationExt};
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

// -- Quest snapshots --

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct QuestLogSnapshot {
    pub entries: Vec<QuestEntrySnapshot>,
    pub watched_quest_ids: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestEntrySnapshot {
    pub quest_id: u32,
    pub title: String,
    pub zone: String,
    /// Objectives done; the quest can be turned in.
    pub completed: bool,
    /// Absolute server Unix seconds; timer continues while disconnected.
    pub expires_at: Option<i64>,
    /// Server clock when this timer was sampled; clients calibrate countdowns on receipt.
    pub timer_sampled_at: Option<i64>,
    pub failed: bool,
    pub repeatability: QuestRepeatability,
    pub objectives: Vec<QuestObjectiveSnapshot>,
    /// Quest level; -1 scales to the player's level.
    pub level: i32,
    /// `QuestSortID`: positive = `AreaTable` zone id (the log's zone header),
    /// negative = `QuestSort` id (class, profession, holiday). `zone` is left empty
    /// because the server has no area names; clients resolve this id.
    pub sort_id: i32,
    /// Objective summary (`LogDescription`).
    pub objectives_text: String,
    /// Authoritative owned story text, available without talking to a giver this session.
    pub description: String,
    pub rewards: crate::protocol::QuestRewards,
    /// Server quest sharing eligibility before the client's current group check.
    pub pushable: bool,
    /// Tracker text once objectives are done (`QuestCompletionLog`).
    pub completion_text: String,
    /// Shown in the objective tracker.
    pub watched: bool,
    pub pois: Vec<crate::protocol::QuestPoiSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuestObjectiveSnapshot {
    pub text: String,
    pub current: u32,
    pub required: u32,
    pub completed: bool,
    /// Displayed but not required for quest completion.
    pub optional: bool,
    pub kind: crate::protocol::QuestObjectiveKind,
    /// Creature, game object or item id the objective counts.
    pub object_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum QuestRepeatability {
    Normal,
    Daily,
    Weekly,
}

// -- Group snapshots --

/// Full roster, sent to every member on each change. An empty `members` list means
/// the recipient is no longer in a group.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GroupRosterSnapshot {
    pub is_raid: bool,
    pub ready_count: u16,
    pub total_count: u16,
    pub members: Vec<GroupMemberSnapshot>,
    pub loot_method: crate::loot::LootMode,
    pub loot_threshold: u8,
}

/// Canonical head customization and only the equipped head visual, independent of
/// entity replication. Stable customization IDs cannot be classified by the server.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct GroupPortraitAppearance {
    pub race: u8,
    pub appearance: crate::components::CharacterAppearance,
    pub head: Option<crate::components::EquippedAppearanceEntry>,
}

#[cfg(test)]
#[test]
#[ignore = "party follower portrait RED; enable after headless GREEN"]
fn party_follower_portrait_preserves_creature_display_on_the_roster_wire() {
    let mut received = serde_json::to_value(GroupPortraitAppearance::default()).unwrap();
    received["creature_display_id"] = serde_json::json!(113753);
    let portrait: GroupPortraitAppearance = serde_json::from_value(received).unwrap();
    let roundtrip = serde_json::to_value(portrait).unwrap();
    assert_eq!(roundtrip["creature_display_id"], 113753);
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GroupMemberSnapshot {
    /// Stable identity for tap eligibility, including offline/out-of-interest members.
    pub character_id: u64,
    pub name: String,
    pub role: GroupRoleSnapshot,
    pub is_leader: bool,
    pub online: bool,
    /// Raid subgroup 1–8; always 1 in a party.
    pub subgroup: u8,
    pub class: u8,
    pub level: u8,
    /// Player entity bits while the member is in the world.
    pub entity: Option<u64>,
    /// Last known head appearance, retained while offline or outside interest.
    pub portrait: GroupPortraitAppearance,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupRoleSnapshot {
    Tank,
    Healer,
    Damage,
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GroupCommandResponse {
    pub message: String,
    pub code: crate::protocol::GroupMessageCode,
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

/// The owner's professions: learned skill lines and every spell learned through
/// them (profession spells such as 3908 Tailoring, and recipes). Sent on enter
/// world and after every change. Recipe data (reagents, category, trivial ranks,
/// output) comes from the client's DB2 tables, as in Retail.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct ProfessionSnapshot {
    pub lines: Vec<crate::profession::ProfessionSkillLine>,
    pub spells: Vec<u32>,
    /// Static tdb_trainer_spell rows for the owner's learned skill lines; not availability.
    pub recipe_sources: Vec<ProfessionTrainerSource>,
    /// Character-owned favorite recipe spell IDs; no account-wide or client-only preference.
    pub favorites: Vec<u32>,
}

/// An authored trainer row. Previewing it never teaches a recipe or authorizes payment.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionTrainerSource {
    pub trainer_id: u32,
    pub spell_id: u32,
    pub cost: u64,
    pub req_skill_line: u32,
    pub req_skill_rank: u16,
    pub req_abilities: Vec<u32>,
    pub req_level: u8,
}

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ReputationSnapshot {
    pub entries: Vec<ReputationEntrySnapshot>,
    /// Owner-only status bar state; None removes a cleared watch, not an omitted update.
    pub watched: Option<WatchedReputationSnapshot>,
}

/// ReputationBar.lua's normalized values, independent of the Journeys reward catalog.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WatchedReputationSnapshot {
    pub faction_id: u32,
    pub name: String,
    pub value: u32,
    pub maximum: u32,
    pub capped: bool,
    pub is_major_faction: bool,
    pub reaction: crate::reputation::Standing,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ReputationEntrySnapshot {
    pub faction_id: u32,
    pub faction_name: String,
    pub standing: String,
    pub value: i32,
}

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
    /// Authored Achievement.IconFileID for the earned alert.
    pub icon_fdid: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WorldMapSnapshot {
    pub discovered_zone_ids: Vec<u32>,
}

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildMemberSnapshot {
    pub character_name: String,
    pub level: u16,
    pub class_name: String,
    pub rank_name: String,
    pub is_online: bool,
    pub officer_note: String,
    pub public_note: String,
    /// Computed for each authenticated recipient, never supplied by a client.
    pub is_self: bool,
    /// Actually observed logout, for offline members only; unknown history stays absent.
    pub last_logout_unix_secs: Option<u64>,
    /// Server-resolved top-level zone label for an online member; unavailable/offline is None.
    pub online_zone: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildSnapshot {
    pub guild_id: u32,
    pub guild_name: String,
    /// Registered guild tabard design; legacy/unregistered guilds are None.
    pub tabard: Option<crate::guild_creation::GuildTabardDesign>,
    /// Actual creation time, Unix seconds; legacy guilds with no recorded date stay None.
    pub created_at: Option<u64>,
    pub motd: String,
    pub info_text: String,
    pub members: Vec<GuildMemberSnapshot>,
}

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IgnoreListSnapshot {
    pub names: Vec<String>,
}

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
    /// No rating is claimed without an authoritative played-match result.
    pub rating: Option<u32>,
    pub season_wins: u32,
    pub season_losses: u32,
    pub weekly_wins: u32,
    pub weekly_losses: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PvpQueueKindSnapshot {
    Battleground { battleground_id: u32, name: String },
    RatedBracket { bracket: PvpBracketSnapshot },
    Arena { mode: ArenaQueueMode },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PvpQueueSnapshot {
    pub kind: PvpQueueKindSnapshot,
}

/// Retail PVPMatchTable columns plus per-map PvpStat counters, scoped to one match.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BattlegroundPlayerScoreSnapshot {
    pub character_id: u64,
    pub name: String,
    pub class: u8,
    pub level: u8,
    pub team: crate::battleground::BgTeam,
    pub killing_blows: u32,
    pub deaths: u32,
    pub damage_done: u64,
    pub healing_done: u64,
    /// Source PvPStatID/value pairs (`pvp_stat_columns`) measured by objective interactions.
    pub objective_scores: Vec<(u32, u32)>,
    /// Unknown honor policies are absent, not synthetic zero rewards/badges.
    pub honorable_kills: Option<u32>,
    pub honor_level: Option<u32>,
}

/// Retained match invitation; absent expiry means the source timer is unknown,
/// not an invented countdown. Waiting+accepted is entry pending, not a live match.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BattlegroundSnapshot {
    pub match_id: u64,
    pub battleground_id: u32,
    pub name: String,
    pub map_id: u32,
    pub team: crate::battleground::BgTeam,
    pub phase: crate::battleground::BgPhase,
    pub accepted: bool,
    pub invitation_time_left_secs: Option<u32>,
    pub score_a: u32,
    pub score_b: u32,
    pub flags: crate::battleground::FlagObjective,
    /// Authored WorldState IDs and authoritative values; no client-side scoring.
    pub world_states: Vec<(u32, i32)>,
    pub outcome: Option<crate::battleground::BgOutcome>,
    pub players: Vec<BattlegroundPlayerScoreSnapshot>,
    pub honor_reward: Option<u32>,
    pub arena: Option<ArenaMatchSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ArenaUnitSnapshot {
    pub character_id: u64,
    pub entity: u64,
    pub name: String,
    pub class: u8,
    pub team: crate::battleground::BgTeam,
    pub health: u32,
    pub max_health: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ArenaMatchSnapshot {
    pub mode: ArenaQueueMode,
    pub preparation_remaining_secs: Option<u32>,
    pub shadow_sight_available: bool,
    pub units: Vec<ArenaUnitSnapshot>,
    pub rating_changes: Vec<ArenaRatingChange>,
    pub shuffle: Option<SoloShuffleSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SoloShuffleSnapshot {
    /// Current1-based round; six rounds total under the realm's unrated policy.
    pub round: u8,
    pub total_rounds: u8,
    /// Character identity, cumulative round wins. Never team totals or rating.
    pub rounds_won: Vec<(u64, u8)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ArenaRatingChange {
    pub character_id: u64,
    pub rating_before: u32,
    pub rating_after: u32,
    pub mmr_before: u32,
    pub mmr_after: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PvpTalentSnapshot {
    pub id: u32,
    pub spell_id: u32,
    pub name: String,
    pub icon: u32,
    pub required_level: u8,
    pub slot_mask: u8,
    pub unavailable_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PvpTalentSlotSnapshot {
    pub index: u8,
    pub required_level: u8,
    pub selected: Option<u32>,
    pub can_change: bool,
    pub unlocked: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PvpTalentsSnapshot {
    pub spec_id: u32,
    pub slots: Vec<PvpTalentSlotSnapshot>,
    pub choices: Vec<PvpTalentSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WarModeSnapshot {
    pub desired: bool,
    pub active: bool,
    pub can_enable: bool,
    pub can_disable: bool,
    pub enlisted_spell: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PvpSnapshot {
    /// Total earned XP, independent of spendable1792. No certified level curve yet.
    pub lifetime_honor_xp: u64,
    pub honor_level: Option<u32>,
    pub honor: u32,
    pub honor_max: u32,
    pub conquest: u32,
    /// Dynamic season policy unavailable is None, not an invented cap.
    pub conquest_max: Option<u32>,
    pub brackets: Vec<PvpBracketStatsSnapshot>,
    pub queue: Option<PvpQueueSnapshot>,
    pub battleground: Option<BattlegroundSnapshot>,
    pub war_mode: Option<WarModeSnapshot>,
    pub talents: Option<PvpTalentsSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BarberShopSnapshot {
    pub appearance: CharacterAppearance,
    /// The player's money in copper.
    pub gold: u64,
}

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

pub fn register_snapshot_messages(app: &mut App) {
    register_server_snapshot::<QuestLogSnapshot>(app);
    register_server_snapshot::<GroupRosterSnapshot>(app);
    register_server_snapshot::<GroupCommandResponse>(app);
    register_server_snapshot::<CombatLogSnapshot>(app);
    register_server_snapshot::<CollectionSnapshot>(app);
    register_server_snapshot::<ProfessionSnapshot>(app);
    register_server_snapshot::<CurrencySnapshot>(app);
    register_server_snapshot::<ReputationSnapshot>(app);
    register_server_snapshot::<AchievementSnapshot>(app);
    register_server_snapshot::<AchievementToastSnapshot>(app);
    register_server_snapshot::<WorldMapSnapshot>(app);
    register_server_snapshot::<RestSnapshot>(app);
    register_server_snapshot::<FriendsSnapshot>(app);
    register_server_snapshot::<GuildSnapshot>(app);
    register_server_snapshot::<WhoSnapshot>(app);
    register_server_snapshot::<CalendarSnapshot>(app);
    register_server_snapshot::<IgnoreListSnapshot>(app);
    register_server_snapshot::<LfgSnapshot>(app);
    register_server_snapshot::<PvpSnapshot>(app);
    register_server_snapshot::<BarberShopSnapshot>(app);
    register_server_snapshot::<DeathSnapshot>(app);
    register_server_snapshot::<DurabilitySnapshot>(app);
    register_server_snapshot::<crate::protocol::DamageMeterSnapshot>(app);
    register_server_snapshot::<crate::protocol::ThreatUpdate>(app);
}

fn register_server_snapshot<M>(app: &mut App)
where
    M: lightyear::prelude::Message + Serialize + DeserializeOwned,
{
    app.register_protocol_message::<M>()
        .add_direction(NetworkDirection::ServerToClient);
}
