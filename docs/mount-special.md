# Mounted special action

`EmoteIntent`/`EmoteEvent` reuse the existing authenticated actor/event transport for `EmoteKind::MountSpecial`. This is an animation action, not social text or a TextEmotes achievement. Existing enum variants retain their order; the appended variant and component field change the traced connect fingerprint. No existing field meaning changes, so semantic epoch remains unchanged.

`Mounted.mount_id` is the authored Mount.db2 identity selected by the authoritative mount aura. Zero identifies NPC/taxi mounts without a catalog row. Do not recover this identity from `mount_display_id`: multiple Mount rows can use the same creature display but different special kits.

Contract/proof: [engine MountSpecial](../../game-engine/docs/specs/mount-special.md). `tests/mount_special.rs` exercises the actual bincode intent/event and component payloads. Baseline RED: missing enum variant and lost mount identity. Current implementation awaits GREEN.
