# Battleground source brackets

BG admission uses the pinned Retail12.1.5.70391 PVPDifficulty range for the selected map and each queued member's level, intersected with the existing BattlemasterList/template admission bounds. A group spanning source ranges is rejected without queuing any member. Matches contain one source range; an older incomplete range must not block a later complete range. Queue entries, the formed match and the live instance retain that range. Arena matching remains separate and gains no Honor/Conquest policy.

No decade arithmetic or broad template maximum substitutes for missing source rows. Examples: map2107 level30–39 retains maximum39; epic maps30/628 use20–79, then80,81–89,90; Deephaul2656 has no source bracket below80. Match pairing does not award Honor. The retained maximum is the prerequisite for TC `Battleground::GetBonusHonorFromKill`, which uses `min(instance maximum,80)`; event-specific awards remain separate work.

## Random32 admission prerequisite — 70448

Random Battleground32 is a multi-map queue, not a concrete `BgTemplate`. Admission uses its first source map566's PVPDifficulty rows (first20–29) intersected with BML32's10–90 bounds; a party may contain at most5 members. Parent matchmaking uses5 minimum/team and10 capacity/team, preserving the requested32 identity and whole parties. The authenticated70448 PVPDifficulty CSV is byte-identical to the bundled source below.

`RANDOM_BG_LEVEL_ELIGIBLE_MAPS` is explicit project policy based on Retail level-banded Random BG behavior, not a TC port: the server chooses a configured map whose own bracket contains all queued levels and stores that level-matching bracket on the concrete instance. It never copies parent RangeIndex across maps. Parent matchmaking preserves original32 separately from the chosen concrete ID. Existing map payouts/start criteria stay unchanged. `RANDOM_BG_WIN_REWARD_PROFILE` remains unresolved: TC's27000/13500/4500/3500 kill-equivalent profile and06:00 server-local reset are not adopted; no new rewards/daily state.

## Fill existing copies

`BG_BACKFILL_WHOLE_PARTY_SMALLER_TEAM` is explicit project policy: a whole queued party joins the smaller existing team only if it fits that team's remaining concrete-map capacity. Never split a party; skip a nonfit while allowing a later smaller fitting group. Actual queued levels must match the concrete instance's own source bracket. Existing phase/score/clock are retained; Ended copies are excluded. The server applies `BG_BACKFILL_MATCH_ORDER` (ascending existing match IDs), preserves each member's original requested queue and counts outstanding/offline reservations before new instance creation. These replace TC's faction/parity and push-front choices, not map eligibility or reward rules.

## Source

Bundled `scripts/reference/pvp-12.1.5.70391/PVPDifficulty.csv` is FDID1339820, layout583BCBE4,354 rows,0 unreadable. Receipt alongside it records raw/schema/CSV SHA256. The local installation advanced to70448 before extraction; the extracted table's content key0554bd932ab15021af8cd9f5eecef070 exactly matches the authenticated cached70391 root25c68dd1c7597cccd94252085c3367ed. It is therefore the pinned70391 content, not an inferred newer-build replacement. CSV SHA256:878ee3b2a6a98ce3e5c916e6aeb30b92d0d0ecaae19ad474093e01751edeb773.

Schema: WoWDBDefs `d6f50d4f351193e84226aa62a36faa0f58528afa/definitions/PVPDifficulty.dbd`. Semantics: TrinityCore a352b1fa `Battleground.cpp`790–795,1467–1470,1528–1534 and `Formulas.h`55–67. Reproducible private extraction/authentication and RED/GREEN proof live in engine `data/diagnostics/pvp-progression-2026-10-16/bg-honor/`; no canonical DB writes.
