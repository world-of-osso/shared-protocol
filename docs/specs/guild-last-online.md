# Guild Last Online recency authority

Guild members carry optional actually observed logout time, not opaque display text or guessed history. TC a352b1fa Guild.cpp603/640/681 supplies logout provenance; Retail TimeUtil.lua24–43 defines elapsed recency formatting in consumers.

## What it must do

- [x] GuildMemberSnapshot preserves optional `last_logout_unix_secs` through JSON/bincode. Unknown is None; remove the duplicate `last_online` display-string authority.
- [x] Protocol field change lands with server and engine consumers, then canonical protocol movement is announced. No old-wire compatibility path.

## How it works

- Consumer `docs/specs/guild-last-online.md` and guild system pages own persistence/presentation.

## Implementation inventory

- `src/protocol_snapshots.rs`: GuildMemberSnapshot.
- `src/protocol_tests.rs`: observed/unknown timestamp wire regression.

## Tests asserting this spec

- `guild_last_online_wire_preserves_observed_timestamp_and_unknown_without_status_text` (enabled after actual wire RED→GREEN).

## Known gaps (current cycle)

None within this slice. gl728 passed and landed protocol50517c9 with server1a2f13fd4 and engine51f1af4f8; canonical protocol was fast-forwarded and announced only with its consumers. Wire RED→GREEN preserves optional time authority; real both-skin disconnect/reconnect/fresh-process recency readback was subsequently accepted. Consumer wiki pages retain exact native evidence and limits.

## Out of scope

New presence classes, sort fields, fabricated historical times, leadership/self-policy changes.
