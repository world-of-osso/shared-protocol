# Owned pet journal combat details

`src/protocol/pet_battle_messages.rs` carries server-owned journal combat details and ability-selection decisions. [Server/client journal documentation](../../../game-engine/docs/wiki/systems/battle-pets.md) owns persistence and presentation.

## What it must do
- [x] Preserve full64-bit pet instance IDs, current/max health, power, speed, next-level XP, selected ability IDs and authored ability options through the network codec.
- [x] Preserve one-based ability slot decisions and locked/error acknowledgements through the codec.
- [ ] Route selection requests client→server and owner detail snapshots server→client on the Collections transport.
- [ ] Keep current level/XP in the existing account PetJournal; do not invent unavailable health or ability records.

## How it works
- [Journal authority and presentation](../../../game-engine/docs/wiki/systems/battle-pets.md)

## Implementation inventory
- `src/protocol/pet_battle_messages.rs` — request and detail payload fields.
- `src/protocol/registration.rs` — message directions.

## Tests asserting this spec
- `tests/pet_journal_payload.rs` — actual network-codec round trips with injury, selected choices, a large instance ID and refusal.

## Known gaps (current cycle)
- [ ] Server persistence/validation and client acknowledgement/rendering follow this wire slice.
- [ ] Native owner-only transport proof remains pending.

## Out of scope
- Heal/revive timers and costs: no sourced policy supplied.
