# Replicated pet identity

- `UnitIsPet(bool)` declares actual pet designation independently of general minion ownership or `PLAYER_CONTROLLED`. Actual pet constructors and SummonProperties Control=PET publish true; ally guardians such as Witherbark Gong143703 do not.
- Local and party pet-frame selection requires that designation and the matching owner in `UnitSummonedBy`. Other owned minions cannot replace a pet.
- The designation survives native wire decoding, value updates and component removal. Adding it changes the connection fingerprint; existing component meanings remain unchanged.
- `PetSpells`/`PetClearSpells` remain owner-only action-bar messages. They do not identify remote party members' pets.
- `CombatLogKind::Summon` records spell-created unit ownership at creation: source=summoner, target=new unit, spell_id=actual summoning spell. Short-lived guardians remain attributable after despawn. Login/script/journal restoration does not synthesize spell events; persistent pets retain their replicated owner relation. The variant is appended without reordering existing kinds.

Source: TrinityCore a352b1fa `Unit.cpp::SetMinion`6284–6320 and `Object.cpp::Map::SummonCreature` category PET. Tests: native actual143703 flags counterexample, real loopback boolean/update/removal, actual called-wolf designation and learned Gong exclusion.
