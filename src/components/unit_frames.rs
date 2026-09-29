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
}

/// Client-facing view of one aura on a unit.
#[derive(
    Reflect, Serialize, Deserialize, bitcode::Encode, bitcode::Decode, Debug, Clone, PartialEq,
)]
pub struct AuraView {
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

    pub fn is_selectable(self) -> bool {
        self.0 & Self::NOT_SELECTABLE == 0
    }

    pub fn is_attackable(self) -> bool {
        self.0 & Self::NON_ATTACKABLE_2 == 0
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
/// sets it from each applied `PlayerInput`.
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
pub struct PlayerMotion(pub u32);

impl PlayerMotion {
    pub const FORWARD: u32 = 0x0000_0001;
    pub const BACKWARD: u32 = 0x0000_0002;
    pub const STRAFE_LEFT: u32 = 0x0000_0004;
    pub const STRAFE_RIGHT: u32 = 0x0000_0008;
    /// Walk mode (run toggled off).
    pub const WALKING: u32 = 0x0000_0100;
    /// Airborne: a jump until it lands.
    pub const FALLING: u32 = 0x0000_0800;
    pub const SWIMMING: u32 = 0x0010_0000;

    /// Whether every bit of `flags` is set.
    pub fn contains(self, flags: u32) -> bool {
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
