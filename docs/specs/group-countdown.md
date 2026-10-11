# Group countdown

Retail Mainline player countdown (`C_PartyInfo.DoCountdown`) over the reliable group channel. UI source: local `Blizzard_FrameXML/Timer.lua` and `Blizzard_CompactRaidFrames/Mainline/Blizzard_CompactRaidFrameManager.lua`.

## What it must do

- [ ] Requests carry the selected integer duration; zero requests cancellation.
- [ ] Server updates carry initiator name, original duration, remaining seconds and UTC deadline; delayed delivery must not restart an expired countdown.
- [ ] Zero original duration means cancellation; positive original duration with zero remaining means normal expiry.
- [ ] Start, replacement, cancellation and expiry reach every online group member.

## How it works

- [Engine group contract](../../../game-engine/docs/specs/group-frames.md).

## Implementation inventory

- `src/protocol/group_messages.rs` — countdown wire types and registration.

## Tests asserting this spec

- `src/protocol_group_tests.rs::countdown_start_and_cancel_round_trip_on_the_wire` — concrete request/update serialization.

## Known gaps (current cycle)

- [ ] Final server/native integration and live countdown/audio/manager proof. Assistant assignment and authoritative assistant snapshots are part of the group wire contract.
- [ ] Local Lua sources name `Constants.PartyCountdownConstants.MaxCountdownSeconds` without exposing its numeric value. Do not infer a duration limit.

## Out of scope

Ready checks retain their existing wire schema.
