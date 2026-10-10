//! Replicated unit state for unit frames: powers, auras, level and faction.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Retail `PowerType` DB2 `PowerTypeEnum` values (world.db `power_type`).
#[derive(
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum PowerType {
    Mana = 0,
    Rage = 1,
    Focus = 2,
    Energy = 3,
    ComboPoints = 4,
    Runes = 5,
    RunicPower = 6,
    SoulShards = 7,
    LunarPower = 8,
    HolyPower = 9,
    Alternate = 10,
    Maelstrom = 11,
    Chi = 12,
    Insanity = 13,
    ArcaneCharges = 16,
    Fury = 17,
    Pain = 18,
    Essence = 19,
    AlternateQuest = 23,
    AlternateEncounter = 24,
    AlternateMount = 25,
}

impl PowerType {
    /// Maps a DB2 `PowerTypeEnum` (also `SpellPower.PowerType`) to a variant.
    pub fn from_db(value: i32) -> Option<Self> {
        Some(match value {
            0 => Self::Mana,
            1 => Self::Rage,
            2 => Self::Focus,
            3 => Self::Energy,
            4 => Self::ComboPoints,
            5 => Self::Runes,
            6 => Self::RunicPower,
            7 => Self::SoulShards,
            8 => Self::LunarPower,
            9 => Self::HolyPower,
            10 => Self::Alternate,
            11 => Self::Maelstrom,
            12 => Self::Chi,
            13 => Self::Insanity,
            16 => Self::ArcaneCharges,
            17 => Self::Fury,
            18 => Self::Pain,
            19 => Self::Essence,
            23 => Self::AlternateQuest,
            24 => Self::AlternateEncounter,
            25 => Self::AlternateMount,
            _ => return None,
        })
    }
}

/// One power bar value in raw DB2 units.
#[derive(
    Reflect, Serialize, Deserialize, bitcode::Encode, bitcode::Decode, Debug, Clone, PartialEq,
)]
pub struct PowerEntry {
    pub power: PowerType,
    pub current: i32,
    pub max: i32,
    /// Progress toward the next whole unit when sent, thousandths (retail `UnitPartialPower`).
    pub partial: u16,
    /// Effective raw units per second when sent (negative decays, 0 idle); the client
    /// predicts `current + partial` between sends, which happen on whole-unit changes.
    pub regen_per_sec: f32,
}

/// All powers of a unit; the primary power comes first.
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
    Default,
)]
pub struct UnitPowers {
    pub entries: Vec<PowerEntry>,
    /// Charged combo points, 1-based ascending (retail `GetUnitChargedPowerPoints`).
    pub charged_points: Vec<u8>,
}

/// Death Knight rune cooldowns in rune index order (retail `GetRuneCooldown`): rune `i`
/// started `duration_ms - ready_in_ms[i]` before the send and is ready when its
/// `ready_in_ms` is 0. A rune waiting behind the three recharging ones has a
/// `ready_in_ms` above `duration_ms`, i.e. a start in the future.
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
    Default,
)]
pub struct UnitRunes {
    /// Full recharge time of one rune when sent, ms.
    pub duration_ms: u32,
    /// Per rune, ms until it is ready when sent.
    pub ready_in_ms: Vec<u32>,
}

/// Client-facing view of one aura on a unit.
#[derive(
    Reflect, Serialize, Deserialize, bitcode::Encode, bitcode::Decode, Debug, Clone, PartialEq,
)]
pub struct AuraView {
    /// Data-backed cast/presentation overrides. Removing the aura restores base state.
    pub overrides: Vec<AuraOverride>,
    /// Server-assigned id, unique per unit while the aura exists.
    pub instance_id: u32,
    pub spell_id: u32,
    /// Caster entity bits, if the caster is known.
    pub caster: Option<u64>,
    pub stacks: u8,
    pub charges: u8,
    /// Full duration; 0 means permanent.
    pub duration_ms: u32,
    /// Remaining duration when the server sent this value.
    pub remaining_ms: u32,
    pub harmful: bool,
    /// Retail `SpellDispelType` id (0 none, 1 magic, 2 curse, 3 disease, 4 poison).
    pub dispel_type: u8,
    /// Bit set of `AuraView::FLAG_*`.
    pub flags: u16,
}

