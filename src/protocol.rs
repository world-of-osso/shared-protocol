use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

pub use crate::protocol_snapshots::*;

use crate::components::{
    CharacterAppearance, CombatStatus, EquipmentAppearance, GuildMembership, Health, Mana,
    ModelDisplay, Mounted, MovementSpeed, Npc, Player, Position, PresenceStatus, Rotation, Zone,
};

/// Unreliable movement position updates, server-to-client only.
pub struct MovementChannel;

/// Reliable ordered channel for combat events, bidirectional.
pub struct CombatChannel;

/// Reliable ordered channel for chat messages, bidirectional.
pub struct ChatChannel;

/// Unreliable client-to-server channel for movement inputs.
pub struct InputChannel;

/// Reliable ordered channel for server-to-client terrain loading commands.
pub struct TerrainChannel;

/// Reliable ordered channel for authentication and character management, bidirectional.
pub struct AuthChannel;

/// Reliable ordered channel for auction house operations, bidirectional.
pub struct AuctionChannel;

/// Reliable ordered channel for trade operations, bidirectional.
pub struct TradeChannel;

/// Reliable ordered channel for talent operations, bidirectional.
pub struct TalentChannel;

/// Reliable ordered channel for profession operations, bidirectional.
pub struct ProfessionChannel;

/// Reliable ordered channel for reputation updates, bidirectional.
pub struct ReputationChannel;

/// Reliable ordered channel for achievement updates, bidirectional.
pub struct AchievementChannel;

/// Reliable ordered channel for world map updates, bidirectional.
pub struct WorldMapChannel;

/// Reliable ordered channel for resting state updates, bidirectional.
pub struct RestChannel;

/// Reliable ordered channel for friends updates, bidirectional.
pub struct FriendsChannel;

/// Reliable ordered channel for ignore list updates, bidirectional.
pub struct IgnoreChannel;

/// Reliable ordered channel for LFG updates, bidirectional.
pub struct LfgChannel;

/// Reliable ordered channel for PVP updates, bidirectional.
pub struct PvpChannel;

/// Reliable ordered channel for barber shop updates, bidirectional.
pub struct BarberShopChannel;

/// Reliable ordered channel for collection updates, bidirectional.
pub struct CollectionChannel;

/// Reliable ordered channel for currency updates, bidirectional.
pub struct CurrencyChannel;

/// Reliable ordered channel for inspect operations, bidirectional.
pub struct InspectChannel;

/// Reliable ordered channel for duel operations, bidirectional.
pub struct DuelChannel;

/// Reliable ordered channel for death and respawn operations, bidirectional.
pub struct DeathChannel;

/// Social emote animation kind.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmoteKind {
    Dance,
    Wave,
}

/// Chat message type.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ChatType {
    Say,
    Yell,
    Party,
    Guild,
    Whisper(String),
    Emote,
}

/// A chat message sent between client and server.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
    pub channel: ChatType,
}

/// Client request to set (or clear) their combat target.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SetTarget {
    /// Server entity bits of the target, or None to clear.
    pub target_entity: Option<u64>,
}

/// Combat event type for damage/death/respawn notifications.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CombatEventType {
    MeleeDamage,
    Death,
    Respawn,
}

/// Combat event sent from server to clients.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CombatEvent {
    pub attacker: u64,
    pub target: u64,
    pub damage: f32,
    pub event_type: CombatEventType,
}

/// Client requests to start casting a spell.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpellCastIntent {
    pub spell_id: Option<u32>,
    pub spell: String,
    pub target_entity: Option<u64>,
}

/// Client requests to stop the current spell cast.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StopSpellCast;

/// Client requests to invite a player to the group.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GroupInviteIntent {
    pub name: String,
}

/// Client requests to uninvite a player from the group.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GroupUninviteIntent {
    pub name: String,
}

/// Client requests a social emote animation and chat broadcast.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EmoteIntent {
    pub emote: EmoteKind,
}

/// Server broadcasts a social emote animation for a player entity.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EmoteEvent {
    pub player_entity: u64,
    pub sender: String,
    pub emote: EmoteKind,
}

/// Server tells clients which terrain map and initial tile to load.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoadTerrain {
    pub map_name: String,
    pub initial_tile_y: u32,
    pub initial_tile_x: u32,
}

/// Entry in the character list sent to the client.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CharacterListEntry {
    pub character_id: u64,
    pub name: String,
    pub level: u16,
    pub race: u8,
    pub class: u8,
    pub appearance: CharacterAppearance,
    pub equipment_appearance: EquipmentAppearance,
}

/// Client sends after connecting to authenticate.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginRequest {
    pub token: Option<String>,
    pub username: String,
    pub password: String,
}

