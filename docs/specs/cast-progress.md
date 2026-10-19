# Cast progress

Shared `CastState` advances source millisecond cast/channel timers without accumulating float drift. Pinned TC `a352b1fa` `Spell.cpp:4240,4265–4275` expires a preparing spell when the elapsed diff reaches its remaining timer. [Server integration](../../../game-server/docs/specs/ramparts-herald.md#tests-asserting-this-spec).

## What it must do

- [ ] Convert replicated seconds and elapsed steps to the nearest integer millisecond, not truncation or an expiry epsilon.
- [ ] A1500ms normal cast remains active through1450ms and completes at the thirtieth50ms step.
- [ ] Preserve channel periods and exact total duration:3000ms/1000ms emits periods at1000/2000ms and completes at3000ms.
- [ ] Preserve normal/channel pushback semantics and the existing replicated f32 fields, types and encoding.

## How it works

- [Cast state and progression](../../src/casting.rs).
- [Server acceptance](../../../game-server/docs/specs/ramparts-herald.md#tests-asserting-this-spec).

## Implementation inventory

- `src/casting.rs` — replicated cast state, progress and pushback.
- `src/casting_tests.rs` — concrete normal/channel/nearest-ms/pushback deadlines.

## Tests asserting this spec

- `src/casting_tests.rs::cast_millisecond_*` — source milliseconds across real repeated50ms calls; three ignored RED regressions and nearest-ms control before the fix.
- Server `herald_ground::encounter_coverage_herald_ground_roar_cast_1500ms_ai_deadline` — actual source-scheduled ordinary Roar must stop blocking AI at1500ms; genuine native RED39.30s precedes shared production changes.

## Known gaps (current cycle)

- [ ] Prove shared RED→GREEN, then native AI GREEN, and co-land protocol/server tips.
- [ ] Independent wide player/NPC/pet/follower/channel/pushback gate remains required.

## Out of scope

- Changing wire fields, source durations, EventMap schedules, pushback policy or timing oracles.
- Cooldown/charge clocks and client cast-bar interpolation.
