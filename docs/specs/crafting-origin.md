# Crafting origin

Owner item instances carry server-recorded crafting provenance. TC a352b1fa `ItemDefines.h:264–283` distinguishes modifier39 SkillLineAbilityID from recipe SpellID; Retail `Professions.lua:378–388` resolves the original recipe from the owned GUID. [Engine contract](../../../game-engine/docs/specs/modern-crafting.md) owns the vertical slice.

## What it must do

- [ ] Existing ItemStack owner messages preserve additive optional `crafting_origin`: authored skill-line-ability identity, its joined recipe spell and an optional actual known crafting quality ID.
- [ ] Ability1355 and recipe2153 remain distinct through JSON and binary serialization.
- [ ] Missing origin remains unknown. Unknown crafting quality is not rarity, zero or a default tier.
- [ ] Unknown origin serializes as explicit JSON `null`, like `durability`; bincode stays positional, so the field is never skipped.

## How it works

- [Native crafting](../../../game-engine/docs/wiki/systems/native-professions-crafting.md).

## Implementation inventory

- `src/protocol/inventory_messages.rs` — owner item instance and origin fields.

## Tests asserting this spec

- `tests/crafting_origin.rs` — owner serialization preserves concrete identities and unknown quality.
- `tests/item_definition_source.rs` — exact owner JSON includes `crafting_origin: null` after bincode round trips.

## Known gaps (current cycle)

- [ ] Atomic protocol/server/engine integration proof.

## Out of scope

Recraft eligibility, operation evaluation, slots and skill watermark. No client-originated lineage writes. Land protocol changes only with matching server and engine consumers.
