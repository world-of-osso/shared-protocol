# Wild pet battles

Authoritative wild PvE battle transport in `src/protocol/pet_battle_messages.rs`, journal combat slots in `src/pet_battle.rs`. Runtime rules remain server-owned, not the historical shared prototype resolver.

## What it must do
- [ ] Preserve start/state/round/end, snapshots, combat text, rewards and captured GUID across the wire.
- [ ] Transport ability1–3, pet swap1–3, pass, forfeit and trap decisions with battle/round identity.
- [ ] Persist three ordered owned distinct journal slots; releasing a pet clears its slot.
- [ ] Both skins consume the same authoritative snapshots and action semantics.

- [x] Preserve HUD species icon/quality/XP, ability family, aura icons and target-specific event feedback; rewards preserve the new level XP threshold. `tests/wild_pet_battle.rs` bincode round trips pass2/2 at1808cdd.

- [x] Preserve a single weather aura, two independently oriented team-pad aura lists, aura names/polarity/signed durations and separate ability-lockdown turns through JSON and the actual bincode wire. Empty/indefinite state is not inferred from HP or cooldown.

## How it works
- [Server turn catalog and producer](../../../game-server/docs/wiki/systems/pet-battle-engine.md).
- Messages use the existing reliable ordered CollectionChannel. Zero timer denotes untimed wild PvE.

## Implementation inventory
- `src/protocol/pet_battle_messages.rs`: wild battle wire snapshots, updates and requests.
- `src/protocol/registration.rs`: protocol registration and direction.
- `src/pet_battle.rs`: journal slots and ownership validation.

## Tests asserting this spec
- `tests/wild_pet_battle.rs`: actual bincode round trips and journal state/serialization behavior.
- `tests/petbattle_effect_hud.rs`: concrete Sunny Day/Cyclone state and max-countdown inputs survive the wire without dropping fields.

## Known gaps (current cycle)
- [x] [Server acceptance](../../../game-server/docs/specs/wild-pet-battles.md) and [native acceptance](../../../game-engine/docs/specs/wild-pet-battles.md) own runtime/live proof; wire tests alone do not certify Retail gameplay parity.

## Out of scope
PvP matchmaking and unsupported turn effects are not added by this wire change. The wire can represent nonzero lockdown; the server's still-gated lockdown handlers must not fabricate turns. All peers must adopt the new snapshot schema together; old-wire compatibility is not supplied.
