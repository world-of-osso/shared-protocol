# Guild ranks wire contract

`GuildRankRequest` and `GuildRanksState` use the existing ordered reliable `GuildChannel`.
Rank IDs are zero-based positions, 2–10 ranks; rank 0 is immutable. Only the Guild Master
edits ranks, flags and bank limits. Promotions/demotions require the corresponding flag
and a strictly lower-ranked target; promotion never reaches the actor's rank.

Snapshots contain ordered ranks/rights, member ranks, recipient rank, purchased tab names
and a typed refusal. Tab withdrawal limits count stacks, not individual items. Zero denies
withdrawals. Gold limits are copper/day, shared with guild repairs; zero denies spending.
Rank 0 alone is unlimited. Unknown flags and invalid ranks/tabs are rejected by the server.

Retail source (local `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`):
- `Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:4,137–238,536–620`: max 10, minimum 2, occupied-rank deletion, tab view/deposit/stacks controls.
- `Blizzard_GuildControlUI/Blizzard_GuildControlUI.lua:420–459`: rank flags, shared repair/withdraw gold allowance.
- `Blizzard_Communities/GuildRoster.lua:137–141`: promotion/demotion strict rank hierarchy.

Protocol additions change the connect-time fingerprint; clients and servers must use the
same protocol revision. No legacy wire fallback.
