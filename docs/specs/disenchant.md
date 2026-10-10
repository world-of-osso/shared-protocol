# Disenchant cast target

Disenchant carries an owned bag item GUID separately from unit and ground targets.

## What it must do
- [ ] Round-trip `SpellCastIntent.target_item_guid` without truncation.
- [ ] Append Retail `CantBeDisenchanted`, `CantBeDisenchantedSkill`, and `NotKnown` refusals without reordering existing variants.

## How it works
- Item target is optional; ordinary casts initialize it to `None`. All peers must use the matching protocol fingerprint.

## Implementation inventory
- `src/protocol/core_messages.rs`: item-GUID target.
- `src/spell_data.rs`: Retail refusal variants.

## Tests asserting this spec
- `src/protocol_spell_tests.rs`: `disenchant_item_target_and_refusals_round_trip`.

## Known gaps (current cycle)
- [ ] Targeted wire test pending.

## Out of scope
- Server eligibility, inventory disposal, loot and client cursor behavior live in consuming repositories.
