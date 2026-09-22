# Character customization selections

`src/components.rs` defines appearance carried by character creation, roster entries and player replication.

## What it must do

- [x] Retain sex and the six existing core selectors unchanged.
- [x] Carry additional non-core selections as ordered `customization_choices: Vec<CustomizationChoiceSelection>` with stable `option_id: u32` and `choice_id: u32`, without a fixed cap; the count/width fixture preserves 300 entries and IDs above `u16::MAX`.
- [x] Preserve all additional selections through derived bitcode appearance encoding and serde-based creation/roster payload encoding, without manufacturing duplicates.
- [ ] Keep core selectors canonical for their options; clients author additional selections only for other options.

## How it works

- [Server persistence](../../../game-server/docs/wiki/systems/persistence.md) owns stored-record upgrades; the current shared wire shape does not decode obsolete stored schemas.

## Implementation inventory

- `src/components.rs`: appearance and additional selection types.
- `src/protocol/core_messages.rs`: creation and roster carriers.
- `src/protocol_snapshots.rs`: appearance snapshots.

## Tests asserting this spec

- `tests/customization_choices.rs`: appearance bitcode and creation/roster payload roundtrips with nonempty selections.

## Known gaps (current cycle)

- [ ] Engine integration owns option catalog validation and disjoint core/non-core authoring.

## Out of scope

- Live compatibility with clients using the old wire schema; client/server builds must agree.
- Catalog services and account-unlock policy.
