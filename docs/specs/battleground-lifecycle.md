# Battleground lifecycle

## Contract

Queue → invitation with explicit accept/decline → isolated map entry → authoritative map objective → scoreboard → authoritative honor award → origin return. Extend existing battleground queue/instance/objective models. Modern and Forever share Retail Mainline behavior; Forever changes art only.

Numeric policies and identifiers require local source evidence. Never derive rewards from queue order or substitute existing unverified constants. Missing objective geometry, numeric policies or local assets remain explicit gaps. World imports are lead-owned; world.db is read-only during this work.

## Source-backed end condition

Classic Arathi Basin (map529) ends at1500 resources, not1600. Source: local `game-engine/data/db2/12.1.0.69933/Map.csv`, row529 `PvpLongDescription_lang`: “- Capture and hold the objectives\r\n- Earn 1500 resources”. The shared instance must declare the scoring team winner at1500; subsequent score events cannot change that result.

## Coverage continuation — 2026-10-12

Kotmogu uses Mainline queue 699/map 998, source minimum 2 and maximum 10 per team, levels 40–90 and 1500 victory points (TDB WorldState 17388). Authored dynamically summoned orb templates 212094/212091/212093/212092 are required, rather than fabricated static TDB placements. PvpStat 418 Orb Possessions then 419 Victory Points follow local PvpScoreboardLayout order 0/1. Orb authority permits one carried orb per player; death/departure returns it directly home. Five-second source ticks award 6/4/2 points using the authored small box/medium trigger/outside regions, clamp team points at 1500 and measure PvpStat 419. Both teams reaching the limit on the same tick draws, matching the script's post-tick CheckWinner. Real-channel server/native/live gates and orb combat aura/kill-credit integration remain open; protocol tests do not certify those boundaries.

Silvershard source metadata: queue 708/map 727, minimum 3 and maximum 10 per team, levels 30–90 and 1500 resources. Local PvpStat 428 is **Carts Controlled**, not the pinned script enum's misleading `CartsCaptured` name. Pure cart rules use the twelve authored TDB waypoint paths, source walk rate 2 yd/s, radius-25 one-second control-zone heartbeats, 150 per controlled depot delivery and one resource per displayed controlled cart every two seconds. Source eastern/northern switches 6467/6468 choose the actual route branches; delivered carts respawn after five seconds. PvpStat 428 measures players inside newly controlled zones, not depot deliveries. North has no authored neutral event and retains its displayed faction state through neutrality until the next faction event or depot reset, matching the script/template combination. Real server NPC movement/switch transport, native rendering and private both-skin lifecycle gates remain open.

## Acceptance status — 2026-10-11

Source-backed template correction also maps WSG to Mainline BattlemasterList1014/map2106, teams5–10 and levels10–90. Values decoded from local CASC FDID1360265, build12.1.0.69933, layout3414CBD7; matching wowdev/WoWDBDefs schema supplies field names only, not gameplay numbers. TDB world_state17303 supplies3 captures. `PvpSnapshot.battleground` now carries the retained match identity, authored map/template, team, Waiting phase and acceptance. `RespondToBattlegroundInvite` echoes that identity and explicit accept/decline over the existing PvP channel. Expiry is absent until sourced; callers must not invent a countdown. Other template IDs and policies, durations, node tick/capture policies, deserter duration and reward amounts remain unverified; their presence is not authentic Retail evidence. Existing static marks rewards are not an implementation of Mainline honor.

Server retains matched rosters and validates acceptance/decline against membership and match identity; duplicate, outsider and stale responses fail without changing another participant. Invitation delivery and response systems are connected to existing PvP snapshots. Acceptance remains Waiting/map-entry-pending, never falsely InProgress.

`LeaveBattleground` echoes the current match identity. Server returns only the owning, arrived participant to its retained origin through NewWorld/WorldPortAck; clears membership and closes an unoccupied/unreserved match copy after the last participant. It does not invent early-leave penalty or automatic-exit timers.

Shared FlagObjective.capture now requires the actual carrier and InProgress phase; an invalid/ended attempt cannot consume a flag or mutate score. Typed BG snapshots expose flags, scores, outcome, roster and objective counters using existing objective models. Type36 WSG227740/227741 are usable through the existing world-object contract. Unknown honor reward/level/kill-policy values are absent, not synthetic numbers.

Required but missing: native popup/response/scoreboard UI, durable origin handling across disconnect/restart, real combat counters, sourced honor award and both-skin live proof. Preparation/match/flag-reset/drop/exit/expiry timing and dropped-flag visual fidelity remain explicit gaps. Server multi-player queue/score regression is unit-level proof only, not a live or transport lifecycle test.