/// Aura332 substitutions, aura293 ordered spell sets and aura312 animation sets.
#[derive(
    Reflect, Serialize, Deserialize, bitcode::Encode, bitcode::Decode, Debug, Clone, PartialEq,
)]
pub enum AuraOverride {
    ActionBar {
        spell_id: u32,
        replacement: u32,
    },
    Animation(u32),
    /// Aura293 OverrideSpellData's ordered10 slots, including zero/empty entries.
    SpellSet {
        id: u32,
        spells: Vec<u32>,
    },
}

impl AuraView {
    pub const FLAG_PASSIVE: u16 = 1 << 0;
    pub const FLAG_HIDDEN: u16 = 1 << 1;
    pub const FLAG_FROM_PLAYER: u16 = 1 << 2;
}

/// All auras currently on a unit.
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
    Default,
)]
pub struct UnitAuras {
    pub auras: Vec<AuraView>,
}

/// Unit level.
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct UnitLevel(pub u8);

/// Retail `FactionTemplate` id, used for reaction/hostility colouring.
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct UnitFactionTemplate(pub u32);

/// `UNIT_FIELD_FLAGS` (TrinityCore `UnitFlags`, `m_unitData->Flags`).
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct UnitFlags(pub u32);

impl UnitFlags {
    /// `UNIT_FLAG_NOT_SELECTABLE`: no nameplate, and the unit cannot be targeted or clicked.
    pub const NOT_SELECTABLE: u32 = 0x0200_0000;
    /// `UNIT_FLAG_NON_ATTACKABLE_2` (`SPELL_AURA_MOD_UNATTACKABLE`): the unit cannot be
    /// attacked (`WorldObject::IsValidAttackTarget`).
    pub const NON_ATTACKABLE_2: u32 = 0x0001_0000;
    /// `UNIT_FLAG_PET_IN_COMBAT`: on a player's pet, the pet is going after a target to
    /// attack (`PetAI::DoAttack`); the pet bar's Attack button is active while it is set.
    pub const PET_IN_COMBAT: u32 = 0x0000_0800;

    pub fn is_selectable(self) -> bool {
        self.0 & Self::NOT_SELECTABLE == 0
    }

    pub fn is_attackable(self) -> bool {
        self.0 & Self::NON_ATTACKABLE_2 == 0
    }
}

/// Entity bits of the units on this creature's threat list (TrinityCore
/// `SMSG_THREAT_UPDATE` membership, sent to the creature's viewers). The Retail UI's
/// `UnitDetailedThreatSituation(unit, creature)` is non-nil exactly for them: nameplates
/// of a non-friendly creature with the player on this list turn hostile red
/// (`CompactUnitFrame_IsOnThreatListWithPlayer`, `considerSelectionInCombatAsHostile`).
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
    Eq,
    Default,
)]
pub struct UnitThreatList(pub Vec<u64>);

/// Stable character IDs of eligible creature tappers. Empty means unclaimed.
/// Clients compare against their local player and current group roster.
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
    Eq,
    Default,
)]
pub struct UnitTap(pub Vec<u64>);

impl UnitTap {
    pub fn denied(&self, viewer: u64, group: &[u64]) -> bool {
        !self.0.is_empty()
            && !self.0.contains(&viewer)
            && !group.iter().any(|member| self.0.contains(member))
    }
}

/// Entity bits of the unit's current target (`None` = no target), for target-of-target.
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct UnitTarget(pub Option<u64>);

