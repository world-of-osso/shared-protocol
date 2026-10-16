# World-quest availability

[Behavior contract](../../game-engine/docs/specs/world-quests.md). `WorldQuestAvailability` has one `AvailableWorldQuest` per active task/window, with shared expiry and reward preview. Its `pois: Vec<WorldQuestPoi>` preserves every loaded authored area, each carrying world `map_id`, `ui_map_id` and its existing mean-vertex `QuestPoiPoint` representative. These are two-dimensional points, not positions with an implicit zero height.

The prior task-level map/position fields are removed, not retained as alternate paths. Multiple areas do not create duplicate logical quests, watches, completions or rewards. The registered serde-layout fingerprint includes nested field shapes and rejects incompatible peers; server and engine consumers must land together. No compatibility fallback or semantic-epoch-only workaround is added.

Tests: `world_quest_multi_floor_areas_round_trip_as_one_task` preserves task82521's two source UI maps2256/2255; the existing availability roundtrip retains deadline/reward payload coverage. This wire change does not create reward-choice authority or a Retail activation calendar.
