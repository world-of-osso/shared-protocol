//! Retail account Mount Journal. IDs are Mount.db2 IDs, not creature displays.

/// Journal categories, distinct from terrain capability flags.
pub mod mount_journal_types {
    pub const GROUND: u32 = 1;
    pub const FLYING: u32 = 2;
    pub const AQUATIC: u32 = 4;
    pub const SKYRIDING: u32 = 8;
    pub const RIDE_ALONG: u32 = 16;
}
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
    /// World entry replaces the catalog; later updates replace only changed rows.
    pub complete: bool,
    pub mounts: Vec<MountJournalEntry>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MountJournalEntry {
    pub mount_id: u32,
    pub spell_id: u32,
    /// Authentic Retail ItemXItemEffect / ItemEffect TriggerType 6 links.
    pub teaching_items: Vec<u32>,
    pub name: String,
    pub description: String,
    pub source_text: String,
    pub source_type: i32,
    pub icon_file_data_id: u32,
    pub display_id: u32,
    /// Capability flags aggregated from MountTypeXCapability (ground/flying/water).
    pub capability_flags: u32,
    pub journal_types: u32,
    pub flags: u32,
    pub collected: bool,
    pub favourite: bool,
    pub usable: bool,
    pub active: bool,
}
