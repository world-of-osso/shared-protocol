# Epic battlegrounds

Retail epic queue metadata and reinforcement rules in `src/battleground.rs`, sourced from local BattlemasterList 12.1.0.69933 (FDID1360265/layout3414CBD7), TDB1210 safe locations and TrinityCore a352b1fa9017138cc2f1dcfc0a51afaa17391ecf.

## What it must do

- [x] Alterac Valley queue1/map30 and Isle of Conquest queue30/map628 admit10–40 players per team, levels20–90. Queue3 is not Retail Alterac Valley.
- [x] Start Alterac at700 and Isle at400 reinforcements; no invented elapsed-time victory limit.
- [x] Refuse an under-minimum roster and levels outside the source bracket.
- [x] A player death deducts one reinforcement; depletion ends the match once.

- [x] Alterac's15 authored nodes preserve initial ownership, four-minute contest/defense, neutral Snowfall counter-assault and permanent75-reinforcement tower destruction.
- [x] Alterac source captains cost100 once, generals end the match, and mine leaders change ownership with one reinforcement per45seconds per owned mine.

## How it works

- [Battleground runtime and source ledger](../../../game-engine/docs/wiki/systems/battlegrounds.md)
- [Server lifecycle contract](../../../game-server/docs/specs/battleground-lifecycle.md)

## Implementation inventory

- `src/battleground.rs` — queue templates, roster eligibility and shared reinforcement state.
- `src/battleground_alterac.rs` — source node/banner/WorldState, captain/general and mine rules.
- `src/battleground_columns.rs` — source63/65/61/64/82 scoreboard ordering.
- `src/protocol/interaction_messages.rs` — source AV button/goober use identity.
- `src/battleground_objectives.rs` — existing player-kill reinforcement rule.

## Tests asserting this spec

- `src/battleground_alterac_tests.rs` — neutral counter-assault, timed tower/defense/permanent destruction, graveyards, captains/general and mine resource ticks.
- `src/battleground_tests/templates_and_queue.rs` — source templates, bracket and ten-per-team admission.
- `src/battleground_tests/state_and_objectives.rs` — reinforcement depletion, saturation and kill credit.
- `src/battleground_tests/integration.rs` — Alterac reinforcement-to-victory pipeline.

## Known gaps (current cycle)

- [ ] Alterac node/tower/mine/captain/general pure rules need normal eight-second Opening casts, source prefab transitions and authoritative combat-death integration. Initial entry is ground-based, not transport-relative.
- [ ] Isle nodes, gates, transports and vehicles still require map-specific rules and runtime integration.
- [ ] Native HUD/source scoreboard columns and both-skin private real-server lifecycle proof remain pending.
- [ ] Alterac source objective registration and button/goober identity now feed server/native eligibility; native rendering/HUD and live proof must be completed before publication. Isle remains objective-ineligible.

## Out of scope

Unsourced honor grants and preparation timers must not be invented. Legacy generic reward tests do not certify Retail rewards.
