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

## Direct member-rank assignment

`SetMemberRank { character_name, rank }` carries only the target and a zero-based rank order. Mainline's detail radio menu uses1..N (`GuildRoster.lua:16–56`); the native client maps display index i to wire i−1. The authenticated connection supplies actor/guild; never accept either from this payload. One request changes the target directly, not a chain of step promotions.

Recheck the target/current/destination rank and choose the Promote right when destination≤current, Demote when destination>current (TC a352b1fa `Guild.cpp:1894–1930`). Reject self, existing actor-or-higher targets, actor-or-higher destinations, rank0 and out-of-range ranks. Current-value UI selection sends no request; an authorized same-rank wire request leaves ranks unchanged. Updates/normal snapshots follow the existing GuildChannel path.

Pinned direct TC processing calls `SendGuildRanksUpdate`→`Member::ChangeRank`, with no lifecycle-log append. Do not manufacture intermediate step events; explicit Promote/Demote commands retain their existing logging. Leadership transfer/disband, rank create/reorder changes and authenticator-backed assignment are excluded named gaps. Upgrade both consumers with the changed request schema; no legacy decoding path.

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

## Confirmed guild disband

`GuildRankRequest::Disband` is an appended unit variant on GuildChannel. It carries no actor or guild identity: the authenticated current guild is the only target, and the server rechecks rank0. Native `/gdisband` and `/guilddisband` open Yes/No first; No/Escape never send this variant. Retail `SlashCommands.lua:805–809` and `GameDialogDefs.lua:662–673` define that confirmation; TC a352b1fa `Guild.cpp:1199–1265,2140–2148` allows the master to destroy a guild with other members present.

Success uses existing empty `GuildRanksState` snapshots for removed online members; there is no fabricated per-member quit history for a destroyed guild. `PetitionManager::release_name` removes only the destroyed guild's case-insensitive reservation; the server writes this manager state in the same transaction as durable guild deletion. Wire/name-release behavioral tests live in `guild_rank_messages.rs` and `guild_creation_tests.rs`; coordinated server/native acceptance is separate.

Protocol additions change the connect-time fingerprint; clients and servers must use the
same protocol revision. No legacy wire fallback.
