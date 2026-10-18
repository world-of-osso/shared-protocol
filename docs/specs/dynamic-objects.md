# Received dynamic objects

Ordinary same-copy clients receive actual Rain of Fire86636 area geometry, matching the caster's persistent dynamic object. Shape/effect/radius authority is active70448 SpellEffect88312/SpellRadius8, not a guessed warning marker. Protocol and both consumers must land together.

## What it must do

- [x] Replicate the actual caster entity bits, spell and resolved effect radius with existing Position.
- [x] Use ordinary entity visibility and removal for area lifetime; no client expiry guesses.
- [x] Preserve numeric identity/geometry through wire encoding.
- [x] Keep other-copy clients excluded by normal server interest.

Backing proof combines the actual server boss/interest boundary with real UDP client decoding; fresh full-client gameplay remains the separate gap below.

## How it works

- [Server spell contract](../../../game-server/docs/specs/rain-of-fire-visibility.md)
- [Engine harness contract](../../../game-engine/docs/specs/bot-harness.md)

## Implementation inventory

- `src/components/dynamic_object.rs`: received geometry and exact spell identity.
- `src/protocol.rs`: traced replicated registration.

## Tests asserting this spec

- `src/components/dynamic_object.rs`: wire roundtrip.
- Consumer server real Stockade/interest tests and engine real UDP replica tests assert lifecycle/copy behavior.

## Known gaps (current cycle)

- [ ] Joint consumer gate and fresh private native harness proof.

## Out of scope

- Other damaging areas, cast destination inference, client expiry timers, geometry/stat tuning.
