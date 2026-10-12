# Forced player facing

Source-defined server turns, including Unusual Compass64385 (TrinityCore a352b1fa), require owning-client adoption rather than an ordinary predicted Rotation snapshot.

## What it must do

- [ ] Replicate an exact `ForcedFacing { epoch, yaw }` correction with its player.
- [ ] Apply yaw once when its epoch matches MovementControl; preserve subsequent player turns.
- [ ] Ignore corrections from a different epoch; do not alter ordinary teleport-facing behavior.
- [ ] Reject old-epoch player inputs without resetting ongoing fall state on a facing-only correction.

## How it works

- Server source contract: game-server `docs/specs/toys.md`.

## Implementation inventory

- `src/components.rs`: ForcedFacing epoch and yaw.
- `src/protocol.rs`: replicated component registration.

## Tests asserting this spec

- `src/protocol_taxi_tests.rs`: exact correction serialization.
- Native game-engine network/world and game-server movement/toy fixtures cover adoption and stale inputs.

## Known gaps (current cycle)

- [ ] Owning-client process proof in Modern and Forever.

## Out of scope

- Generic dummy scripts, random movement destinations and new player input controls.
