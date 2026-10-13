//! On-demand character achievement window data and copy-scoped dungeon objectives.
//! Catalog cursors are exclusive ascending IDs; None means the page is terminal.

use serde::{Deserialize, Serialize};

pub const ACHIEVEMENT_CATEGORY_PAGE_SIZE: usize = 32;
pub const ACHIEVEMENT_PAGE_SIZE: usize = 8;
pub const ACHIEVEMENT_CRITERIA_PAGE_SIZE: usize = 16;
pub const ACHIEVEMENT_NAME_MAX_BYTES: usize = 256;
pub const ACHIEVEMENT_SEARCH_MAX_CHARS: usize = 256;
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

/// Copy-scoped replacement snapshot on InstanceChannel. None clears the scenario tracker.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ScenarioProgress {
    pub map_id: u32,
    pub instance_id: u32,
    pub difficulty_id: u32,
    pub scenario: Option<ScenarioStage>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ScenarioStage {
    pub scenario_id: u32,
    pub name: String,
    /// Retail Scenario.Type, including 3 (dungeon display).
    pub kind: u32,
    /// Retail Scenario.Flags (bit 0x2 suppresses the numbered stage text).
    pub flags: u32,
    pub step_id: u32,
    /// One-based. num_stages + 1 denotes completion.
    pub stage: u32,
    pub num_stages: u32,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub criteria: Vec<AchievementCriterionLine>,
}

/// AchievementChannel, client to server. Each request produces one bounded page.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum QueryAchievementCatalog {
    /// Another online player; nested Comparison requests are rejected by the server.
    Comparison {
        target_entity: u64,
        query: Box<AchievementComparisonQuery>,
    },
    Summary {
        scope: AchievementScope,
    },
    Categories {
        after_id: u32,
    },
    Category {
        category_id: u32,
        after_id: u32,
    },
    Criteria {
        achievement_id: u32,
        after_id: u32,
    },
    Achievement {
        achievement_id: u32,
    },
    Statistics {
        category_id: u32,
        after_id: u32,
    },
    Search {
        text: String,
        scope: AchievementScope,
        after_id: u32,
    },
}

/// Account and character filters refer to the authored 0x20000 flag, not other characters' state.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AchievementScope {
    #[default]
    All,
    Account,
    Character,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AchievementCategoryTotals {
    pub category_id: u32,
    pub total: u32,
    pub completed: u32,
}

/// Full eligible catalog totals, independent of pages opened by the client.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AchievementSummary {
    pub scope: AchievementScope,
    pub points: u32,
    pub total: u32,
    pub completed: u32,
    pub categories: Vec<AchievementCategoryTotals>,
    /// At most three earned rows, newest known date first; unknown dates remain absent.
    pub recent: Vec<AchievementCatalogEntry>,
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
    /// Authentic Achievement.Flags (account 0x20000; statistic 0x1).
    pub flags: u32,
    pub earned: bool,
    /// Unix seconds UTC. Historical earned IDs may have no known date.
    pub earned_at: Option<i64>,
    pub progress_supported: bool,
    pub criteria: Vec<AchievementCriterionLine>,
    pub next_criteria_id: Option<u32>,
    /// Unclamped authoritative counter for supported statistics; None means unevaluated.
    pub statistic_value: Option<u64>,
}

/// AchievementChannel, server to client. Unknown IDs return empty terminal pages.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AchievementCatalogPage {
    /// Target-correlated reply; missing targets return an explicit error, not own progress.
    Comparison {
        target_entity: u64,
        target_name: String,
        page: Option<Box<AchievementComparisonPage>>,
        error: Option<String>,
    },
    Summary(AchievementSummary),
    Statistics {
        category_id: u32,
        achievements: Vec<AchievementCatalogEntry>,
        next_id: Option<u32>,
    },
    Search {
        text: String,
        scope: AchievementScope,
        achievements: Vec<AchievementCatalogEntry>,
        next_id: Option<u32>,
    },
    Achievement {
        achievement_id: u32,
        entry: Option<AchievementCatalogEntry>,
    },
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

/// Nonrecursive ordinary-query projection; the protocol layout tracer forbids recursive messages.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AchievementComparisonQuery {
    Summary {
        scope: AchievementScope,
    },
    Categories {
        after_id: u32,
    },
    Category {
        category_id: u32,
        after_id: u32,
    },
    Criteria {
        achievement_id: u32,
        after_id: u32,
    },
    Achievement {
        achievement_id: u32,
    },
    Statistics {
        category_id: u32,
        after_id: u32,
    },
    Search {
        text: String,
        scope: AchievementScope,
        after_id: u32,
    },
}

/// Nonrecursive ordinary-page projection with unchanged catalog semantics.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AchievementComparisonPage {
    Summary(AchievementSummary),
    Statistics {
        category_id: u32,
        achievements: Vec<AchievementCatalogEntry>,
        next_id: Option<u32>,
    },
    Search {
        text: String,
        scope: AchievementScope,
        achievements: Vec<AchievementCatalogEntry>,
        next_id: Option<u32>,
    },
    Achievement {
        achievement_id: u32,
        entry: Option<AchievementCatalogEntry>,
    },
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

