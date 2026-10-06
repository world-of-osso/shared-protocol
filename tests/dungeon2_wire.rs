use serde::{Serialize, de::DeserializeOwned};
use shared::protocol::*;

fn round_trip<T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug>(value: T) {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&value, config).unwrap();
    let (decoded, used): (T, usize) = bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(used, bytes.len());
    assert_eq!(decoded, value);
}

#[test]
fn dungeon2_dungeon_progress_wire_round_trip() {
    round_trip(DungeonProgress {
        map_id: 34,
        instance_id: 42,
        difficulty_id: 1,
        encounters: vec![DungeonEncounterProgress {
            encounter_id: 1144,
            name: "Hogger".into(),
            defeated: true,
            optional: None,
            flags: 4,
        }],
    });
}

#[test]
fn dungeon2_catalog_requests_wire_round_trip() {
    round_trip(QueryAchievementCatalog::Categories { after_id: 0 });
    round_trip(QueryAchievementCatalog::Category {
        category_id: 14808,
        after_id: 632,
    });
    round_trip(QueryAchievementCatalog::Criteria {
        achievement_id: 633,
        after_id: 3386,
    });
}

#[test]
fn dungeon2_catalog_pages_wire_round_trip() {
    let criterion = AchievementCriterionLine {
        tree_id: 3386,
        order_index: 0,
        criteria_id: 18527,
        description: "Hogger".into(),
        current: 1,
        required: 1,
        completed: true,
        progress_supported: true,
    };
    round_trip(AchievementCatalogPage::Categories {
        categories: vec![AchievementCategoryEntry {
            category_id: 14808,
            parent_id: 168,
            order_index: 1,
            name: "Classic".into(),
        }],
        next_id: Some(14808),
    });
    round_trip(AchievementCatalogPage::Category {
        category_id: 14808,
        achievements: vec![AchievementCatalogEntry {
            achievement_id: 633,
            name: "Stormwind Stockade".into(),
            description: "Defeat Hogger.".into(),
            points: 10,
            icon_fdid: 134163,
            earned: true,
            earned_at: Some(1791262800),
            progress_supported: true,
            criteria: vec![criterion.clone()],
            next_criteria_id: None,
        }],
        next_id: None,
    });
    round_trip(AchievementCatalogPage::Criteria {
        achievement_id: 633,
        criteria: vec![criterion],
        next_id: None,
    });
}
