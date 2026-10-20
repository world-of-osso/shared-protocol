# Guild Last Online recency authority

Guild members carry optional actually observed logout time, not opaque display text or guessed history. TC a352b1fa Guild.cpp603/640/681 supplies logout provenance; Retail TimeUtil.lua24–43 defines elapsed recency formatting in consumers.

## What it must do

- [x] GuildMemberSnapshot preserves optional `last_logout_unix_secs` through JSON/bincode. Unknown is None; remove the duplicate `last_online` display-string authority.
- [ ] Protocol field change lands with server and engine consumers, then canonical protocol movement is announced. No old-wire compatibility path.

## How it works

- Consumer `docs/specs/guild-last-online.md` and guild system pages own persistence/presentation.

## Implementation inventory

- `src/protocol_snapshots.rs`: GuildMemberSnapshot.
- `src/protocol_tests.rs`: observed/unknown timestamp wire regression.

## Tests asserting this spec

- `guild_last_online_wire_preserves_observed_timestamp_and_unknown_without_status_text` (ignored RED pushed first).

## Known gaps (current cycle)

- [ ] Actual wire RED observed (missing timestamp after JSON/bincode). Optional authority implemented; focused GREEN, synchronized consumers and coordinated landing pending.

## Out of scope

New presence classes, sort fields, fabricated historical times, leadership/self-policy changes.
