# Protocol changelog

## Unreleased

- **Coordinated wire break:** `TransportState` adds source acceleration and per-knot stop, delay, arrival-event and departure-event data. Server and client must update together; mixed versions cannot decode this bitcode struct. Existing zero-stop Seething routes retain their constant-speed behavior.
