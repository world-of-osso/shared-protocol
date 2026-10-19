# Guild online member zone

`GuildMemberSnapshot.online_zone` carries an optional server-authoritative label for an online member's zone. Consumers render it in their existing roster/detail fields; no new UI or sorting contract.

## What it must do

- [x] JSON and bincode retain a known label and an unavailable value exactly.
- [x] The producer uses existing GroundMap→AreaTable zone authority and real names loaded with AreaTable; offline/unavailable stays None.
- [x] Protocol and both consumers land together; no old-wire fallback or invented default label.

## How it works

- Server `docs/wiki/systems/guild-lifecycle.md` and engine `docs/wiki/systems/guilds.md` describe their producer/consumer behavior.

## Implementation inventory

- `src/protocol_snapshots.rs`: optional online-zone member field.
- `src/protocol_tests.rs`: concrete JSON/bincode preservation.

## Tests asserting this spec

- `protocol::tests::guild_online_zone_wire_preserves_known_and_unavailable_labels`.

## Known gaps (current cycle)

- [x] Targeted wire GREEN and coordinated gz689 passed; protocol b3a34fe/server e3f3f86c6/engine d802433d9 landed together. Accepted two authenticated members prove real-zone display, refresh and normal disconnect in both skins; engine `docs/wiki/systems/guilds.md#online-member-zone` owns native artifact/PNG provenance and proof limits.

## Out of scope

Zone sorting, offline timestamps, mobile/Club presence and map-name/default-label substitutes.
