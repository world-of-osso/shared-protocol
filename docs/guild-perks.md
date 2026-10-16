# Guild perk catalog

Mainline `GuildPerkSpells.csv` selects actual perk spell IDs in DB2 row order. `load_guild_perks` joins installed `SpellName.csv` and `SpellMisc.csv` for real labels/icons; missing metadata returns an explicit source error. No wire messages or hard-coded production perk definitions are added.

Installed rows2/7/10/14/16 identify78633/83944/83958/83950/83951. Client Benefits uses the source `GuildPerks.lua`/XML; server membership grant/removal follows TrinityCore a352b1fa `Guild.cpp:2322,3002`. A displayed/known spell is not proof of a working spell effect.

Tests: `guild_perks::tests` GREEN2/2 at e50b02c covers authentic definitions/order and refusal to invent missing metadata. Client source-list1/1 and server actual membership/rebuild/quit/reopen1/1 pass; native/effect proof and final consumer gates remain open. Do not publish before both consumers are gated.
