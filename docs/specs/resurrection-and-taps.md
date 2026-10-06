# Resurrection offers and creature taps

Shared wire contract for player resurrection consent and viewer-relative creature taps. Definitions live in `src/protocol/gameplay_messages.rs`, `src/components/unit_frames.rs` and `src/protocol_snapshots.rs`.

## What it must do

- [x] Server-to-client `ResurrectionOffer` carries caster entity/name, spell and remaining milliseconds on DeathChannel.
- [x] Client-to-server `ResurrectionResponse` identifies the caster/spell offer and acceptance or decline on DeathChannel.
- [x] Replicated `UnitTap` contains stable character IDs; absent/empty means untapped. The viewer or any current group member being a tapper exempts that viewer from denial.
- [x] Group roster entries carry stable character IDs independently of entity visibility/online state.
- [x] Both message directions and tap membership round-trip without loss.

## How it works

- One replicated list is shared by all viewers. Consumers compare it with their selected character ID and current roster; no per-viewer component or continuous damage counter crosses the wire.
- Standard bincode payload cost: one vector-length varint plus ID varints; raw IDs are 8N bytes. Roster identity adds one ID varint per entry on existing roster updates.

## Implementation inventory

- `src/protocol/gameplay_messages.rs`: offer and response messages.
- `src/protocol/registration.rs`: message directions.
- `src/components/unit_frames.rs`: tap component and pure denial predicate.
- `src/protocol.rs`: tap replication registration.
- `src/protocol_snapshots.rs`: roster character identity.

## Tests asserting this spec

- `tests/rezrtap.rs`: offer/response/tap round trips, viewer/group predicate and concrete payload size.
- `src/protocol_group_tests.rs`: roster portrait and stable identity round trip.

## Known gaps (current cycle)

None in the requested wire scope. Targeted offer/tap tests pass 2/2 and roster wire test passes 1/1 at `566d820`; logs `/tmp/claude/rezrtap-protocol-{final,roster}.out`.

## Out of scope

Self-resurrection, pet resurrection, encounter resurrection budgets and compatibility fallbacks. All peers must use the matching protocol schema.
