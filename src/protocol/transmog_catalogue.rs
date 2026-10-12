//! DB2-authored wardrobe data; these identities never substitute for paid
//! ItemModifiedAppearance choices. Collection ownership is visual-level.
use super::{EquipmentSlot, TransmogChoice, TransmogSource};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransmogCollectionState {
    Collected,
    Uncollected,
    NonCollectable,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogCatalogueSource {
    pub source: TransmogSource,
    pub collection: TransmogCollectionState,
    pub slots: Vec<EquipmentSlot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogSet {
    pub set_id: u32,
    pub parent_set_id: u32,
    pub name: String,
    pub source_ids: Vec<u32>,
}

/// Authored TransmogOutfitEntry, not an invented player outfit identifier.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogOutfitEntry {
    pub id: u32,
    pub name: String,
    pub order_index: u32,
    pub cost: u64,
    pub source: u8,
    pub flags: u32,
    pub set_type: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogOutfitChoice {
    pub slot: EquipmentSlot,
    pub choice: TransmogChoice,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogOutfitIllusion {
    pub slot: EquipmentSlot,
    /// TransmogIllusion.ID, never an ItemModifiedAppearance.ID.
    pub illusion_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogOutfit {
    pub id: u32,
    pub name: String,
    pub choices: Vec<TransmogOutfitChoice>,
    pub illusions: Vec<TransmogOutfitIllusion>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogIllusion {
    pub id: u32,
    pub enchantment_id: u32,
    /// SpellItemEnchantment.Name_lang and IconFileDataID (C_TransmogCollection.GetIllusionInfo).
    pub name: String,
    pub icon_fdid: u32,
    pub cost: u64,
    pub unlock_condition_id: u32,
    pub flags: u32,
    pub collected: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogOutfitSlot {
    pub id: u32,
    pub slot: EquipmentSlot,
    pub flags: u32,
    pub collection_type: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogOutfitOption {
    pub id: u32,
    pub slot_info_id: u32,
    pub name: String,
    pub option: u8,
    pub flags: u32,
}

/// Player-created presets are separate from DB2-authored outfit entries.
/// IDs are allocated and persisted by the server, never fabricated DB2 IDs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogCustomSet {
    pub id: u64,
    pub name: String,
    pub choices: Vec<TransmogOutfitChoice>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SaveTransmogCustomSet {
    pub npc: u64,
    pub id: Option<u64>,
    pub name: String,
    pub choices: Vec<TransmogOutfitChoice>,
}

/// Saving never applies equipment, binds items, or charges the paid Apply quote.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SaveTransmogOutfit {
    pub npc: u64,
    pub outfit: TransmogOutfit,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransmogOutfitError {
    NotInteracting,
    EntryUnavailable,
    InvalidName,
    InvalidChoice,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogOutfitFailed {
    pub npc: u64,
    pub error: TransmogOutfitError,
}
