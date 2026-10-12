# Transmog catalogue

- [x] TransmogState separates paid eligible slot choices from the visual-level collected/uncollected/non-collectable catalogue.
- [x] State carries authentic TransmogSet/TransmogSetItem membership, TransmogOutfitEntry/SlotInfo/SlotOption records, saved outfit choices and TransmogIllusion identities, conditions and costs, with each illusion's SpellItemEnchantment name and icon.
- [x] SaveTransmogOutfit saves configuration into an authored outfit entry; it never applies equipment or charges the existing paid Apply quote.
- [x] Player-created Custom Sets remain separate from DB2 outfit entries. SaveTransmogCustomSet creates or updates account presets with server-allocated persisted IDs.
- [x] Illusion IDs and ItemModifiedAppearance source IDs remain separate namespaces. Save failures are explicit and separate from paid Apply failures.

Sources: local Retail12.1.0.69933 DB2; WoWDBDefs layouts C6875C71, E6EFF061, 91D51304, C034B46F, 8C80370F, DCE032AF, SpellItemEnchantment D5D2C264. Retail Blizzard_Transmog defines the shared UI; Forever changes art only.

Proof: `tests/transmog_catalogue.rs` roundtrips catalogue states, DB2 outfit metadata, saved appearance/illusion choices and save refusals using JSON and the live bincode encoder. Consumer eligibility, ownership and durability remain server responsibilities.
