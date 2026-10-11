# Group loot

Group-loot messages and roll lifecycle in `src/loot.rs` and `src/protocol/loot_messages.rs`. Server integration belongs to game-server; rendering belongs to game-engine.

## What it must do
- [ ] Keep every eligible member pending, including members unable to Need; reject ineligible Need without consuming their response.
- [ ] Resolve Need before Transmog before Greed; within Transmog, prefer eligible/appropriate classes before comparing RNG (Blizzard Warbands Preview, article24061008). Use injected1–100 values, stable identity tiebreaks, one response per member and no premature winner.
- [ ] Auto-pass unanswered members after 60 seconds; preserve results for loot history and reconnect replay.
- [ ] Carry authoritative item, eligibility reasons, choices, values, winner and delivery status; clients cannot submit roll numbers.
- [ ] Leader-only method/threshold requests and roster policy; active Mainline methods Group Loot and Personal. Legacy enum variants do not establish availability.

## How it works
- Server and engine own their integration documentation.
- Existing roll primitives migrated from retired game-server/crates/shared/src/loot.rs. Fixed old auto-pass-on-cannot-Need behavior.
- Retail GroupLootFrame.lua and LootHistory.lua define UI; Blizzard 10.1 update notes define Transmog priority over Greed. TrinityCore a352b1fa Loot.cpp defines vote timeout and direct delivery.

## Implementation inventory
- `src/loot.rs`: roll/window rules.
- `src/protocol/loot_messages.rs`: group choice, history request and roll snapshot.
- `src/protocol/group_messages.rs`: leader policy intents.
- `src/protocol_snapshots.rs`: roster policy.

## Tests asserting this spec
- `tests/group_loot.rs`

## Integration proof — 2026-10-11

Server and native engine branches pin protocol revision `c6e063c37d647c08661af16770ded3589eec23f4`. Its six deterministic tests pass; server-only Transmog class preference adds no wire fields. Ordered real UDP snapshot/choice test passes; two private54300 accounts prove actual inventory deltas, numeric results and history in Modern and Forever. Server deterministic Transmog1 > Greed99 test also replays completed history on a replaced authenticated connection. [Consumer proof ledger](../../../game-engine/docs/wiki/systems/group-loot.md).

## Known gaps (current cycle)
- [ ] Independent final integration verification; verifier backend authentication expired.

## Out of scope
- Master loot; retired distribution policies.
