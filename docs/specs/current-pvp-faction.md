# Current PvP faction

Replicate the player's faction used by `PlayerCondition.CurrentPvpFaction`, following TrinityCore a352b1fa `ConditionMgr::IsPlayerMeetingCondition`.

## What it must do

- [ ] Carry 0 for unknown,1 for Horde/non-Alliance,2 for Alliance. Known neutral Pandaren follows TC's outside-match non-Alliance condition branch without changing neutral reactions.
- [ ] Use racial faction outside matches; entered BG/arena team inside matches.
- [ ] Preserve racial faction while invited or when entry fails; restore it after successful return.
- [ ] Transmit faction-only changes independently of reaction templates.

## How it works

- [Toy rendering](../../../game-engine/docs/wiki/systems/toy-box.md)

## Implementation inventory

- `src/components/unit_frames.rs`: `CurrentPvpFaction` scalar component.
- `src/protocol.rs`: replication registration.

## Tests asserting this spec

- Engine `godot/network/src/wire_tests.rs`: `toyfx20_native_bridge_receives_current_pvp_faction_and_team_only_changes`.
- Server map entry/return channel fixtures.

## Known gaps (current cycle)

- [ ] Entry, return, wire and authored War Banner condition proof pending.

## Out of scope

- NPC reaction templates and faction redesign.
