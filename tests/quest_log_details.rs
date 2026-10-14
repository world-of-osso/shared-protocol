use shared::protocol::QuestLogSnapshot;

#[test]
fn owned_quest_snapshot_keeps_story_rewards_and_pushability_across_wire_round_trip() {
    let wire = serde_json::json!({
        "entries": [{
            "quest_id": 28766, "title": "Beating Them Back!", "zone": "",
            "completed": false, "expires_at": null, "timer_sampled_at": null,
            "failed": false, "repeatability": "Normal", "objectives": [],
            "level": -1, "sort_id": 9,
            "objectives_text": "Kill the attacking Blackrock worgs!",
            "description": "I'm Marshal McBride, commander of this garrison. Welcome, $n.",
            "rewards": {"money": 2000, "items": [], "choice_items": []},
            "pushable": false, "completion_text": "", "watched": true, "pois": []
        }],
        "watched_quest_ids": [28766]
    });
    let snapshot: QuestLogSnapshot = serde_json::from_value(wire.clone()).unwrap();
    let bytes = bincode::serde::encode_to_vec(&snapshot, bincode::config::standard()).unwrap();
    let (decoded, consumed): (QuestLogSnapshot, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(consumed, bytes.len());
    assert_eq!(serde_json::to_value(decoded).unwrap(), wire);
}
