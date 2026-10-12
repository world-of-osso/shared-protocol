//! Retail account Mount Journal. IDs are Mount.db2 IDs, not creature displays.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SetMountFavourite {
    pub mount_id: u32,
    pub favourite: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SummonRandomFavouriteMount;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MountJournalUpdate {
    pub mounts: Vec<MountJournalEntry>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MountJournalEntry {
    pub mount_id: u32,
    pub spell_id: u32,
    pub name: String,
    pub description: String,
    pub source_text: String,
    pub source_type: u32,
    pub icon_file_data_id: u32,
    pub display_id: u32,
    /// Capability flags aggregated from MountTypeXCapability (ground/flying/water).
    pub capability_flags: u32,
    pub collected: bool,
    pub favourite: bool,
    pub usable: bool,
    pub active: bool,
}
