//! NPC interaction (Retail `GOSSIP_HELLO` / `NPC_INTERACTION_OPEN_RESULT` semantics).
//!
//! Every NPC replicates `NpcFlags`. The client sends `InteractNpc` when the player
//! right-clicks an NPC; the server validates the request and answers with
//! `InteractionOpened` (the frame to open) or `InteractionFailed`. A gossip menu
//! routes its options through `SelectGossipOption`. The server sends
//! `InteractionClosed` when the interaction ends (player walked away, NPC died or
//! entered combat, the client sent `CloseInteraction`, or another NPC was opened).
//! NPCs are addressed by server entity bits, like `SetTarget`.
//!
//! Game objects replicate `GameObjectInfo`. The client sends `UseGameObject` when
//! the player right-clicks one (Retail `CMSG_GAME_OBJ_USE`); the server answers
//! with the same `InteractionOpened` / `InteractionFailed` / `InteractionClosed`
//! messages, whose `npc` field then carries the object's entity bits. All messages
//! use `InteractionChannel`.

use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

/// Reliable ordered channel for NPC interaction, bidirectional.
pub struct InteractionChannel;

/// Retail NPC interaction flags: bits 0-31 are `UNIT_NPC_FLAG_*` (TrinityCore
/// `NPCFlags`), bits 32-63 are `UNIT_NPC_FLAG_2_*` (`NPCFlags2`).
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct NpcFlags(pub u64);

impl NpcFlags {
    pub const GOSSIP: u64 = 0x0000_0001;
    pub const QUESTGIVER: u64 = 0x0000_0002;
    pub const TRAINER: u64 = 0x0000_0010;
    pub const TRAINER_CLASS: u64 = 0x0000_0020;
    pub const TRAINER_PROFESSION: u64 = 0x0000_0040;
    pub const VENDOR: u64 = 0x0000_0080;
    pub const VENDOR_AMMO: u64 = 0x0000_0100;
    pub const VENDOR_FOOD: u64 = 0x0000_0200;
    pub const VENDOR_POISON: u64 = 0x0000_0400;
    pub const VENDOR_REAGENT: u64 = 0x0000_0800;
    pub const REPAIR: u64 = 0x0000_1000;
    pub const FLIGHTMASTER: u64 = 0x0000_2000;
    pub const SPIRIT_HEALER: u64 = 0x0000_4000;
    pub const AREA_SPIRIT_HEALER: u64 = 0x0000_8000;
    pub const INNKEEPER: u64 = 0x0001_0000;
    pub const BANKER: u64 = 0x0002_0000;
    pub const PETITIONER: u64 = 0x0004_0000;
    pub const TABARD_DESIGNER: u64 = 0x0008_0000;
    pub const BATTLEMASTER: u64 = 0x0010_0000;
    pub const AUCTIONEER: u64 = 0x0020_0000;
    pub const STABLEMASTER: u64 = 0x0040_0000;
    pub const GUILD_BANKER: u64 = 0x0080_0000;
    pub const SPELLCLICK: u64 = 0x0100_0000;
    pub const PLAYER_VEHICLE: u64 = 0x0200_0000;
    pub const MAILBOX: u64 = 0x0400_0000;
    pub const ARTIFACT_POWER_RESPEC: u64 = 0x0800_0000;
    pub const TRANSMOGRIFIER: u64 = 0x1000_0000;
    pub const VAULTKEEPER: u64 = 0x2000_0000;
    pub const WILD_BATTLE_PET: u64 = 0x4000_0000;
    pub const BLACK_MARKET: u64 = 0x8000_0000;

    /// Bits of AzerothCore (3.3.5) `NPCFlags` that exist in Retail. AzerothCore
    /// bits 0x1 (GOSSIP) … 0x0400_0000 (MAILBOX) have the same value and meaning in
    /// Retail (its SPIRITGUIDE 0x8000 is Retail AREA_SPIRIT_HEALER). AzerothCore
    /// defines no higher bits and no `NPCFlags2`.
    const AZEROTHCORE_MASK: u32 = 0x07FF_FFFF;

    /// Retail flags from AzerothCore `creature_template.npcflag`.
    pub fn from_azerothcore(npcflag: u32) -> Self {
        Self(u64::from(npcflag & Self::AZEROTHCORE_MASK))
    }

