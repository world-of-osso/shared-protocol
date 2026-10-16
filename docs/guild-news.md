# Guild news wire

Mainline `Blizzard_Communities/GuildNews.lua`, `Blizzard_FrameXML/GuildUtil.lua`; producer conditions in TrinityCore a352b1fa `AchievementMgr.cpp:475`, `Player.cpp:23904,27194`, `SpellEffects.cpp:1452`.

`GuildNewsRequest` and `GuildNewsState` use ordered reliable GuildChannel. Query and sticky requests never supply an actor/guild ID; the server resolves authenticated current membership. Sticky uses immutable news identity, not a client filter/sort index, and requires the Edit MOTD right. Typed nonmember/permission/missing-entry refusals remain distinct.

Entries retain actual event-time member names, Unix timestamps and250-row insertion order (`SharedDefines.h:6627`). Player-achievement names/descriptions come from installed achievement metadata; item IDs resolve through the actual item catalog. No guild-achievement, encounter or calendar entries exist until corresponding real guild producers exist. This is not lifecycle or bank history.

Consumers gated at engine5e1661761/server5001d9d1/protocol9bb88b6: engine news4/4, server news6/6, wire round-trip1/1, all three cargo checks and format checks pass. Private UDP5891 real acquisitions, ordinary-member sticky refusal, post-quit privacy and both-skin native controls/250-row overflow pass. Fresh server/client processes preserve exact IDs12–261, timestamps and the actual sticky with identical binary receipts. Proof ledger: `/home/osso/.worktrees/logs/guild-news-2026-10-15-run1/`; independent lead integration remains separate. Publish only with these gated consumers.
