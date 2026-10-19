# Voluntary guild leadership transfer

Represent the sourced `/gleader <member>` request on GuildChannel. Mainline `SlashCommands.lua:793–799` calls C_GuildInfo.SetLeader; TC a352b1fa Guild.cpp1531–1576 owns the normal transfer boundary. Server and engine guild specifications own authority and input behavior.

## What it must do

- [x] `TransferLeadership { character_name }` preserves only the named target through JSON/bincode; actor and guild are supplied by authenticated server state, not new packet fields.
- [x] Protocol and both consumers land together. No old-wire compatibility path or default transfer target.

## How it works

- Server `docs/wiki/systems/guild-lifecycle.md`; engine `docs/wiki/systems/guilds.md`.

## Implementation inventory

- `src/protocol/guild_rank_messages.rs`: target-only transfer appended to the existing GuildChannel request contract.
- `src/protocol_tests.rs`: target-only wire regression, enabled after actual RED→GREEN.

## Tests asserting this spec

- `guild_leadership_transfer_wire_preserves_only_the_target_name`.

## Known gaps (current cycle)

None within this slice. gz709 passed and landed protocol724ec71 with serverb15526d95 and engine79b33ab93. Wire RED→GREEN proves exact JSON/bincode target preservation; real typed transfer and fresh-process authority were subsequently accepted in both skins. Consumer wiki pages retain native evidence and scope.

## Out of scope

Disband, inactivity dethroning, authenticator eligibility and new presence fields. Self-transfer uses the explicitly approved server policy GUILD_SELF_LEADER_TRANSFER_REFUSED_UNSOURCED, not a claim about Retail's unavailable native C API.
