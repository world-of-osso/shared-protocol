# Unit flag fields

Replicated source flag fields for encounter scripts, including TC a352b1fa Ruby Life Pools Executed371652. Definitions: `src/components/unit_frames.rs`.

## What it must do

- [x] Preserve independent32-bit UnitFlags/UnitFlags2/UnitFlags3 fields through encoded payloads.
- [x] Use source bits: PREVENT_EMOTES_FROM_CHAT_TEXT0x20000000, FEIGN_DEATH0x1 in flags2, FAKE_DEAD0x2000 in flags3; retain unrelated bits.
- [ ] Register flags2/flags3 as replicated components alongside flags1. All peers require matching protocol schemas.

## How it works

- [Server hook](../../../game-server/docs/wiki/systems/boss-scripts.md).

## Implementation inventory

- `src/components/unit_frames.rs`: separate raw flag fields and source masks.
- `src/protocol.rs`: replication registration.

## Tests asserting this spec

- `tests/unit_flags.rs`: exact source values and independent payload round trips.
- Server `scripted_ai/tests/data_bosses.rs`: real Executed cast preserves bits without a real death.

## Known gaps (current cycle)

- [ ] Native death-pose/rendering and real-channel delivery are unproved.

## Out of scope

Generic template flags2/flags3 import and new combat or visual behavior inferred from names. This slice carries the source hook's fields, not native visual certification.
