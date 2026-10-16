# Guild news wire

Mainline `Blizzard_Communities/GuildNews.lua`, `Blizzard_FrameXML/GuildUtil.lua`; producer conditions in TrinityCore a352b1fa `AchievementMgr.cpp:475`, `Player.cpp:23904,27194`, `SpellEffects.cpp:1452`.

`GuildNewsRequest` and `GuildNewsState` use ordered reliable GuildChannel. Query and sticky requests never supply an actor/guild ID; the server resolves authenticated current membership. Sticky uses immutable news identity, not a client filter/sort index, and requires the Edit MOTD right. Typed nonmember/permission/missing-entry refusals remain distinct.

Entries retain actual event-time member names, Unix timestamps and250-row insertion order (`SharedDefines.h:6627`). Player-achievement names/descriptions come from installed achievement metadata; item IDs resolve through the actual item catalog. No guild-achievement, encounter or calendar entries exist until corresponding real guild producers exist. This is not lifecycle or bank history.

Consumers and actual-action/persistence acceptance are pending. New messages must not be published before both engine and server consumers are gated.
