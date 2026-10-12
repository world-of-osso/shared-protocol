# Transmog catalogue

- [x] TransmogState carries paid eligible slot choices (usable collected sources, the only items Retail's Transmogrifier lists) and no item catalogue: every state reply is resent after Apply or a save, and a ~4.8k-source catalogue delayed each reply by ~15 s.
- [x] State carries authentic TransmogSet/TransmogSetItem membership, TransmogOutfitEntry/SlotInfo/SlotOption records, saved outfit choices and TransmogIllusion identities, conditions and costs, with each illusion's SpellItemEnchantment name and icon.
- [x] Each TransmogSet carries one primary appearance per transmog slot with its visual-level collected/uncollected/non-collectable state (collected member first), so set progress covers slots the character has not equipped.
- [x] Outfit records carry the player-selected IconSelector `icon_fdid` as a required wire field; zero is explicitly empty, not an inferred default. Legacy redb records require a versioned server migration.
- [x] SaveTransmogOutfit saves configuration into an authored outfit entry; it never applies equipment or charges the existing paid Apply quote.
- [x] Player-created Custom Sets remain separate from DB2 outfit entries. SaveTransmogCustomSet creates or updates account presets with server-allocated persisted IDs.
- [x] Illusion IDs and ItemModifiedAppearance source IDs remain separate namespaces. Save failures are explicit and separate from paid Apply failures.

Sources: local Retail12.1.0.69933 DB2; WoWDBDefs layouts C6875C71, E6EFF061, 91D51304, C034B46F, 8C80370F, DCE032AF, SpellItemEnchantment D5D2C264. Retail Blizzard_Transmog defines the shared UI; Forever changes art only.

Proof: `tests/transmog_catalogue.rs` roundtrips set-piece collection states, DB2 outfit metadata, saved appearance/illusion choices and save refusals using JSON and the live bincode encoder. Consumer eligibility, ownership and durability remain server responsibilities.
