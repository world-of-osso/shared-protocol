# Skirmish Queue Again request

`RequeueSkirmish` represents the single client action for project policy `SKIRMISH_REQUEUE_LEAVE_THEN_QUEUE_SAME_MODE`. Retail PVPMatchResults.lua490–494 establishes immediate button disable, not the native transition order; `SKIRMISH_REQUEUE_TRANSITION_UNQUALIFIED` remains named.

## What it must do

- [x] Carry only `match_id: u64`; the server derives the played skirmish mode from its authoritative finished match.
- [x] Preserve the scalar identity across JSON and binary serialization, including stale/invalid identities for server-side refusal.
- [x] Register client-to-server without changing existing message registration identities.

## How it works

- A plain serde struct on `PvpChannel`, registered client-to-server after `SetRecipeFavorite`; the server owns all lifecycle meaning (game-server `docs/specs/pvp-progression.md`).

## Implementation inventory

- `src/protocol/gameplay_messages.rs` — match-bound request payload.
- `src/protocol/registration.rs` — appended client-to-server registration.
- `src/protocol_tests.rs` — concrete scalar wire cases.

## Tests asserting this spec

- `src/protocol_tests.rs::skirmish_requeue_wire_carries_only_the_finished_match_identity`.
- game-server `maps::tests::skirmish_requeue` native channel tests send it client-to-server through the registered protocol.

## Known gaps (current cycle)

- None in this repository. Wire test 1/1 at76b415e; server native lifecycle9/9 in its ledger.

## Out of scope

- Server lifecycle/admission implementation and client rendering belong to their respective repositories.
- No client-selected mode, new reward/penalty, rated/BG/Shuffle requeue or guessed Retail native ordering.
