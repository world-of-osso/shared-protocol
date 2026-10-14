# Guild tabard catalog

`guild_tabards::GuildTabardCatalog` reads installed GuildColorBackground/Border/Emblem, GuildEmblem and GuildTabardBackground/Border/Emblem CSVs. Source IDs remain IDs, not RGB values. RGB reads the named columns; exported Blue-before-Green column order must not swap channels. Character components3/4 are torso upper/lower; their layers are background, emblem, border.

The csv dependency is shared so server validation and client preview use the same parser and lookup decisions. Missing tables, columns, records, out-of-range IDs and unavailable texture combinations fail explicitly. There is no old-build data path.

`GuildTabardRequest::{Open,Save}` and `GuildTabardReply::{Vendor,Saved,Failed}` travel on ordered-reliable GuildChannel. Requests carry the interacting NPC identity; the server owns membership, rank, catalog validation and personal-gold payment. `GuildSnapshot.tabard` is optional authoritative design, not a preview draft. These additive serde-shape changes enter the layout fingerprint; no existing-field meaning changed, so semantic epoch1 remains. Landing requires the full protocol suite, including connect rejection.

Tests use concrete records copied from the already-exported12.1.5.70391 catalog in `tests/fixtures/guild-tabards-70391`; no DB2 export was performed. `tests/guild_tabards.rs` asserts real texture FDIDs/color channels and rejects invalid choices.
