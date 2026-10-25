//! Realm-backed Mainline Suggested Content and Traveler's Log.
//! Unsupported criteria are explicitly unavailable; never encoded as zero progress.
use super::{AchievementCriterionLine};
#[cfg(feature = "transport")]
use super::ProtocolRegistrationExt;
use bevy::prelude::*;
#[cfg(feature = "transport")]
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AdventureGuideRequest {
    Query,
    TrackActivity { activity_id: u32, tracked: bool },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct JournalMonth {
    pub year: i32,
    pub month: u32,
    pub name: String,
    pub start_unix: i64,
    pub end_unix: i64,
    pub now_unix: i64,
    /// Actual start of the realm's monthly observation stream, not invented history.
    pub tracking_since: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PerksActivityProgress {
    Available {
        current: u64,
        required: u64,
        completed: bool,
        criteria: Vec<AchievementCriterionLine>,
    },
    Unavailable {
        reason: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PerksActivitySnapshot {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub points: u32,
    pub supersedes: u32,
    pub priority: i32,
    pub tags: Vec<u32>,
    pub tracked: bool,
    pub progress: PerksActivityProgress,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PerksActivityTag {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AdventureSuggestion {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub reward_description: String,
    pub texture_fdid: u32,
    /// Display-only suggestions have no activation button.
    pub button_text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct SuggestionCensus {
    pub source_rows: u32,
    pub unsupported_conditions: u32,
    pub failed_conditions: u32,
    pub unsupported_actions: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AdventureGuideSnapshot {
    pub month: JournalMonth,
    pub activities: Vec<PerksActivitySnapshot>,
    pub tags: Vec<PerksActivityTag>,
    pub suggestions: Vec<AdventureSuggestion>,
    pub suggestion_census: SuggestionCensus,
    /// Sum of completed source contributions, not a fabricated currency/reward grant.
    pub points: u32,
    /// Source interval fields have no verified calendar mapping: policy may show all rows.
    pub activity_scope_notice: Option<String>,
    /// Missing source catalog/bindings, not fabricated empty content.
    pub source_error: Option<String>,
}

/// Clock-only update after an initial full Guide, ordered on `InstanceChannel`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AdventureGuideClock {
    pub now_unix: i64,
}

#[cfg(feature = "transport")]
pub(super) fn register(app: &mut App) {
    app.register_protocol_message::<AdventureGuideRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<AdventureGuideSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<AdventureGuideClock>()
        .add_direction(NetworkDirection::ServerToClient);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unavailable_activity_wire_state_does_not_become_zero_progress() {
        let activity = PerksActivitySnapshot {
            id: 3,
            name: "Complete 30 Quests".into(),
            description: "Complete quests that grant experience.".into(),
            points: 200,
            supersedes: 0,
            priority: 0,
            tags: vec![],
            tracked: true,
            progress: PerksActivityProgress::Unavailable {
                reason: "Criteria tree 135693 contains untracked criteria".into(),
            },
        };
        let bytes = serde_json::to_vec(&activity).unwrap();
        let decoded: PerksActivitySnapshot = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, activity);
        assert!(matches!(
            decoded.progress,
            PerksActivityProgress::Unavailable { .. }
        ));
    }
}
