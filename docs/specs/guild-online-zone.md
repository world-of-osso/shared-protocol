# Guild online member zone

`GuildMemberSnapshot.online_zone` carries an optional server-authoritative label for an online member's zone. Consumers render it in their existing roster/detail fields; no new UI or sorting contract.

## What it must do

- [x] JSON and bincode retain a known label and an unavailable value exactly.
- [ ] The producer uses existing GroundMap→AreaTable zone authority and the real zone-name catalog; offline/unavailable stays None.
- [ ] Protocol and both consumers land together; no old-wire fallback or invented default label.

## How it works

- Server `docs/wiki/systems/guild-lifecycle.md` and engine `docs/wiki/systems/guilds.md` describe their producer/consumer behavior.

## Implementation inventory

- `src/protocol_snapshots.rs`: optional online-zone member field.
- `src/protocol_tests.rs`: concrete JSON/bincode preservation.

## Tests asserting this spec

- `protocol::tests::guild_online_zone_wire_preserves_known_and_unavailable_labels`.

## Known gaps (current cycle)

- [ ] Coordinated protocol/server/engine gate and native proof pending; targeted wire GREEN passed.

## Out of scope

Zone sorting, offline timestamps, mobile/Club presence and map-name/default-label substitutes.
