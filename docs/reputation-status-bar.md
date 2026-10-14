# Watched reputation payload

`ReputationSnapshot.watched` is owner-only authoritative bar state on the existing `ReputationStateUpdate`/`ReputationChannel`, not the bulk Journeys reward catalog. `None` explicitly removes the watched bar. An update with `snapshot: None` retains the last snapshot and reports only its message/error.

`WatchedReputationSnapshot` carries real Faction ID/name, normalized nonnegative `value`, strictly positive `maximum`, capped status, major-faction classification and standard reaction. Major factions use renown standing/level threshold; capped standard Exalted uses1/1. `RenownRequest::SetWatchedFaction` acknowledgements publish this state independently of their existing RenownSnapshot reply; refusals preserve the previous watch.

Source/consumer contract: [Retail reputation status bar](../../game-engine/docs/specs/reputation-status-bar.md). New serde shape changes the protocol-layout fingerprint; no existing field changed meaning, so the semantic epoch is unchanged.
