# Guild perk catalog

Mainline `GuildPerkSpells.csv` selects actual perk spell IDs in DB2 row order. `load_guild_perks` joins installed `SpellName.csv` and `SpellMisc.csv` for real labels/icons; missing metadata returns an explicit source error. No wire messages or hard-coded production perk definitions are added.

Installed rows2/7/10/14/16 identify78633/83944/83958/83950/83951. Client Benefits uses the source `GuildPerks.lua`/XML; server membership grant/removal follows TrinityCore a352b1fa `Guild.cpp:2322,3002`. A displayed/known spell is not proof of a working spell effect.

Tests: `guild_perks::tests` covers authentic definitions/order and refusal to invent missing metadata. Consumer UI, membership reconciliation and native proof remain pending; do not publish before both consumers are gated.