/// How a creature is moving: Retail `MOVEMENTFLAG_FORWARD`, walking with
/// `MOVEMENTFLAG_WALKING` (`MoveSplineFlag::Walkmode`). The client plays the stand, walk
/// or run animation from it.
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
)]
pub enum CreatureMotion {
    #[default]
    Still,
    Walk,
    Run,
}

/// How a player is moving, as Retail sends it in the `MovementInfo` flags of each
/// `SMSG_MOVE_UPDATE` the server rebroadcasts from a player's `CMSG_MOVE_*`
/// (TrinityCore `HandleMovementOpcode`). Bits keep TrinityCore `MovementFlags` values
/// (MovementInfo.h); other clients pick the remote unit's animation from them. The server
/// sets direction/modes from each applied `PlayerInput` and airborne state from gravity.
/// `JUMP_STARTED` is a project marker, not a Retail movement flag.
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
)]
pub struct PlayerMotion(pub u64);

impl PlayerMotion {
    pub const FORWARD: u64 = 0x0000_0001;
    pub const BACKWARD: u64 = 0x0000_0002;
    pub const STRAFE_LEFT: u64 = 0x0000_0004;
    pub const STRAFE_RIGHT: u64 = 0x0000_0008;
    /// Walk mode (run toggled off).
    pub const WALKING: u64 = 0x0000_0100;
    /// Airborne under gravity, whether a jump or an unjumped fall.
    pub const FALLING: u64 = 0x0000_0800;
    /// This airborne interval began with a jump; cleared on landing, swimming or flight.
    /// Project-only bit outside Retail's defined flags. Retail carries jump origin in
    /// CMSG_MOVE_JUMP, not FALLING_FAR (which describes a falling phase, not its origin).
    pub const JUMP_STARTED: u64 = 0x8000_0000_0000_0000;
    pub const SWIMMING: u64 = 0x0010_0000;
    /// The unit may fly: set by the server while a SPELL_AURA_FLY (201) or
    /// SPELL_AURA_MOD_INCREASE_MOUNTED_FLIGHT_SPEED (207) aura is on it (`Unit::SetCanFly`).
    pub const CAN_FLY: u64 = 0x0080_0000;
    /// The unit is flying: no gravity (a `PlayerInput::flying` the server accepted).
    pub const FLYING: u64 = 0x0100_0000;
    /// The unit may skyride: set by the server while a SPELL_AURA_ADV_FLYING (446) aura is
    /// on it (`AuraEffect::HandleModAdvFlying` → `Unit::SetCanAdvFly`, Unit.cpp:14027).
    pub const CAN_ADV_FLY: u64 = 0x0000_2000_0000_0000;
    /// The unit is skyriding: a `FLYING` flight of a unit that `CAN_ADV_FLY`, moved by
    /// momentum (`crate::skyriding`).
    pub const ADV_FLYING: u64 = 0x0000_4000_0000_0000;

    /// Whether every bit of `flags` is set.
    pub fn contains(self, flags: u64) -> bool {
        self.0 & flags == flags
    }
}

/// `UnitData::StandState` (TrinityCore `UnitStandStateType`, UnitDefines.h).
#[derive(
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
)]
pub enum StandState {
    #[default]
    Stand = 0,
    Sit = 1,
    SitChair = 2,
    Sleep = 3,
    SitLowChair = 4,
    SitMediumChair = 5,
    SitHighChair = 6,
    Dead = 7,
    Kneel = 8,
    Submerged = 9,
}

impl TryFrom<u8> for StandState {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, u8> {
        Ok(match value {
            0 => Self::Stand,
            1 => Self::Sit,
            2 => Self::SitChair,
            3 => Self::Sleep,
            4 => Self::SitLowChair,
            5 => Self::SitMediumChair,
            6 => Self::SitHighChair,
            7 => Self::Dead,
            8 => Self::Kneel,
            9 => Self::Submerged,
            _ => return Err(value),
        })
    }
}

