//! Pet control messages: the owner's pet action bar (TrinityCore `SMSG_PET_SPELLS_MESSAGE`,
//! `Player::PetSpellInitialize`, Player.cpp:22440-22477; `SMSG_PET_CLEAR_SPELLS`) and the
//! orders its buttons give (`CMSG_PET_ACTION`, `WorldSession::HandlePetAction`,
//! PetHandler.cpp:64-116) and the autocast toggle (`CMSG_PET_SPELL_AUTOCAST`,
//! `HandlePetSpellAutocastOpcode`, PetHandler.cpp:628-676). All travel on `CombatChannel`.
//!
//! A bar slot is a packed unit action button (`MAKE_UNIT_ACTION_BUTTON`, CharmInfo.h:32-34):
//! the action (a spell ID, a `COMMAND_*` or a `REACT_*`) in the low 24 bits and its
//! `ActiveStates` type in the high 8.

use serde::{Deserialize, Serialize};

/// `MAX_UNIT_ACTION_BAR_INDEX` (CharmInfo.h:85); Retail `NUM_PET_ACTION_SLOTS`.
pub const PET_ACTION_BAR_SLOTS: usize = 10;

/// `ActiveStates` (UnitDefines.h:431-438): the type byte of a packed action button.
pub const ACT_PASSIVE: u8 = 0x01;
pub const ACT_DISABLED: u8 = 0x81;
pub const ACT_ENABLED: u8 = 0xC1;
pub const ACT_COMMAND: u8 = 0x07;
pub const ACT_REACTION: u8 = 0x06;

/// `CommandStates` (UnitDefines.h:461-468).
pub const COMMAND_STAY: u32 = 0;
pub const COMMAND_FOLLOW: u32 = 1;
pub const COMMAND_ATTACK: u32 = 2;
pub const COMMAND_ABANDON: u32 = 3;
pub const COMMAND_MOVE_TO: u32 = 4;

/// `ReactStates` (UnitDefines.h:441-447).
pub const REACT_PASSIVE: u32 = 0;
pub const REACT_DEFENSIVE: u32 = 1;
pub const REACT_AGGRESSIVE: u32 = 2;
pub const REACT_ASSIST: u32 = 3;

/// `MAKE_UNIT_ACTION_BUTTON(action, type)`.
pub const fn pet_action_button(action: u32, kind: u8) -> u32 {
    (action & 0x00FF_FFFF) | ((kind as u32) << 24)
}

/// `UNIT_ACTION_BUTTON_ACTION(packed)`.
pub const fn pet_action_button_action(packed: u32) -> u32 {
    packed & 0x00FF_FFFF
}

/// `UNIT_ACTION_BUTTON_TYPE(packed)`.
pub const fn pet_action_button_type(packed: u32) -> u8 {
    (packed >> 24) as u8
}

/// `SMSG_PET_SPELLS_MESSAGE` to the owner: its pet, the pet's `CommandState` and
/// `ReactState`, and the ten bar slots as packed action buttons (0 empty). Sent when the pet
/// comes out and again whenever an order changes its command or react state.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PetSpells {
    /// Server entity bits of the pet.
    pub pet: u64,
    pub command_state: u32,
    pub react_state: u32,
    pub action_buttons: [u32; PET_ACTION_BAR_SLOTS],
}

/// `SMSG_PET_CLEAR_SPELLS`: the owner no longer has a pet out; its bar goes away.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PetClearSpells;

/// `CMSG_PET_ACTION`: a pet bar button pressed. `action` is the packed button; `target` the
/// unit it acts on (the owner's target for Attack); `position` the ground point of Move To,
/// as `[x, y, z]` of a `Position` (the replicated unit position space).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct PetAction {
    /// Server entity bits of the pet.
    pub pet: u64,
    pub action: u32,
    pub target: Option<u64>,
    pub position: Option<[f32; 3]>,
}

/// `CMSG_PET_SPELL_AUTOCAST` (Retail `TogglePetAutocast`, a right click on a pet spell
/// button): turn autocast of the pet's `spell` on (`ACT_ENABLED`) or off (`ACT_DISABLED`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PetSpellAutocast {
    /// Server entity bits of the pet.
    pub pet: u64,
    pub spell: u32,
    pub enabled: bool,
}
