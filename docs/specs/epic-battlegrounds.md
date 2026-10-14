# Epic battlegrounds

Retail epic queue metadata and reinforcement rules in `src/battleground.rs`, sourced from local BattlemasterList 12.1.0.69933 (FDID1360265/layout3414CBD7), TDB1210 safe locations and TrinityCore a352b1fa9017138cc2f1dcfc0a51afaa17391ecf.

## What it must do

- [x] Alterac Valley queue1/map30 and Isle of Conquest queue30/map628 admit10–40 players per team, levels20–90. Queue3 is not Retail Alterac Valley.
- [x] Start Alterac at700 and Isle at400 reinforcements; no invented elapsed-time victory limit.
- [x] Refuse an under-minimum roster and levels outside the source bracket.
- [x] A player death deducts one reinforcement; depletion ends the match once.

- [x] Alterac's15 authored nodes preserve initial ownership, four-minute contest/defense, neutral Snowfall counter-assault and permanent75-reinforcement tower destruction.
- [x] Alterac source captains cost100 once, generals end the match, and mine leaders change ownership with one reinforcement per45seconds per owned mine.
- [x] Isle's seven source nodes preserve one-minute contest/defense, Quarry/Refinery45-second reinforcement ticks, source WorldStates and245/246 assault/defense columns. Boss deaths award the opposing team victory; live gate breach remains a separate runtime gap.
- [x] Isle's six sourced gates begin closed. An authoritative gate-destruction event flips that gate's open/closed WorldStates and enables its keep banner, without changing reinforcements. Building damage and native siege reachability require separate server/live proof.

## How it works

- [Battleground runtime and source ledger](../../../game-engine/docs/wiki/systems/battlegrounds.md)
- [Server lifecycle contract](../../../game-server/docs/specs/battleground-lifecycle.md)

## Implementation inventory

- `src/battleground.rs` — queue templates, roster eligibility and shared reinforcement state.
- `src/battleground_alterac.rs` — source node/banner/WorldState, captain/general and mine rules.
- `src/battleground_isle.rs` — source seven-node capture, resource/reinforcement and boss-victory rules.
- `src/battleground_columns.rs` — source63/65/61/64/82 scoreboard ordering.
- `src/protocol/interaction_messages.rs` — source AV button/goober use identity.
- `src/battleground_objectives.rs` — existing player-kill reinforcement rule.

## Tests asserting this spec

- `src/battleground_alterac_tests.rs` — neutral counter-assault, timed tower/defense/permanent destruction, graveyards, captains/general and mine resource ticks.
- `src/battleground_tests/templates_and_queue.rs` — source templates, bracket and ten-per-team admission.
- `src/battleground_tests/state_and_objectives.rs` — reinforcement depletion, saturation and kill credit.
- `src/battleground_tests/integration.rs` — Alterac reinforcement-to-victory pipeline.

## Known gaps (current cycle)

- [ ] Alterac normal Opening, source spawn groups and authoritative deaths have server-channel proof; native both-skin Snowfall capture/return is partial live proof, not normal-combat victory.
- [ ] Isle normal Opening/node/spawn-group integration and both-skin Quarry capture/tick/return are implemented; destructible gates, siege vehicles, hangar transport and victory remain open.
- [ ] Both epics' native whole-match victory, preparation, resurrection and reward boundaries remain unproved; source HUD/scoreboard and partial private lifecycle proof are tracked in the linked runtime ledger.
- [ ] Both maps are objective-eligible. Eligibility does not certify canonical asset publication, siege reachability or complete native gameplay.

## Out of scope

Unsourced honor grants and preparation timers must not be invented. Legacy generic reward tests do not certify Retail rewards.
