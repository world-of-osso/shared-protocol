//! On-demand character achievement window data and copy-scoped dungeon objectives.
//! Catalog cursors are exclusive ascending IDs; None means the page is terminal.

use serde::{Deserialize, Serialize};

pub const ACHIEVEMENT_CATEGORY_PAGE_SIZE: usize = 32;
pub const ACHIEVEMENT_PAGE_SIZE: usize = 8;
pub const ACHIEVEMENT_CRITERIA_PAGE_SIZE: usize = 16;
pub const ACHIEVEMENT_NAME_MAX_BYTES: usize = 256;
pub const ACHIEVEMENT_DESCRIPTION_MAX_BYTES: usize = 2048;
pub const DUNGEON_ENCOUNTER_MAX_ROWS: usize = 32;

/// Complete copy snapshot on InstanceChannel, including an empty list outside instances.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DungeonProgress {
    pub map_id: u32,
    pub instance_id: u32,
    pub difficulty_id: u32,
    pub encounters: Vec<DungeonEncounterProgress>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DungeonEncounterProgress {
    pub encounter_id: u32,
    pub name: String,
    pub defeated: bool,
    /// None when the source has no verified optionality semantics.
    pub optional: Option<bool>,
    pub flags: u32,
}

/// AchievementChannel, client to server. Each request produces one bounded page.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryAchievementCatalog {
    Categories { after_id: u32 },
    Category { category_id: u32, after_id: u32 },
    Criteria { achievement_id: u32, after_id: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AchievementCategoryEntry {
    pub category_id: u32,
    /// -1 denotes a root category.
    pub parent_id: i32,
    pub order_index: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AchievementCriterionLine {
    pub tree_id: u32,
    pub order_index: i32,
    pub criteria_id: u32,
    pub description: String,
    pub current: u64,
    pub required: u64,
    pub completed: bool,
    /// False is unevaluated, not a completed zero-target criterion.
    pub progress_supported: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AchievementCatalogEntry {
    pub achievement_id: u32,
    pub name: String,
    pub description: String,
    pub points: u32,
    pub icon_fdid: u32,
    pub earned: bool,
    /// Unix seconds UTC. Historical earned IDs may have no known date.
    pub earned_at: Option<i64>,
    pub progress_supported: bool,
    pub criteria: Vec<AchievementCriterionLine>,
    pub next_criteria_id: Option<u32>,
}

/// AchievementChannel, server to client. Unknown IDs return empty terminal pages.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AchievementCatalogPage {
    Categories {
        categories: Vec<AchievementCategoryEntry>,
        next_id: Option<u32>,
    },
    Category {
        category_id: u32,
        achievements: Vec<AchievementCatalogEntry>,
        next_id: Option<u32>,
    },
    Criteria {
        achievement_id: u32,
        criteria: Vec<AchievementCriterionLine>,
        next_id: Option<u32>,
    },
}
