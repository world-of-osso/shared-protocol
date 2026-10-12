# Retail reputation standing

- Traditional faction reputation uses signed values: Hated−42000, Hostile−6000, Unfriendly−3000, Neutral0, Friendly3000, Honored9000, Revered21000, Exalted42000. Standard standing clamps to−42000…42000.
- Starting standing uses the first matching Faction.ReputationRaceMask/ClassMask/Base rule, including noncontiguous ChrRaces.PlayableRaceBit values and class-specific Cenarion Circle baselines. Unknown faction-rule matches return0, as Retail's base-reputation lookup does.
- The compiled registry contains every reputation-indexed faction from authenticated Mainline12.1.0.69933 Faction.csv. Reproduce with `scripts/import_reputation_baselines.py <pinned-db2-dir> src/reputation_factions.rs`; generated source records CSV hashes. The previous three-faction registry and invented opposing spillover are replaced, not retained as an alternate path. Authored reward/spillover policies remain a separate gameplay-data responsibility.
- New characters initialize all435 faction baselines for their race and class. Character tracking/watch preferences remain intact.
- Old unsigned saves migrate once into server `reputations_retail_v2`: new authentic baseline plus old saved value minus the old race baseline, clamped to signed bounds. Missing factions initialize from current DB2. Preserve earned achievements. Lead authorized this policy on2026-10-11 after reporting production's zero-saved-character export.
- Historical deltas lost by old zero/max clamps cannot be recovered; migration preserves only recoverable differences. No fabricated history or new earn date is created.
- Achievement type46 consumes positive signed standings from load/change boundaries. Type46 quantities serialized before support are invalid and must be rebuilt from signed reputation; legacy earned IDs remain preserved.

Primary sources: TrinityCore a352b1fa ReputationMgr.cpp38–59,133–139,839–857; RaceMask.h GetRaceBit; CriteriaHandler.cpp709–715. Pinned local Faction and ChrRaces CSV hashes are embedded in generated source.

Special friendship/renown/paragon rank, cap and reward policies are not certified by the traditional standing conversion.
