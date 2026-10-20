# Skirmish Queue Again request

`RequeueSkirmish` represents the single client action for project policy `SKIRMISH_REQUEUE_LEAVE_THEN_QUEUE_SAME_MODE`. Retail PVPMatchResults.lua490–494 establishes immediate button disable, not the native transition order; `SKIRMISH_REQUEUE_TRANSITION_UNQUALIFIED` remains named. [Protocol mechanics](../wiki/systems/skirmish-requeue.md).

## What it must do

- [ ] Carry only `match_id: u64`; the server derives the played skirmish mode from its authoritative finished match.
- [ ] Preserve the scalar identity across JSON and binary serialization, including stale/invalid identities for server-side refusal.
- [ ] Register client-to-server without changing existing message registration identities.

## How it works

- [Protocol mechanics](../wiki/systems/skirmish-requeue.md).

## Implementation inventory

- `src/protocol/gameplay_messages.rs` — match-bound request payload.
- `src/protocol/registration.rs` — appended client-to-server registration.
- `src/protocol_tests.rs` — concrete scalar wire cases.

## Tests asserting this spec

- `src/protocol_tests.rs::skirmish_requeue_wire_carries_only_the_finished_match_identity`.
- Server native channel tests will exercise registration and direction with the actual lifecycle.

## Known gaps (current cycle)

- [ ] Wire and native request proof pending; payload/registration initially scaffold the server RED tests after observed UI RED.

## Out of scope

- Server lifecycle/admission implementation and client rendering belong to their respective repositories.
- No client-selected mode, new reward/penalty, rated/BG/Shuffle requeue or guessed Retail native ordering.
