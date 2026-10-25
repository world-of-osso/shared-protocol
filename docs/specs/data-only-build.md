# Data-only protocol builds

The canonical shared records and payloads remain usable by headless combat consumers without network registration.

## What it must do

- [ ] `default-features = false` retains the same component, catalog and message types without Lightyear or Replicon dependencies.
- [ ] Default builds retain protocol registration, channel registration and handshake behavior.
- [ ] Feature selection does not change payload fields or encodings.

## How it works

- [Data-only protocol boundary](../wiki/systems/data-only-build.md).

## Implementation inventory

- `Cargo.toml`: optional transport dependencies and default feature.
- `src/lib.rs`: conditional protocol-plugin export.
- `src/protocol.rs`: conditional registration and transport tests.
- `src/protocol/*_messages.rs`, `src/protocol_snapshots.rs`: unconditional records with conditional registration.

## Tests asserting this spec

- Existing `src/protocol_spell_tests.rs` and other protocol tests retain default-build registration and wire assertions.
- Data-only library compilation asserts the transport-free API boundary.

## Known gaps (current cycle)

No remaining API-compilation gap in this change.

## Out of scope

- Changes to game records, protocol values or network behavior.
