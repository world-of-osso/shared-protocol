# Player motion

`PlayerMotion` carries replicated direction, mode and airborne state in its existing `u64` (`src/components/unit_frames.rs`).

## What it must do

- [x] Preserve Retail movement bit values, including FALLING (0x800).
- [x] Carry unjumped FALLING and FALLING with JUMP_STARTED through bitcode and bincode; retain replication registration.
- [x] Encode ordinary forward FALLING in 3 bincode bytes and the same flags with jump origin in 9 bytes.

## How it works

- [Retail movement flags](https://github.com/TrinityCore/TrinityCore/blob/master/src/server/game/Entities/Object/MovementInfo.h): FALLING and FALLING_FAR are falling-state flags, not jump-origin markers.
- [Retail movement handler](https://github.com/TrinityCore/TrinityCore/blob/master/src/server/game/Handlers/MovementHandler.cpp): CMSG_MOVE_JUMP is a separate event. JUMP_STARTED stores that origin for an airborne interval in project-only bit 63; no field is added.

## Implementation inventory

- `src/components/unit_frames.rs` — movement bits and jump-origin marker.

## Tests asserting this spec

- `src/protocol_player_motion_tests.rs` — bit values, round trips/registration and bounded bincode cost.

## Known gaps (current cycle)

None.

## Out of scope

Retail long-fall phase generation: not needed to distinguish a jump from walking off a ledge.