/// Server responds with auth result and character list.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginResponse {
    pub success: bool,
    pub token: String,
    pub characters: Vec<CharacterListEntry>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CharacterListUpdate {
    pub character: CharacterListEntry,
}

/// Client sends to register a new account.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

/// Server responds to account registration.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegisterResponse {
    pub success: bool,
    pub token: String,
    pub pending_approval: bool,
    pub error: Option<String>,
}

/// Client requests creating a new character.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateCharacter {
    pub name: String,
    pub race: u8,
    pub class: u8,
    pub appearance: CharacterAppearance,
}

/// Server responds to character creation.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateCharacterResponse {
    pub success: bool,
    pub character: Option<CharacterListEntry>,
    pub error: Option<String>,
}

/// Client requests deleting a character.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeleteCharacter {
    pub character_id: u64,
}

/// Server responds to character deletion.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeleteCharacterResponse {
    pub success: bool,
    pub character_id: u64,
    pub error: Option<String>,
}

/// Client selects a character to enter the world with.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelectCharacter {
    pub character_id: u64,
}

/// Server responds to character selection with world entry result.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnterWorldResponse {
    pub success: bool,
    pub player_entity: Option<u64>,
    pub error: Option<String>,
}

/// Client requests inviting another character into their guild.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuildInviteRequest {
    pub target_character_name: String,
}

/// Server responds to guild invite request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuildInviteResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// Client accepts their pending guild invitation.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuildAcceptInviteRequest;

/// Server responds to guild invitation acceptance.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuildAcceptInviteResponse {
    pub success: bool,
    pub guild_id: Option<u32>,
    pub guild_name: Option<String>,
    pub error: Option<String>,
}

/// Client requests recent chat history.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatHistoryRequest {
    pub limit: u16,
}

/// Server responds with recent chat history.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatHistoryResponse {
    pub messages: Vec<ChatMessage>,
    pub error: Option<String>,
}

