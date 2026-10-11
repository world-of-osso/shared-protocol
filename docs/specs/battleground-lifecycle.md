# Battleground lifecycle

## Contract

Queue → invitation with explicit accept/decline → isolated map entry → authoritative map objective → scoreboard → authoritative honor award → origin return. Extend existing battleground queue/instance/objective models. Modern and Forever share Retail Mainline behavior; Forever changes art only.

Numeric policies and identifiers require local source evidence. Never derive rewards from queue order or substitute existing unverified constants. Missing objective geometry, numeric policies or local assets remain explicit gaps. World imports are lead-owned; world.db is read-only during this work.

## Source-backed end condition

Classic Arathi Basin (map529) ends at1500 resources, not1600. Source: local `game-engine/data/db2/12.1.0.69933/Map.csv`, row529 `PvpLongDescription_lang`: “- Capture and hold the objectives\r\n- Earn 1500 resources”. The shared instance must declare the scoring team winner at1500; subsequent score events cannot change that result.

## Acceptance status — 2026-10-11

Source-backed template correction also maps WSG to Mainline BattlemasterList1014/map2106, teams5–10 and levels10–90. Values decoded from local CASC FDID1360265, build12.1.0.69933, layout3414CBD7; matching wowdev/WoWDBDefs schema supplies field names only, not gameplay numbers. TDB world_state17303 supplies3 captures. `PvpSnapshot.battleground` now carries the retained match identity, authored map/template, team, Waiting phase and acceptance. `RespondToBattlegroundInvite` echoes that identity and explicit accept/decline over the existing PvP channel. Expiry is absent until sourced; callers must not invent a countdown. Other template IDs and policies, durations, node tick/capture policies, deserter duration and reward amounts remain unverified; their presence is not authentic Retail evidence. Existing static marks rewards are not an implementation of Mainline honor.

Server retains matched rosters and validates acceptance/decline against membership and match identity; duplicate, outsider and stale responses fail without changing another participant. Invitation delivery and response systems are connected to existing PvP snapshots. Acceptance remains Waiting/map-entry-pending, never falsely InProgress.

`LeaveBattleground` echoes the current match identity. Server returns only the owning, arrived participant to its retained origin through NewWorld/WorldPortAck; clears membership and closes an unoccupied/unreserved match copy after the last participant. It does not invent early-leave penalty or automatic-exit timers.

Shared FlagObjective.capture now requires the actual carrier and InProgress phase; an invalid/ended attempt cannot consume a flag or mutate score. Typed BG snapshots expose flags, scores, outcome, roster and objective counters using existing objective models. Type36 WSG227740/227741 are usable through the existing world-object contract. Unknown honor reward/level/kill-policy values are absent, not synthetic numbers.

Required but missing: native popup/response/scoreboard UI, durable origin handling across disconnect/restart, real combat counters, sourced honor award and both-skin live proof. Preparation/match/flag-reset/drop/exit/expiry timing and dropped-flag visual fidelity remain explicit gaps. Server multi-player queue/score regression is unit-level proof only, not a live or transport lifecycle test.
