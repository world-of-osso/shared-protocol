use super::*;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::AppMessageExt;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

fn assert_wire_round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(value, config).unwrap();
    let (decoded, read): (T, usize) = bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(&decoded, value);
}

/// "Kobold Camp Cleanup" (7): kill 8 Kobold Vermin (6), POI blob in Northshire.
fn kobold_camp_cleanup(current: u32) -> QuestEntrySnapshot {
    QuestEntrySnapshot {
        quest_id: 7,
        title: "Kobold Camp Cleanup".into(),
        zone: String::new(),
        completed: false,
        repeatability: QuestRepeatability::Normal,
        objectives: vec![QuestObjectiveSnapshot {
            text: "Kobold Vermin slain".into(),
            current,
            required: 8,
            completed: false,
            kind: QuestObjectiveKind::Monster,
            object_id: 6,
        }],
        level: 2,
        sort_id: 9,
        objectives_text: "Kill 8 Kobold Vermin, then return to Marshal McBride.".into(),
        completion_text: String::new(),
        watched: true,
        pois: vec![QuestPoiSnapshot {
            objective_index: 0,
            map_id: 0,
            world_map_area_id: 30,
            floor: 0,
            priority: 0,
            flags: 1,
            points: vec![
                QuestPoiPoint { x: -8797, y: -259 },
                QuestPoiPoint { x: -8766, y: -253 },
            ],
        }],
    }
}

#[test]
fn quest_log_snapshot_and_update_round_trip() {
    assert_wire_round_trip(&QuestLogSnapshot {
        entries: vec![kobold_camp_cleanup(3)],
        watched_quest_ids: vec![7],
    });
    assert_wire_round_trip(&QuestLogUpdate {
        changed: vec![kobold_camp_cleanup(4)],
        removed: vec![783],
        watched_quest_ids: vec![7],
    });
}

#[test]
fn quest_giver_messages_round_trip() {
    let dagger = QuestRewardItem {
        item_id: 2224,
        name: "Militia Dagger".into(),
        count: 1,
    };
    assert_wire_round_trip(&QuestGiverStatusQuery { npcs: vec![42, 43] });
    assert_wire_round_trip(&QuestGiverStatusMultiple {
        statuses: vec![QuestGiverStatusEntry {
            npc: 42,
            status: QuestGiverStatus::Reward,
        }],
    });
    assert_wire_round_trip(&QuestGiverQuestList {
        npc: 42,
        quests: vec![QuestGiverQuestEntry {
            quest_id: 783,
            title: "A Threat Within".into(),
            level: 1,
            state: QuestGiverQuestState::Available,
        }],
    });
    assert_wire_round_trip(&QuestGiverQuestDetails {
        npc: 42,
        quest_id: 18,
        title: "Brotherhood of Thieves".into(),
        description: "Defias thieves...".into(),
        objectives_text: "Bring 8 Red Burlap Bandanas".into(),
        level: 4,
        min_level: 2,
        suggested_group: 0,
        objectives: kobold_camp_cleanup(0).objectives,
        rewards: QuestRewards {
            money: 0,
            items: vec![],
            choice_items: vec![dagger.clone()],
        },
    });
    assert_wire_round_trip(&QuestGiverChooseReward {
        npc: 42,
        quest_id: 18,
        choice_index: Some(0),
    });
    assert_wire_round_trip(&QuestGiverQuestComplete {
        quest_id: 18,
        money: 0,
        items: vec![dagger],
        xp: 100,
    });
    assert_wire_round_trip(&QuestFailed {
        quest_id: 7,
        reason: QuestFailedReason::DontHaveRequirement,
    });
}

#[test]
fn quest_giver_status_orders_by_marker_priority() {
    let mut statuses = [
        QuestGiverStatus::Available,
        QuestGiverStatus::Reward,
        QuestGiverStatus::Incomplete,
        QuestGiverStatus::None,
    ];
    statuses.sort();
    assert_eq!(
        statuses,
        [
            QuestGiverStatus::None,
            QuestGiverStatus::Incomplete,
            QuestGiverStatus::Available,
            QuestGiverStatus::Reward,
        ]
    );
}

#[test]
fn protocol_plugin_registers_quest_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<QuestLogSnapshot>());
    assert!(app.is_message_registered::<QuestLogUpdate>());
    assert!(app.is_message_registered::<QuestGiverStatusQuery>());
    assert!(app.is_message_registered::<QuestGiverStatusMultiple>());
    assert!(app.is_message_registered::<QuestGiverHello>());
    assert!(app.is_message_registered::<QuestGiverQuestList>());
    assert!(app.is_message_registered::<QuestGiverQueryQuest>());
    assert!(app.is_message_registered::<QuestGiverQuestDetails>());
    assert!(app.is_message_registered::<QuestGiverAcceptQuest>());
    assert!(app.is_message_registered::<QuestGiverCompleteQuest>());
    assert!(app.is_message_registered::<QuestGiverRequestItems>());
    assert!(app.is_message_registered::<QuestGiverOfferReward>());
    assert!(app.is_message_registered::<QuestGiverChooseReward>());
    assert!(app.is_message_registered::<QuestGiverQuestComplete>());
    assert!(app.is_message_registered::<AbandonQuest>());
    assert!(app.is_message_registered::<SetQuestWatched>());
    assert!(app.is_message_registered::<QuestFailed>());
}