impl TryFrom<QueryAchievementCatalog> for AchievementComparisonQuery {
    type Error = &'static str;
    fn try_from(value: QueryAchievementCatalog) -> Result<Self, Self::Error> {
        Ok(match value {
            QueryAchievementCatalog::Comparison { .. } => {
                return Err("nested achievement comparison is invalid");
            }
            QueryAchievementCatalog::Summary { scope } => {
                AchievementComparisonQuery::Summary { scope }
            }
            QueryAchievementCatalog::Categories { after_id } => {
                AchievementComparisonQuery::Categories { after_id }
            }
            QueryAchievementCatalog::Category {
                category_id,
                after_id,
            } => AchievementComparisonQuery::Category {
                category_id,
                after_id,
            },
            QueryAchievementCatalog::Criteria {
                achievement_id,
                after_id,
            } => AchievementComparisonQuery::Criteria {
                achievement_id,
                after_id,
            },
            QueryAchievementCatalog::Achievement { achievement_id } => {
                AchievementComparisonQuery::Achievement { achievement_id }
            }
            QueryAchievementCatalog::Statistics {
                category_id,
                after_id,
            } => AchievementComparisonQuery::Statistics {
                category_id,
                after_id,
            },
            QueryAchievementCatalog::Search {
                text,
                scope,
                after_id,
            } => AchievementComparisonQuery::Search {
                text,
                scope,
                after_id,
            },
        })
    }
}

impl From<AchievementComparisonQuery> for QueryAchievementCatalog {
    fn from(value: AchievementComparisonQuery) -> Self {
        match value {
            AchievementComparisonQuery::Summary { scope } => {
                QueryAchievementCatalog::Summary { scope }
            }
            AchievementComparisonQuery::Categories { after_id } => {
                QueryAchievementCatalog::Categories { after_id }
            }
            AchievementComparisonQuery::Category {
                category_id,
                after_id,
            } => QueryAchievementCatalog::Category {
                category_id,
                after_id,
            },
            AchievementComparisonQuery::Criteria {
                achievement_id,
                after_id,
            } => QueryAchievementCatalog::Criteria {
                achievement_id,
                after_id,
            },
            AchievementComparisonQuery::Achievement { achievement_id } => {
                QueryAchievementCatalog::Achievement { achievement_id }
            }
            AchievementComparisonQuery::Statistics {
                category_id,
                after_id,
            } => QueryAchievementCatalog::Statistics {
                category_id,
                after_id,
            },
            AchievementComparisonQuery::Search {
                text,
                scope,
                after_id,
            } => QueryAchievementCatalog::Search {
                text,
                scope,
                after_id,
            },
        }
    }
}

impl TryFrom<AchievementCatalogPage> for AchievementComparisonPage {
    type Error = &'static str;
    fn try_from(value: AchievementCatalogPage) -> Result<Self, Self::Error> {
        Ok(match value {
            AchievementCatalogPage::Comparison { .. } => {
                return Err("nested achievement comparison is invalid");
            }
            AchievementCatalogPage::Summary(value) => AchievementComparisonPage::Summary(value),
            AchievementCatalogPage::Statistics {
                category_id,
                achievements,
                next_id,
            } => AchievementComparisonPage::Statistics {
                category_id,
                achievements,
                next_id,
            },
            AchievementCatalogPage::Search {
                text,
                scope,
                achievements,
                next_id,
            } => AchievementComparisonPage::Search {
                text,
                scope,
                achievements,
                next_id,
            },
            AchievementCatalogPage::Achievement {
                achievement_id,
                entry,
            } => AchievementComparisonPage::Achievement {
                achievement_id,
                entry,
            },
            AchievementCatalogPage::Categories {
                categories,
                next_id,
            } => AchievementComparisonPage::Categories {
                categories,
                next_id,
            },
            AchievementCatalogPage::Category {
                category_id,
                achievements,
                next_id,
            } => AchievementComparisonPage::Category {
                category_id,
                achievements,
                next_id,
            },
            AchievementCatalogPage::Criteria {
                achievement_id,
                criteria,
                next_id,
            } => AchievementComparisonPage::Criteria {
                achievement_id,
                criteria,
                next_id,
            },
        })
    }
}

impl From<AchievementComparisonPage> for AchievementCatalogPage {
    fn from(value: AchievementComparisonPage) -> Self {
        match value {
            AchievementComparisonPage::Summary(value) => AchievementCatalogPage::Summary(value),
            AchievementComparisonPage::Statistics {
                category_id,
                achievements,
                next_id,
            } => AchievementCatalogPage::Statistics {
                category_id,
                achievements,
                next_id,
            },
            AchievementComparisonPage::Search {
                text,
                scope,
                achievements,
                next_id,
            } => AchievementCatalogPage::Search {
                text,
                scope,
                achievements,
                next_id,
            },
            AchievementComparisonPage::Achievement {
                achievement_id,
                entry,
            } => AchievementCatalogPage::Achievement {
                achievement_id,
                entry,
            },
            AchievementComparisonPage::Categories {
                categories,
                next_id,
            } => AchievementCatalogPage::Categories {
                categories,
                next_id,
            },
            AchievementComparisonPage::Category {
                category_id,
                achievements,
                next_id,
            } => AchievementCatalogPage::Category {
                category_id,
                achievements,
                next_id,
            },
            AchievementComparisonPage::Criteria {
                achievement_id,
                criteria,
                next_id,
            } => AchievementCatalogPage::Criteria {
                achievement_id,
                criteria,
                next_id,
            },
        }
    }
}
