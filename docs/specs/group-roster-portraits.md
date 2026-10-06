# Group roster portraits

The group roster carries enough character appearance to build a member's head without a replicated entity. Protocol source: `src/protocol_snapshots.rs`. Server lifecycle and client integration: game-server `docs/group-roster-portraits.md`.

## What it must do

- [x] `GroupMemberSnapshot.portrait` carries `race`, canonical `appearance: CharacterAppearance`, and `head: Option<EquippedAppearanceEntry>`.
- [x] Serialization preserves sex, core face/skin/eye/hair/facial selectors, stable customization option/choice IDs, alternate-form selections, and head item/display/hidden state.
- [x] Every roster includes each member's last known portrait, including offline members, independent of zone or replication interest.
- [x] Joining, relogging, changing character appearance, or changing/removing/hiding head equipment sends a roster; unchanged portraits and non-head equipment do not cause periodic portrait messages.

## How it works

- See game-server `docs/group-roster-portraits.md` for lifecycle, renderer evidence and exact client hookup.

## Implementation inventory

- `src/protocol_snapshots.rs`: `GroupPortraitAppearance` and roster field.
- `src/protocol_group_tests.rs`: concrete roster serialization round trip.

## Tests asserting this spec

- `src/protocol_group_tests.rs::roster_portrait_round_trips_on_the_wire`.
- game-server `crates/server/src/group/tests.rs`: `roster_portrait_*` loopback delivery tests.
- Verified 2026-10-06: protocol `e0a6d19` round trip passes (1/1); server `cb40477` roster fixtures pass (4/4), including actual persisted head-item equip through inventory networking. Both use the locked, capped local server-build helper.

## Known gaps (current cycle)

- [ ] Client hookup is intentionally deferred; changing this required field requires protocol and consumers to update together.

## Out of scope

- Client rendering changes, full-body/weapon equipment, group persistence across server restart, and new appearance editing or transmog APIs. Existing groups are in-memory and disband when no members remain online.
