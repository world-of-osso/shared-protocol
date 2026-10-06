# Dungeon and achievement window data

Phase-2 wire contract in `src/protocol/achievement_catalog_messages.rs`. Server runtime, bounds enforcement and client hookup are documented by game-server `docs/specs/dungeon-achievements.md` and `docs/dungeon-achievements-client.md`.

## What it must do

- [x] Round-trip DungeonProgress copy identity, ordered named encounters, defeated state, unknown optionality and raw flags.
- [x] Round-trip each QueryAchievementCatalog variant and response variant, including continuation IDs, category parents, earned UTC dates and u64 criterion counters.
- [x] Register DungeonProgress server-to-client on InstanceChannel; catalog request/response on AchievementChannel.
- [x] Bound server catalog pages to 32 categories / 8 achievements / 16 criterion lines; metadata names/criterion descriptions 256 bytes and achievement descriptions 2048 bytes.

## How it works

- Existing bidirectional reliable ordered InstanceChannel and AchievementChannel carry the added message types.
- Cursors are exclusive ascending IDs; None is terminal. Criteria continue independently of category achievement pages.
- Historical dates and optionality are optional values; unknown is never fabricated.

## Implementation inventory

- `src/protocol/achievement_catalog_messages.rs` — messages and fixed bounds.
- `src/protocol/registration.rs` — wire layout/direction registration.
- `src/protocol.rs` — public exports.

## Tests asserting this spec

- `tests/dungeon2_wire.rs` — bincode/lightyear encoder round trips.
- Server's Stockade host fixture exercises actual message registration/delivery.

## Known gaps (current cycle)

Four targeted wire tests pass at `f115ea5`; the maximum category page is 56,397 bytes. Server host fixtures exercise actual delivery and pagination. Client implementation follows separately; no optional DB2 flag interpretation is invented.

## Out of scope

Client rendering, account/guild achievement evaluation and every Retail scenario API.