/// Client movement input sent each tick.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerInput {
    /// Normalized movement direction in world space.
    pub direction: [f32; 3],
    /// Character facing yaw in radians.
    pub facing_yaw: f32,
    /// Whether the player is jumping.
    pub jumping: bool,
    /// Whether the player is running (vs walking).
    pub running: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuctionDuration {
    Short,
    Medium,
    Long,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuctionTimeLeft {
    Short,
    Medium,
    Long,
    VeryLong,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuctionSortField {
    Name,
    MinBid,
    Buyout,
    TimeLeft,
    Quality,
    RequiredLevel,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuctionSortDir {
    Asc,
    Desc,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AuctionInventoryItem {
    pub item_guid: u64,
    pub item_id: u32,
    pub name: String,
    pub quality: u8,
    pub required_level: u16,
    pub stack_count: u32,
    pub vendor_sell_price: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AuctionListingSummary {
    pub auction_id: u64,
    pub item: AuctionInventoryItem,
    pub owner_name: String,
    pub stack_count: u32,
    pub min_bid: u32,
    pub current_bid: Option<u32>,
    pub min_next_bid: u32,
    pub buyout_price: Option<u32>,
    pub time_left: AuctionTimeLeft,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AuctionSearchQuery {
    pub text: String,
    pub page: u32,
    pub page_size: u32,
    pub min_level: Option<u16>,
    pub max_level: Option<u16>,
    pub quality: Option<u8>,
    pub usable_only: bool,
    pub sort_field: AuctionSortField,
    pub sort_dir: AuctionSortDir,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AuctionHouseOpened {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OpenAuctionHouse;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryAuctions {
    pub query: AuctionSearchQuery,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AuctionSearchResults {
    pub query: AuctionSearchQuery,
    pub total_results: u32,
    pub results: Vec<AuctionListingSummary>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PlaceBid {
    pub auction_id: u64,
    pub amount: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BuyoutAuction {
    pub auction_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreateAuction {
    pub item_guid: u64,
    pub stack_count: u32,
    pub min_bid: u32,
    pub buyout_price: Option<u32>,
    pub duration: AuctionDuration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CancelAuction {
    pub auction_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryOwnedAuctions;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryBidAuctions;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryAuctionInventory;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AuctionInventorySnapshot {
    pub gold: u32,
    pub items: Vec<AuctionInventoryItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OwnedAuctionListResponse {
    pub listings: Vec<AuctionListingSummary>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BidAuctionListResponse {
    pub listings: Vec<AuctionListingSummary>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AuctionOperationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AuctionMailEntry {
    pub mail_id: u64,
    pub subject: String,
    pub body: String,
    pub attached_money: u32,
    pub attached_item: Option<AuctionInventoryItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryAuctionMailbox;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AuctionMailboxSnapshot {
    pub entries: Vec<AuctionMailEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ClaimAuctionMail {
    pub mail_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradePhase {
    PendingOutgoing,
    PendingIncoming,
    Open,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TradeItemSnapshot {
    pub item_guid: u64,
    pub item_id: u32,
    pub name: String,
    pub quality: u8,
    pub stack_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TradePartySnapshot {
    pub name: String,
    pub accepted: bool,
    pub gold: u32,
    pub slots: Vec<Option<TradeItemSnapshot>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TradeSnapshot {
    pub phase: TradePhase,
    pub player: TradePartySnapshot,
    pub other: TradePartySnapshot,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InitiateTrade {
    pub target_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AcceptTrade;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DeclineTrade;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CancelTrade;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SetTradeItem {
    pub slot: u8,
    pub item_guid: u64,
    pub stack_count: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ClearTradeItem {
    pub slot: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SetTradeMoney {
    pub copper: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ConfirmTrade;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TradeStateUpdate {
    pub trade: Option<TradeSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryTalents;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ApplyTalentChoice {
    pub talent_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ResetTalents;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TalentSpecTabSnapshot {
    pub name: String,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TalentNodeSnapshot {
    pub talent_id: u32,
    pub name: String,
    pub points_spent: u8,
    pub max_points: u8,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TalentSnapshot {
    pub spec_tabs: Vec<TalentSpecTabSnapshot>,
    pub talents: Vec<TalentNodeSnapshot>,
    pub points_remaining: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TalentStateUpdate {
    pub snapshot: Option<TalentSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryInspectTarget {
    pub target_entity: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InspectSnapshot {
    pub target_name: String,
    pub equipment_appearance: EquipmentAppearance,
    pub talents: TalentSnapshot,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InspectStateUpdate {
    pub snapshot: Option<InspectSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InitiateDuel {
    pub target_entity: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AcceptDuel;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DeclineDuel;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DuelPhaseSnapshot {
    PendingOutgoing,
    PendingIncoming,
    Active,
    Completed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DuelBoundarySnapshot {
    pub center_x: f32,
    pub center_z: f32,
    pub radius: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DuelResultSnapshot {
    Won,
    Lost,
    Declined,
    Cancelled,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DuelSnapshot {
    pub phase: DuelPhaseSnapshot,
    pub opponent_name: String,
    pub boundary: Option<DuelBoundarySnapshot>,
    pub result: Option<DuelResultSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DuelStateUpdate {
    pub snapshot: Option<DuelSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryProfessions;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CraftProfessionRecipe {
    pub recipe_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GatherProfessionNode {
    pub node_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionStateUpdate {
    pub snapshot: Option<ProfessionSnapshot>,
    pub message: Option<String>,
    pub skill_up: Option<ProfessionSkillSnapshot>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SummonMount {
    pub mount_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DismissMount;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SummonPet {
    pub pet_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DismissPet;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CollectionStateUpdate {
    pub snapshot: Option<CollectionSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EarnCurrency {
    pub currency_id: u32,
    pub amount: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SpendCurrency {
    pub currency_id: u32,
    pub amount: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CurrencyStateUpdate {
    pub snapshot: Option<CurrencySnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ReputationStateUpdate {
    pub snapshot: Option<ReputationSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AchievementStateUpdate {
    pub snapshot: Option<AchievementSnapshot>,
    pub completed: Option<AchievementToastSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WorldMapStateUpdate {
    pub snapshot: Option<WorldMapSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RestStateUpdate {
    pub snapshot: Option<RestSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryFriends;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AddFriend {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RemoveFriend {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetPresenceStatus {
    pub status: PresenceStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FriendsStateUpdate {
    pub snapshot: Option<FriendsSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryIgnoreList;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AddIgnore {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RemoveIgnore {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IgnoreListStateUpdate {
    pub snapshot: Option<IgnoreListSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryLfgStatus;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueueForLfg {
    pub role: GroupRoleSnapshot,
    pub dungeon_ids: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DequeueFromLfg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RespondToLfgRoleCheck {
    pub accepted: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LfgStateUpdate {
    pub snapshot: Option<LfgSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryPvpStatus;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueueForBattleground {
    pub battleground_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueueForRatedPvp {
    pub bracket: PvpBracketSnapshot,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DequeueFromPvp;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PvpStateUpdate {
    pub snapshot: Option<PvpSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryBarberShopStatus;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ApplyBarberShopChanges {
    pub appearance: CharacterAppearance,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BarberShopStateUpdate {
    pub snapshot: Option<BarberShopSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QueryDeathStatus;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ReleaseSpirit;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ResurrectAtCorpse;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AcceptSpiritHealerResurrection;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DeathStateUpdate {
    pub snapshot: Option<DeathSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

/// Registers shared protocol: components for replication and channels.
/// Must be added AFTER `ServerPlugins`/`ClientPlugins` but BEFORE any entity is spawned.
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        register_replicated_components(app);
        register_messages(app);
        register_channels(app);
    }
}

fn register_replicated_components(app: &mut App) {
    app.register_component::<Position>();
    app.register_component::<Health>();
    app.register_component::<Mana>();
    app.register_component::<Player>();
    app.register_component::<Npc>();
    app.register_component::<ModelDisplay>();
    app.register_component::<Rotation>();
    app.register_component::<MovementSpeed>();
    app.register_component::<CombatStatus>();
    app.register_component::<Mounted>();
    app.register_component::<Zone>();
    app.register_component::<GuildMembership>();
    app.register_component::<PresenceStatus>();
    app.register_component::<EquipmentAppearance>();
}

fn register_messages(app: &mut App) {
    register_core_messages(app);
    register_account_messages(app);
    register_character_messages(app);
    register_guild_and_chat_messages(app);
    register_auction_messages(app);
    register_trade_messages(app);
    register_talent_messages(app);
    register_inspect_messages(app);
    register_duel_messages(app);
    register_profession_messages(app);
    register_reputation_messages(app);
    register_achievement_messages(app);
    register_world_map_messages(app);
    register_rest_messages(app);
    register_friends_messages(app);
    register_ignore_messages(app);
    register_lfg_messages(app);
    register_pvp_messages(app);
    register_barber_shop_messages(app);
    register_death_messages(app);
    register_collection_messages(app);
    register_currency_messages(app);
    crate::protocol_snapshots::register_snapshot_messages(app);
}

fn register_core_messages(app: &mut App) {
    app.register_message::<PlayerInput>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<ChatMessage>()
        .add_direction(NetworkDirection::Bidirectional);
    app.register_message::<SetTarget>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<CombatEvent>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<SpellCastIntent>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<StopSpellCast>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<EmoteIntent>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<EmoteEvent>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<GroupInviteIntent>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<GroupUninviteIntent>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<LoadTerrain>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_account_messages(app: &mut App) {
    app.register_message::<LoginRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<LoginResponse>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<RegisterRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<RegisterResponse>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_character_messages(app: &mut App) {
    app.register_message::<CreateCharacter>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<CreateCharacterResponse>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<DeleteCharacter>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<DeleteCharacterResponse>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<CharacterListUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<SelectCharacter>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<EnterWorldResponse>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_guild_and_chat_messages(app: &mut App) {
    app.register_message::<GuildInviteRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<GuildInviteResponse>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<GuildAcceptInviteRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<GuildAcceptInviteResponse>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<ChatHistoryRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<ChatHistoryResponse>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_auction_messages(app: &mut App) {
    register_auction_query_messages(app);
    register_auction_action_messages(app);
    register_auction_mail_messages(app);
}

fn register_auction_query_messages(app: &mut App) {
    app.register_message::<OpenAuctionHouse>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<AuctionHouseOpened>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<QueryAuctions>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<AuctionSearchResults>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<QueryOwnedAuctions>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<OwnedAuctionListResponse>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<QueryBidAuctions>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<BidAuctionListResponse>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<QueryAuctionInventory>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<AuctionInventorySnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_auction_action_messages(app: &mut App) {
    app.register_message::<PlaceBid>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<BuyoutAuction>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<CreateAuction>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<CancelAuction>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<AuctionOperationResponse>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_auction_mail_messages(app: &mut App) {
    app.register_message::<QueryAuctionMailbox>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<AuctionMailboxSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<ClaimAuctionMail>()
        .add_direction(NetworkDirection::ClientToServer);
}

fn register_trade_messages(app: &mut App) {
    app.register_message::<InitiateTrade>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<AcceptTrade>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<DeclineTrade>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<CancelTrade>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<SetTradeItem>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<ClearTradeItem>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<SetTradeMoney>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<ConfirmTrade>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<TradeStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_talent_messages(app: &mut App) {
    app.register_message::<QueryTalents>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<ApplyTalentChoice>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<ResetTalents>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<TalentStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_inspect_messages(app: &mut App) {
    app.register_message::<QueryInspectTarget>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<InspectStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_duel_messages(app: &mut App) {
    app.register_message::<InitiateDuel>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<AcceptDuel>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<DeclineDuel>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<DuelStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_profession_messages(app: &mut App) {
    app.register_message::<QueryProfessions>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<CraftProfessionRecipe>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<GatherProfessionNode>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<ProfessionStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_reputation_messages(app: &mut App) {
    app.register_message::<ReputationStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_achievement_messages(app: &mut App) {
    app.register_message::<AchievementStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_world_map_messages(app: &mut App) {
    app.register_message::<WorldMapStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_rest_messages(app: &mut App) {
    app.register_message::<RestStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_friends_messages(app: &mut App) {
    app.register_message::<QueryFriends>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<AddFriend>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<RemoveFriend>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<SetPresenceStatus>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<FriendsStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_ignore_messages(app: &mut App) {
    app.register_message::<QueryIgnoreList>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<AddIgnore>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<RemoveIgnore>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<IgnoreListStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_lfg_messages(app: &mut App) {
    app.register_message::<QueryLfgStatus>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<QueueForLfg>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<DequeueFromLfg>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<RespondToLfgRoleCheck>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<LfgStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_pvp_messages(app: &mut App) {
    app.register_message::<QueryPvpStatus>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<QueueForBattleground>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<QueueForRatedPvp>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<DequeueFromPvp>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<PvpStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_barber_shop_messages(app: &mut App) {
    app.register_message::<QueryBarberShopStatus>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<ApplyBarberShopChanges>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<BarberShopStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_death_messages(app: &mut App) {
    app.register_message::<QueryDeathStatus>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<ReleaseSpirit>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<ResurrectAtCorpse>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<AcceptSpiritHealerResurrection>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<DeathStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_collection_messages(app: &mut App) {
    app.register_message::<SummonMount>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<DismissMount>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<SummonPet>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<DismissPet>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<CollectionStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_currency_messages(app: &mut App) {
    app.register_message::<EarnCurrency>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<SpendCurrency>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<CurrencyStateUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn register_channels(app: &mut App) {
    app.add_channel::<MovementChannel>(ChannelSettings {
        mode: ChannelMode::UnorderedUnreliable,
        ..default()
    })
    .add_direction(NetworkDirection::ServerToClient);

    app.add_channel::<InputChannel>(ChannelSettings {
        mode: ChannelMode::UnorderedUnreliable,
        ..default()
    })
    .add_direction(NetworkDirection::ClientToServer);

    app.add_channel::<CombatChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<ChatChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<TerrainChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::ServerToClient);

    app.add_channel::<AuthChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<AuctionChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<TradeChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<TalentChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<InspectChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<DuelChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<ProfessionChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<ReputationChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<AchievementChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<WorldMapChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<FriendsChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<IgnoreChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<LfgChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<PvpChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<BarberShopChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<DeathChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<CollectionChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<CurrencyChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);

    app.add_channel::<RestChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::ServerToClient);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{EquipmentVisualSlot, EquippedAppearanceEntry};

    #[test]
    fn chat_type_serialization_round_trip() {
        let types = vec![
            ChatType::Say,
            ChatType::Yell,
            ChatType::Party,
            ChatType::Guild,
            ChatType::Whisper("TargetPlayer".into()),
            ChatType::Emote,
        ];
        for ct in types {
            let serialized = serde_json::to_string(&ct).unwrap();
            let deserialized: ChatType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(ct, deserialized);
        }
    }

    #[test]
    fn chat_message_serialization_round_trip() {
        let msg = ChatMessage {
            sender: "Alice".into(),
            content: "Hello world".into(),
            channel: ChatType::Whisper("Bob".into()),
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: ChatMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(msg.sender, deserialized.sender);
        assert_eq!(msg.content, deserialized.content);
        assert_eq!(msg.channel, deserialized.channel);
    }

    #[test]
    fn guild_invite_messages_round_trip() {
        let invite = GuildInviteRequest {
            target_character_name: "TargetOne".into(),
        };
        let encoded = serde_json::to_string(&invite).unwrap();
        let decoded: GuildInviteRequest = serde_json::from_str(&encoded).unwrap();
        assert_eq!(invite.target_character_name, decoded.target_character_name);

        let invite_resp = GuildInviteResponse {
            success: true,
            error: None,
        };
        let encoded = serde_json::to_string(&invite_resp).unwrap();
        let decoded: GuildInviteResponse = serde_json::from_str(&encoded).unwrap();
        assert_eq!(invite_resp.success, decoded.success);
        assert_eq!(invite_resp.error, decoded.error);

        let accept_resp = GuildAcceptInviteResponse {
            success: true,
            guild_id: Some(3),
            guild_name: Some("Raid Team".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&accept_resp).unwrap();
        let decoded: GuildAcceptInviteResponse = serde_json::from_str(&encoded).unwrap();
        assert_eq!(accept_resp.success, decoded.success);
        assert_eq!(accept_resp.guild_id, decoded.guild_id);
        assert_eq!(accept_resp.guild_name, decoded.guild_name);
        assert_eq!(accept_resp.error, decoded.error);
    }

    #[test]
    fn chat_history_response_round_trip() {
        let resp = ChatHistoryResponse {
            messages: vec![
                ChatMessage {
                    sender: "Alice".into(),
                    content: "hello".into(),
                    channel: ChatType::Say,
                },
                ChatMessage {
                    sender: "Bob".into(),
                    content: "guild hi".into(),
                    channel: ChatType::Guild,
                },
            ],
            error: None,
        };
        let encoded = serde_json::to_string(&resp).unwrap();
        let decoded: ChatHistoryResponse = serde_json::from_str(&encoded).unwrap();
        assert_eq!(resp.messages.len(), decoded.messages.len());
        assert_eq!(resp.messages[0].sender, decoded.messages[0].sender);
        assert_eq!(resp.messages[1].channel, decoded.messages[1].channel);
        assert_eq!(resp.error, decoded.error);
    }

    #[test]
    fn emote_messages_round_trip() {
        let intent = EmoteIntent {
            emote: EmoteKind::Dance,
        };
        let encoded = serde_json::to_string(&intent).unwrap();
        let decoded: EmoteIntent = serde_json::from_str(&encoded).unwrap();
        assert_eq!(intent, decoded);

        let event = EmoteEvent {
            player_entity: 77,
            sender: "Alice".into(),
            emote: EmoteKind::Wave,
        };
        let encoded = serde_json::to_string(&event).unwrap();
        let decoded: EmoteEvent = serde_json::from_str(&encoded).unwrap();
        assert_eq!(event, decoded);
    }

    #[test]
    fn trade_state_update_round_trip() {
        let update = TradeStateUpdate {
            trade: Some(TradeSnapshot {
                phase: TradePhase::Open,
                player: TradePartySnapshot {
                    name: "Theron".into(),
                    accepted: true,
                    gold: 12_345,
                    slots: vec![Some(TradeItemSnapshot {
                        item_guid: 99,
                        item_id: 17,
                        name: "Bronze Sword".into(),
                        quality: 2,
                        stack_count: 1,
                    })],
                },
                other: TradePartySnapshot {
                    name: "Alice".into(),
                    accepted: false,
                    gold: 500,
                    slots: vec![None],
                },
            }),
            message: Some("trade updated".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: TradeStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn talent_state_update_round_trip() {
        let update = TalentStateUpdate {
            snapshot: Some(TalentSnapshot {
                spec_tabs: vec![
                    TalentSpecTabSnapshot {
                        name: "Protection".into(),
                        active: true,
                    },
                    TalentSpecTabSnapshot {
                        name: "Holy".into(),
                        active: false,
                    },
                ],
                talents: vec![TalentNodeSnapshot {
                    talent_id: 101,
                    name: "Divine Strength".into(),
                    points_spent: 1,
                    max_points: 1,
                    active: true,
                }],
                points_remaining: 50,
            }),
            message: Some("talent applied".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: TalentStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn profession_state_update_round_trip() {
        let update = ProfessionStateUpdate {
            snapshot: Some(ProfessionSnapshot {
                skills: vec![ProfessionSkillSnapshot {
                    profession: "Mining".into(),
                    current: 12,
                    max: 75,
                }],
                recipes: vec![ProfessionRecipeSnapshot {
                    spell_id: 5001,
                    profession: "Blacksmithing".into(),
                    name: "Copper Bracers".into(),
                    craftable: true,
                    cooldown: None,
                }],
            }),
            message: Some("crafted Copper Bracers".into()),
            skill_up: Some(ProfessionSkillSnapshot {
                profession: "Blacksmithing".into(),
                current: 13,
                max: 75,
            }),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: ProfessionStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn reputation_state_update_round_trip() {
        let update = ReputationStateUpdate {
            snapshot: Some(ReputationSnapshot {
                entries: vec![
                    ReputationEntrySnapshot {
                        faction_id: 72,
                        faction_name: "Stormwind".into(),
                        standing: "Friendly".into(),
                        value: 21_010,
                    },
                    ReputationEntrySnapshot {
                        faction_id: 47,
                        faction_name: "Ironforge".into(),
                        standing: "Friendly".into(),
                        value: 21_002,
                    },
                ],
            }),
            message: Some("gained 10 reputation with Stormwind".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: ReputationStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn currency_state_update_round_trip() {
        let update = CurrencyStateUpdate {
            snapshot: Some(CurrencySnapshot {
                entries: vec![CurrencyEntrySnapshot {
                    id: 1,
                    name: "Honor".into(),
                    amount: 125,
                }],
            }),
            message: Some("earned 125 Honor".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: CurrencyStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn collection_state_update_round_trip() {
        let update = CollectionStateUpdate {
            snapshot: Some(CollectionSnapshot {
                mounts: vec![CollectionMountSnapshot {
                    mount_id: 101,
                    name: "Swift Brown Steed".into(),
                    known: true,
                    active: true,
                }],
                pets: vec![CollectionPetSnapshot {
                    pet_id: 202,
                    name: "Brown Rabbit".into(),
                    known: true,
                    active: false,
                }],
            }),
            message: Some("summoned Swift Brown Steed".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: CollectionStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn achievement_state_update_round_trip() {
        let update = AchievementStateUpdate {
            snapshot: Some(AchievementSnapshot {
                earned_ids: vec![1, 2],
                progress: vec![AchievementProgressSnapshot {
                    achievement_id: 3,
                    current: 37,
                    required: 40,
                    completed: false,
                }],
            }),
            completed: Some(AchievementToastSnapshot {
                achievement_id: 2,
                name: "Level 20".into(),
                points: 10,
            }),
            message: Some("achievement progress updated".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: AchievementStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn world_map_state_update_round_trip() {
        let update = WorldMapStateUpdate {
            snapshot: Some(WorldMapSnapshot {
                discovered_zone_ids: vec![12, 1519, 1537],
            }),
            message: Some("world map discovery updated".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: WorldMapStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn rest_state_update_round_trip() {
        let update = RestStateUpdate {
            snapshot: Some(RestSnapshot {
                in_rest_area: true,
                rest_area_kind: Some(RestAreaKindSnapshot::Inn),
                rested_xp: 240,
                rested_xp_max: 600,
            }),
            message: Some("rest state updated".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: RestStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn friends_state_update_round_trip() {
        let update = FriendsStateUpdate {
            snapshot: Some(FriendsSnapshot {
                entries: vec![FriendCharacterSnapshot {
                    name: "Theron".into(),
                    level: 12,
                    class_name: "Paladin".into(),
                    area: "Elwynn Forest".into(),
                    presence: PresenceStatus::Online,
                    note: String::new(),
                }],
            }),
            message: Some("friend added".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: FriendsStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn ignore_list_state_update_round_trip() {
        let update = IgnoreListStateUpdate {
            snapshot: Some(IgnoreListSnapshot {
                names: vec!["Alice".into(), "Bob".into()],
            }),
            message: Some("ignore list updated".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: IgnoreListStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn lfg_state_update_round_trip() {
        let update = LfgStateUpdate {
            snapshot: Some(LfgSnapshot {
                queued: true,
                selected_role: Some(GroupRoleSnapshot::Tank),
                dungeon_ids: vec![100],
                queue_size: 3,
                average_wait_secs: 42,
                in_demand_roles: vec![GroupRoleSnapshot::Healer],
                role_check: Some(LfgRoleCheckSnapshot {
                    dungeon_id: 100,
                    dungeon_name: "Deadmines".into(),
                    assigned_role: GroupRoleSnapshot::Tank,
                    accepted_count: 2,
                    total_count: 5,
                }),
                match_found: Some(LfgMatchFoundSnapshot {
                    dungeon_id: 100,
                    dungeon_name: "Deadmines".into(),
                    assigned_role: GroupRoleSnapshot::Tank,
                    members: vec![LfgMatchMemberSnapshot {
                        name: "Theron".into(),
                        role: GroupRoleSnapshot::Tank,
                    }],
                }),
            }),
            message: Some("role check started".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: LfgStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn pvp_state_update_round_trip() {
        let update = PvpStateUpdate {
            snapshot: Some(PvpSnapshot {
                honor: 750,
                honor_max: 15_000,
                conquest: 120,
                conquest_max: 1_800,
                brackets: vec![
                    PvpBracketStatsSnapshot {
                        bracket: PvpBracketSnapshot::Arena2v2,
                        rating: 1516,
                        season_wins: 1,
                        season_losses: 0,
                        weekly_wins: 1,
                        weekly_losses: 0,
                    },
                    PvpBracketStatsSnapshot {
                        bracket: PvpBracketSnapshot::RatedBattleground,
                        rating: 1500,
                        season_wins: 0,
                        season_losses: 0,
                        weekly_wins: 0,
                        weekly_losses: 0,
                    },
                ],
                queue: Some(PvpQueueSnapshot {
                    kind: PvpQueueKindSnapshot::Battleground {
                        battleground_id: 1,
                        name: "Warsong Gulch".into(),
                    },
                }),
            }),
            message: Some("queued for Warsong Gulch".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: PvpStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn barber_shop_state_update_round_trip() {
        let update = BarberShopStateUpdate {
            snapshot: Some(BarberShopSnapshot {
                appearance: CharacterAppearance {
                    sex: 0,
                    skin_color: 2,
                    face: 3,
                    eye_color: 4,
                    hair_style: 5,
                    hair_color: 6,
                    facial_style: 1,
                },
                gold: 87_500,
            }),
            message: Some("barber shop ready".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: BarberShopStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn death_state_update_round_trip() {
        let update = DeathStateUpdate {
            snapshot: Some(DeathSnapshot {
                state: DeathStateSnapshot::Ghost,
                corpse: Some(DeathPositionSnapshot {
                    map_id: 0,
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                }),
                graveyard: Some(DeathPositionSnapshot {
                    map_id: 0,
                    x: 4.0,
                    y: 5.0,
                    z: 6.0,
                }),
                can_resurrect_at_corpse: true,
                spirit_healer_available: false,
            }),
            message: Some("released spirit".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: DeathStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn inspect_state_update_round_trip() {
        let update = InspectStateUpdate {
            snapshot: Some(InspectSnapshot {
                target_name: "Alice".into(),
                equipment_appearance: EquipmentAppearance {
                    entries: vec![EquippedAppearanceEntry {
                        slot: EquipmentVisualSlot::Head,
                        item_id: Some(100),
                        display_info_id: Some(200),
                        inventory_type: 1,
                        hidden: false,
                    }],
                },
                talents: TalentSnapshot {
                    spec_tabs: vec![TalentSpecTabSnapshot {
                        name: "Protection".into(),
                        active: true,
                    }],
                    talents: vec![TalentNodeSnapshot {
                        talent_id: 101,
                        name: "Divine Strength".into(),
                        points_spent: 1,
                        max_points: 1,
                        active: true,
                    }],
                    points_remaining: 50,
                },
            }),
            message: Some("inspect ready".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: InspectStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn duel_state_update_round_trip() {
        let update = DuelStateUpdate {
            snapshot: Some(DuelSnapshot {
                phase: DuelPhaseSnapshot::Active,
                opponent_name: "Alice".into(),
                boundary: Some(DuelBoundarySnapshot {
                    center_x: 10.0,
                    center_z: 15.0,
                    radius: 30.0,
                }),
                result: None,
            }),
            message: Some("duel started".into()),
            error: None,
        };
        let encoded = serde_json::to_string(&update).unwrap();
        let decoded: DuelStateUpdate = serde_json::from_str(&encoded).unwrap();
        assert_eq!(update, decoded);
    }

    fn sample_auction_search_results() -> AuctionSearchResults {
        AuctionSearchResults {
            query: AuctionSearchQuery {
                text: "linen".into(),
                page: 0,
                page_size: 20,
                min_level: Some(1),
                max_level: Some(10),
                quality: Some(1),
                usable_only: false,
                sort_field: AuctionSortField::Name,
                sort_dir: AuctionSortDir::Asc,
            },
            total_results: 1,
            results: vec![AuctionListingSummary {
                auction_id: 7,
                item: AuctionInventoryItem {
                    item_guid: 12,
                    item_id: 2589,
                    name: "Linen Cloth".into(),
                    quality: 1,
                    required_level: 1,
                    stack_count: 20,
                    vendor_sell_price: 13,
                },
                owner_name: "Seller".into(),
                stack_count: 20,
                min_bid: 100,
                current_bid: Some(125),
                min_next_bid: 131,
                buyout_price: Some(200),
                time_left: AuctionTimeLeft::Long,
            }],
        }
    }

    #[test]
    fn auction_search_results_round_trip() {
        let msg = sample_auction_search_results();
        let encoded = serde_json::to_string(&msg).unwrap();
        let decoded: AuctionSearchResults = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.query.text, "linen");
        assert_eq!(decoded.total_results, 1);
        assert_eq!(decoded.results[0].auction_id, 7);
        assert_eq!(decoded.results[0].item.name, "Linen Cloth");
        assert_eq!(decoded.results[0].time_left, AuctionTimeLeft::Long);
    }

    #[test]
    fn equipment_appearance_round_trips() {
        let snapshot = EquipmentAppearance {
            entries: vec![
                EquippedAppearanceEntry {
                    slot: EquipmentVisualSlot::Head,
                    item_id: Some(19019),
                    display_info_id: Some(12345),
                    inventory_type: 1,
                    hidden: false,
                },
                EquippedAppearanceEntry {
                    slot: EquipmentVisualSlot::MainHand,
                    item_id: Some(17182),
                    display_info_id: Some(54321),
                    inventory_type: 21,
                    hidden: false,
                },
            ],
        };

        let bitcode_encoded = bitcode::encode(&snapshot);
        let bitcode_decoded: EquipmentAppearance = bitcode::decode(&bitcode_encoded).unwrap();
        assert_eq!(bitcode_decoded, snapshot);

        let json_encoded = serde_json::to_string(&snapshot).unwrap();
        let json_decoded: EquipmentAppearance = serde_json::from_str(&json_encoded).unwrap();
        assert_eq!(json_decoded, snapshot);
    }
}
