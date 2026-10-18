# Guild perk catalog

## Mobile Banking project policy

User decision2026-10-17 defines `MOBILE_BANKING_COOLDOWN_2H` as7200000ms for83958, overriding installed70448 SpellCooldowns recovery3600000ms. Server cooldown receipts and client spell tooltip share this constant; no wire shape changes. Installed88304/88306 summon206602/206603 for300000ms. Casting requires guild membership/perk, not Friendly; chest users require guild membership and Friendly faction1168. User clarification2026-10-17 excludes resurrection guild perks from World of Osso;83950 is not a pending effect gap.


Mainline `GuildPerkSpells.csv` selects actual perk spell IDs in DB2 row order. `load_guild_perks` joins installed `SpellName.csv` and `SpellMisc.csv` for real labels/icons; missing metadata returns an explicit source error. No wire messages or hard-coded production perk definitions are added.

Installed rows2/7/10/14/16 identify78633/83944/83958/83950/83951. Client Benefits uses the source `GuildPerks.lua`/XML; server membership grant/removal follows TrinityCore a352b1fa `Guild.cpp:2322,3002`. A displayed/known spell is not proof of a working spell effect.

Tests: `guild_perks::tests` GREEN2/2 at e50b02c covers authentic definitions/order and refusal to invent missing metadata. Client source-list1/1 and server actual membership/rebuild/quit/reopen1/1 pass. Mobile policy consumers landed through425/425b/455e; server6/6 and client tooltip1/1 pass. Consumer native evidence proves both-skin cast/chest/use/refusal/despawn and actual accepted recasts after real2h:7311223ms for the Modern-origin interval and7242700ms for Forever, identical artifacts and no clock injection. Prior interrupted trial results remain intact; completed cases are identified separately rather than inferred from displayed spell knowledge.
