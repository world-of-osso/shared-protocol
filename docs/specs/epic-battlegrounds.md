# Epic battlegrounds

Retail epic queue metadata and reinforcement rules in `src/battleground.rs`, sourced from local BattlemasterList 12.1.0.69933 (FDID1360265/layout3414CBD7), TDB1210 safe locations and TrinityCore a352b1fa9017138cc2f1dcfc0a51afaa17391ecf.

## What it must do

- [x] Alterac Valley queue1/map30 and Isle of Conquest queue30/map628 admit10–40 players per team, levels20–90. Queue3 is not Retail Alterac Valley.
- [x] Start Alterac at700 and Isle at400 reinforcements; no invented elapsed-time victory limit.
- [x] Refuse an under-minimum roster and levels outside the source bracket.
- [x] A player death deducts one reinforcement; depletion ends the match once.

## How it works

- [Battleground runtime and source ledger](../../../game-engine/docs/wiki/systems/battlegrounds.md)
- [Server lifecycle contract](../../../game-server/docs/specs/battleground-lifecycle.md)

## Implementation inventory

- `src/battleground.rs` — queue templates, roster eligibility and shared reinforcement state.
- `src/battleground_objectives.rs` — existing player-kill reinforcement rule.

## Tests asserting this spec

- `src/battleground_tests/templates_and_queue.rs` — source templates, bracket and ten-per-team admission.
- `src/battleground_tests/state_and_objectives.rs` — reinforcement depletion, saturation and kill credit.
- `src/battleground_tests/integration.rs` — Alterac reinforcement-to-victory pipeline.

## Known gaps (current cycle)

- [ ] Authored nodes, towers, mines, captains, generals, gates, transports and vehicles require map-specific rules and runtime integration.
- [ ] Native HUD/source scoreboard columns and both-skin private real-server lifecycle proof remain pending.
- [ ] Metadata registration alone does not admit either map through server/native objective eligibility.

## Out of scope

Unsourced honor grants and preparation timers must not be invented. Legacy generic reward tests do not certify Retail rewards.
