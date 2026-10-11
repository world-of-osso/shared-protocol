# Group loot

Group-loot messages and roll lifecycle in `src/loot.rs` and `src/protocol/loot_messages.rs`. Server integration belongs to game-server; rendering belongs to game-engine.

## What it must do
- [ ] Keep every eligible member pending, including members unable to Need; reject ineligible Need without consuming their response.
- [ ] Resolve Need before Transmog before Greed, with injected 1–100 roll values, stable identity tiebreaks, one response per member, and no premature winner.
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

## Known gaps (current cycle)
- [ ] Server and engine integration/live proof.

## Out of scope
- Master loot; retired distribution policies.
