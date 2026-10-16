# World-quest availability

[Behavior contract](../../game-engine/docs/specs/world-quests.md). `WorldQuestAvailability` has one `AvailableWorldQuest` per active task/window, with shared expiry and reward preview. Its `pois: Vec<WorldQuestPoi>` preserves every loaded authored area, each carrying world `map_id`, `ui_map_id` and explicit `x`, `y`, `z` from the componentwise mean of the source area's QuestPOIPoint vertices. Source height is mandatory; the importer and wire must not replace it with zero.

The prior task-level map/XY-position fields are removed, not retained as alternate paths. The runtime area table must be rebuilt by the importer to include source Z; old XY-only data fails loading rather than supplying a fallback height. Multiple areas do not create duplicate logical quests, watches, completions or rewards. The registered serde-layout fingerprint includes nested field shapes and rejects incompatible peers; server and engine consumers must land together. No compatibility fallback or semantic-epoch-only workaround is added.

Tests: `world_quest_multi_floor_areas_round_trip_as_one_task` preserves task82521's lower/main UI maps2256/2255 and negative source heights; the existing availability roundtrip retains deadline/reward payload coverage. This wire change does not create reward-choice authority or a Retail activation calendar.
