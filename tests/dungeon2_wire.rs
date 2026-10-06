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

#[test]
fn dungeon2_maximum_category_payload_is_below_64_kib() {
    let criterion = AchievementCriterionLine {
        tree_id: u32::MAX,
        order_index: i32::MIN,
        criteria_id: u32::MAX,
        description: "x".repeat(ACHIEVEMENT_NAME_MAX_BYTES),
        current: u64::MAX,
        required: u64::MAX,
        completed: true,
        progress_supported: false,
    };
    let achievement = AchievementCatalogEntry {
        achievement_id: u32::MAX,
        name: "x".repeat(ACHIEVEMENT_NAME_MAX_BYTES),
        description: "x".repeat(ACHIEVEMENT_DESCRIPTION_MAX_BYTES),
        points: u32::MAX,
        icon_fdid: u32::MAX,
        earned: true,
        earned_at: Some(i64::MAX),
        progress_supported: false,
        criteria: vec![criterion; ACHIEVEMENT_CRITERIA_PAGE_SIZE],
        next_criteria_id: Some(u32::MAX),
    };
    let page = AchievementCatalogPage::Category {
        category_id: u32::MAX,
        achievements: vec![achievement; ACHIEVEMENT_PAGE_SIZE],
        next_id: Some(u32::MAX),
    };
    let bytes = bincode::serde::encode_to_vec(&page, bincode::config::standard()).unwrap();
    println!("maximum category wire bytes: {}", bytes.len());
    assert!(bytes.len() < 64 * 1024);
    round_trip(page);
}