impl StandState {
    /// `Unit::IsSitState` (Unit.cpp:10787-10794): on the ground or a chair.
    pub fn is_sit(self) -> bool {
        matches!(
            self,
            Self::Sit
                | Self::SitChair
                | Self::SitLowChair
                | Self::SitMediumChair
                | Self::SitHighChair
        )
    }

    /// `Unit::IsStandState` (Unit.cpp:10796-10800): neither sitting, asleep nor
    /// kneeling.
    pub fn is_stand(self) -> bool {
        !self.is_sit() && !matches!(self, Self::Sleep | Self::Kneel)
    }
}

/// A player's `UnitData::StandState`. Creatures carry theirs in `UnitPose`, with the
/// sheath and emote state of their addon, which the server does not model for players.
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
)]
pub struct PlayerStandState(pub StandState);

/// `UnitData::SheatheState` (TrinityCore `SheathState`, UnitDefines.h): which weapons are
/// drawn.
#[derive(
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
)]
pub enum SheathState {
    /// Weapons sheathed.
    #[default]
    Unarmed = 0,
    /// Main and off hand drawn.
    Melee = 1,
    /// Ranged weapon drawn.
    Ranged = 2,
}

impl TryFrom<u8> for SheathState {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, u8> {
        Ok(match value {
            0 => Self::Unarmed,
            1 => Self::Melee,
            2 => Self::Ranged,
            _ => return Err(value),
        })
    }
}

/// A unit's pose (TrinityCore `Creature::LoadCreaturesAddon`): stand state, drawn weapons
/// and `UnitData::EmoteState`, the looping Emotes.db2 ID it plays (0 none).
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
)]
pub struct UnitPose {
    pub stand_state: StandState,
    pub sheath_state: SheathState,
    pub emote_state: u32,
}

/// A creature's `creature_template.rank` (TrinityCore `CreatureClassifications`,
/// SharedDefines.h; world.db keeps AzerothCore's 3 = world boss). Retail's
/// `UnitClassification` token for each is [`Self::token`]. Players carry none.
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub enum CreatureClassification {
    Normal = 0,
    Elite = 1,
    RareElite = 2,
    WorldBoss = 3,
    Rare = 4,
    Trivial = 5,
    MinusMob = 6,
}

impl CreatureClassification {
    /// The classification of a `creature_template.rank`; `None` outside 0..=6.
    pub fn from_rank(rank: u8) -> Option<Self> {
        Some(match rank {
            0 => Self::Normal,
            1 => Self::Elite,
            2 => Self::RareElite,
            3 => Self::WorldBoss,
            4 => Self::Rare,
            5 => Self::Trivial,
            6 => Self::MinusMob,
            _ => return None,
        })
    }

    /// Retail `UnitClassification(unit)`.
    pub fn token(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Elite => "elite",
            Self::RareElite => "rareelite",
            Self::WorldBoss => "worldboss",
            Self::Rare => "rare",
            Self::Trivial => "trivial",
            Self::MinusMob => "minus",
        }
    }
}

/// The `Vignette` DB2 ID a unit shows on the minimap and world map
/// (`creature_template.VignetteID`; TrinityCore `Unit::SetVignette`). Present while
/// the vignette exists: the server removes it on death unless the vignette has
/// `PersistsThroughDeath`, and replication adds and removes it with the unit's
/// visibility, as TrinityCore sends `SMSG_VIGNETTE_UPDATE` Added/Removed.
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct UnitVignette(pub u32);

/// The unit that owns this one (`UF::UnitData::SummonedBy`, set by TrinityCore
/// `Unit::SetOwnerGUID` from `Unit::SetMinion`): a hunter pet carries its owner's
/// entity bits. The owner's client finds its pet (`UnitPet`, `UnitIsUnit("pet", …)`)
/// as the unit summoned by its player.
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct UnitSummonedBy(pub u64);
