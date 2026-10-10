# Wild pet battles

Authoritative wild PvE battle transport in `src/protocol/pet_battle_messages.rs`, journal combat slots in `src/pet_battle.rs`. Runtime rules remain server-owned, not the historical shared prototype resolver.

## What it must do
- [ ] Preserve start/state/round/end, snapshots, combat text, rewards and captured GUID across the wire.
- [ ] Transport ability1–3, pet swap1–3, pass, forfeit and trap decisions with battle/round identity.
- [ ] Persist three ordered owned distinct journal slots; releasing a pet clears its slot.
- [ ] Both skins consume the same authoritative snapshots and action semantics.

## How it works
- [Server turn catalog and producer](../../../game-server/docs/wiki/systems/pet-battle-engine.md).
- Messages use the existing reliable ordered CollectionChannel. Zero timer denotes untimed wild PvE.

## Implementation inventory
- `src/protocol/pet_battle_messages.rs`: wild battle wire snapshots, updates and requests.
- `src/protocol/registration.rs`: protocol registration and direction.
- `src/pet_battle.rs`: journal slots and ownership validation.

## Tests asserting this spec
- `tests/wild_pet_battle.rs`: actual bincode round trips and journal state/serialization behavior.

## Known gaps (current cycle)
- [ ] Server and native client consumers, spawn/capture/XP lifecycle and live proof.

## Out of scope
PvP matchmaking and unsupported turn effects are not added by this wire change.
