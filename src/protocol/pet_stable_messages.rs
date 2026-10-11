//! Retail Stable UI: zero-based slots, 5 active + 200 stable (including secondary).
//! Capacity from installed PetConstantsDocumentation.lua; all mutations are owner scoped.
use super::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

pub const PET_ACTIVE_SLOTS: usize = 5;
pub const PET_STABLE_SLOTS: usize = 200;
pub struct PetStableChannel;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PetStableOperation {
    List,
    Swap { from: u16, to: u16 },
    Rename { slot: u16, name: String },
    Abandon { slot: u16 },
    Specialization { slot: u16, specialization: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PetStableRequest {
    pub npc: u64,
    pub operation: PetStableOperation,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StablePetInfo {
    pub slot: u16,
    pub creature_entry: u32,
    pub display_id: u32,
    pub name: String,
    pub level: u8,
    pub family: String,
    pub family_icon: u32,
    pub specialization: Option<u32>,
    pub health: f32,
    pub max_health: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PetSpecializationInfo {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PetStableContents {
    pub npc: u64,
    pub current: Option<u16>,
    pub unlocked_active: [bool; PET_ACTIVE_SLOTS],
    pub pets: Vec<StablePetInfo>,
    pub specializations: Vec<PetSpecializationInfo>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PetStableError {
    NotInteracting,
    HunterOnly,
    InvalidSlot,
    SlotLocked,
    PetNotFound,
    InvalidName,
    InvalidSpecialization,
}

impl PetStableError {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotInteracting => "You must be at a stable master.",
            Self::HunterOnly => "Only hunters can stable pets.",
            Self::InvalidSlot => "Invalid pet slot.",
            Self::SlotLocked => "That Call Pet slot is locked.",
            Self::PetNotFound => "No pet in that slot.",
            Self::InvalidName => "Invalid pet name.",
            Self::InvalidSpecialization => "Invalid pet specialization.",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PetStableFailed {
    pub error: PetStableError,
}

pub(crate) fn register_pet_stable_protocol(app: &mut App) {
    app.add_channel::<PetStableChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(Default::default()),
        ..Default::default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_protocol_message::<PetStableRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<PetStableContents>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<PetStableFailed>()
        .add_direction(NetworkDirection::ServerToClient);
}