    pub fn contains(self, flag: u64) -> bool {
        self.0 & flag != 0
    }
}

/// Retail frame opened by an NPC role (Retail `PlayerInteractionType`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NpcRole {
    QuestGiver,
    Vendor,
    Trainer,
    FlightMaster,
    Innkeeper,
    Banker,
    Petitioner,
    TabardDesigner,
    Battlemaster,
    AuctionHouse,
    StableMaster,
    GuildBanker,
}

/// AzerothCore `GAMEOBJECT_TYPE_GUILD_BANK`.
pub const GAMEOBJECT_TYPE_GUILD_BANK: u8 = 34;

/// A world game object (AzerothCore `gameobject_template`).
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    PartialEq,
)]
pub struct GameObjectInfo {
    pub entry: u32,
    /// AzerothCore `GameobjectTypes`, e.g. `GAMEOBJECT_TYPE_GUILD_BANK`.
    pub go_type: u8,
    /// Retail `GameObjectDisplayInfo` ID.
    pub display_id: u32,
    pub name: String,
    pub scale: f32,
}

/// One selectable gossip line. `icon` is the Retail `GossipOptionIcon` id
/// (0 chat, 1 vendor, 2 taxi, 3 trainer, 5 binder, 6 money bag, 7 talk, 8 tabard,
/// 9 battle).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GossipMenuOption {
    pub option_id: u32,
    pub icon: u8,
    pub text: String,
}

/// A gossip frame: NPC greeting text plus options. `menu_id` is the
/// `gossip_menu` id (0 for a menu built from the NPC's roles).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GossipMenu {
    pub menu_id: u32,
    pub text: String,
    pub options: Vec<GossipMenuOption>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum InteractionKind {
    Gossip(GossipMenu),
    Role(NpcRole),
}

/// Why the server refused an interaction; `message` is the Retail UI error text.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionError {
    /// Unknown entity or an NPC with nothing to interact with.
    InvalidTarget,
    TooFarAway,
    TargetDead,
    TargetHostile,
    TargetInCombat,
    PlayerDead,
}

impl InteractionError {
    pub fn message(self) -> &'static str {
        match self {
            Self::InvalidTarget => "Invalid target", // SPELL_FAILED_BAD_TARGETS
            Self::TooFarAway => "You are too far away.", // ERR_USE_TOO_FAR
            Self::TargetDead => "Your target is dead", // SPELL_FAILED_TARGETS_DEAD
            Self::TargetHostile => "Target is hostile", // SPELL_FAILED_TARGET_ENEMY
            Self::TargetInCombat => "Target is in combat", // SPELL_FAILED_TARGET_AFFECTING_COMBAT
            Self::PlayerDead => "You can't do that when you're dead.", // ERR_PLAYER_DEAD
        }
    }
}

/// Client right-clicked an NPC (Retail `CMSG_GOSSIP_HELLO` and the role hellos).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InteractNpc {
    pub npc: u64,
}

/// Client right-clicked a game object (Retail `CMSG_GAME_OBJ_USE`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UseGameObject {
    pub object: u64,
}

/// Client picked an option of the open gossip menu (`CMSG_GOSSIP_SELECT_OPTION`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SelectGossipOption {
    pub npc: u64,
    pub option_id: u32,
}

/// Client closed the interaction frame (`CMSG_CLOSE_INTERACTION`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CloseInteraction {
    pub npc: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InteractionOpened {
    pub npc: u64,
    pub kind: InteractionKind,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InteractionFailed {
    pub npc: u64,
    pub error: InteractionError,
}

/// The interaction with `npc` ended server-side; close its frame.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InteractionClosed {
    pub npc: u64,
}

pub(super) fn register_interaction_protocol(app: &mut App) {
    app.component::<NpcFlags>().replicate();
    app.component::<GameObjectInfo>().replicate();
    app.add_channel::<InteractionChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_message::<InteractNpc>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<UseGameObject>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<SelectGossipOption>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<CloseInteraction>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_message::<InteractionOpened>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<InteractionFailed>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<InteractionClosed>()
        .add_direction(NetworkDirection::ServerToClient);
}
