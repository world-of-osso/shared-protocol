# Guild tabard catalog

`guild_tabards::GuildTabardCatalog` reads installed GuildColorBackground/Border/Emblem, GuildEmblem and GuildTabardBackground/Border/Emblem CSVs. Source IDs remain IDs, not RGB values. RGB reads the named columns; exported Blue-before-Green column order must not swap channels. Character components3/4 are torso upper/lower; their layers are background, emblem, border.

The csv dependency is shared so server validation and client preview use the same parser and lookup decisions. Missing tables, columns, records, out-of-range IDs and unavailable texture combinations fail explicitly. There is no old-build data path.

Tests use concrete records copied from the already-exported12.1.5.70391 catalog in `tests/fixtures/guild-tabards-70391`; no DB2 export was performed. `tests/guild_tabards.rs` asserts real texture FDIDs/color channels and rejects invalid choices.
