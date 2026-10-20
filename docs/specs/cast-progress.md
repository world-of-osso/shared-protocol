# Cast progress

Shared `CastState` advances source millisecond cast/channel timers without accumulating float drift. Pinned TC `a352b1fa` `Spell.cpp:4240,4265–4275` expires a preparing spell when the elapsed diff reaches its remaining timer. [Server integration](../../../game-server/docs/specs/ramparts-herald.md#tests-asserting-this-spec).

## What it must do

- [x] Convert replicated seconds and elapsed steps to the nearest integer millisecond, not truncation or an expiry epsilon.
- [x] A1500ms normal cast remains active through1450ms and completes at the thirtieth50ms step.
- [x] Preserve channel periods and exact total duration:3000ms/1000ms emits periods at1000/2000ms and completes at3000ms.
- [x] Preserve normal/channel pushback semantics.
- [ ] Preserve existing replicated f32 fields, types and encoding; no wire declaration changes, independent protocol gate pending.

## How it works

- [Cast state and progression](../../src/casting.rs).
- [Server acceptance](../../../game-server/docs/specs/ramparts-herald.md#tests-asserting-this-spec).

## Implementation inventory

- `src/casting.rs` — replicated cast state, progress and pushback.
- `src/casting_tests.rs` — concrete normal/channel/nearest-ms/pushback deadlines.

## Tests asserting this spec

- `src/casting_tests.rs::cast_millisecond_*` — source milliseconds across real repeated50ms calls; three genuine normal/channel/pushback deadline REDs and one nearest-ms control PASS at7b09847 before the fix. Integer-ms implementation retains the f32 wire fields and unchanged pushback methods; all53 shared casting-family cases pass atbb18da9 (including the four new cases and existing normal/channel/pushback/charge/cooldown/projectile controls),0ignored/no warnings; the three regressions are enabled. Native actual1500ms AI expiry passes50.60s at server2b6397005/protocol38afb69; six retained ground/ordinary Omor pushback controls pass105.21s. The native probe waits for actual natural Roar admission, preserving initial category-cooldown refusal; its1450/1500ms assertions are unchanged.
- Server `herald_ground::encounter_coverage_herald_ground_roar_cast_1500ms_ai_deadline` — actual source-scheduled ordinary Roar must stop blocking AI at1500ms; genuine native RED39.30s precedes shared production changes.

## Known gaps (current cycle)

- [ ] Co-land protocol/server tips after the independent wide gate; shared53PASS and native actual1500ms GREEN plus six retained controls are recorded above.
- [ ] Independent wide player/NPC/pet/follower/channel/pushback gate remains required.

## Out of scope

- Changing wire fields, source durations, EventMap schedules, pushback policy or timing oracles.
- Cooldown/charge clocks and client cast-bar interpolation.
