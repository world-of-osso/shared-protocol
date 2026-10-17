# Guild ranks wire contract

`GuildRankRequest` and `GuildRanksState` use the existing ordered reliable `GuildChannel`.
Rank IDs are zero-based positions, 2–10 ranks; rank 0 is immutable. Only the Guild Master
edits ranks, flags and bank limits. Promotions/demotions require the corresponding flag
and a strictly lower-ranked target; promotion never reaches the actor's rank.

`GuildRankRequest::Leave` removes the actor; rank zero receives `LeaderCannotLeave`.
`RemoveMember` requires removal rights and a strictly lower-ranked target. Success refreshes
all affected online rosters, including the removed character's empty roster.
`GuildInvitation` carries the authoritative inviter and guild on AuthChannel. It is sent
to online targets and redelivered on world entry from the persisted invitation.
Its optional `GuildTabardDesign` is the guild's actual registered design at delivery;
no registered design stays `None`. Retail GuildInviteFrame.lua11 consumes this payload;
TC a352b1fa Guild.cpp1727–1738 fills it from the inviting guild. Deploy both consumers
with this changed wire schema; no legacy decoding path.
`GuildDeclineInviteRequest` clears that pending invite; accept remains `GuildAcceptInviteRequest`.

Snapshots contain ordered ranks/rights, member ranks, recipient rank, purchased tab names
and a typed refusal. Tab withdrawal limits count stacks, not individual items. Zero denies
withdrawals. Gold limits are copper/day, shared with guild repairs; zero denies spending.
Rank 0 alone is unlimited. Unknown flags and invalid ranks/tabs are rejected by the server.

Rights use pinned TrinityCore `a352b1fa` `src/server/game/Guilds/Guild.h:102–127`
`GuildRankRights` values, not consecutive UI checkbox indices. `GUILD_RIGHT_ALL` is
only the fifteen implemented rights; unnamed/unsupported TC bits are not accepted.
Server guild layout 3 translates layout-2 compact social bits individually; existing
bank booleans, limits, tabs, money and member positions are unchanged. Layout 0/1
migrations retain their existing default grants. Upgrade client and server together:
[semantic epoch1](specs/protocol-check.md) folds this meaning change into the connect-time
fingerprint even though field shapes are unchanged; different epochs are rejected.

Retail source (local `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`):
- `Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:4,137–238,536–620`: max 10, minimum 2, occupied-rank deletion, tab view/deposit/stacks controls.
- `Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:420–459`: rank flags, shared repair/withdraw gold allowance.
- `Blizzard_Communities/GuildRoster.lua:137–141`: promotion/demotion strict rank hierarchy.

## Lifecycle log

`GuildRankRequest::QueryEventLog` carries no guild ID: the authenticated actor selects
its own guild. `GuildEventLog` replies on GuildChannel with oldest-to-newest real
insertion order, current server Unix seconds and an optional `GuildRankError`.
Each `GuildEvent` carries its mutation-time Unix seconds and one of six kinds:
invite/join/promote/demote/remove/quit, with event-time actor/member/rank names.
TC a352b1fa `Guild.h:209–216` defines these kinds; `SharedDefines.h:6626` limits
retained history to100. Mainline `GuildInfo.lua:158–195` renders newest first.
Empty actual history stays empty; refused operations never create events, and
mutations commit their event atomically. Bank logs and news are separate histories.

Protocol additions change the connect-time fingerprint; clients and servers must use the
same protocol revision. No legacy wire fallback.
