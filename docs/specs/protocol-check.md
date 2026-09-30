# Connect-time protocol check

`src/protocol/protocol_check.rs`, registered first by `ProtocolPlugin`, rejects a connection whose peer was built with different registries.

## What it must do

- [x] On `Connected`, each side sends a `ProtocolFingerprint`: lightyear's message and channel registry hashes and replicon's `ProtocolHash` (replication rules), each covering type names in registration order, plus message and component layout hashes covering each registered type's wire layout.
- [x] A differing fingerprint rejects the connection on the receiving side: `ProtocolRejected(reason)` names every differing registry (message, component, channel, message layout, component layout; a layout is named only when that registry's type names match), and the link drops 1 s later so the peer also receives this side's fingerprint.
- [x] Dropping a link disconnects the peer at once on either side: the peer sees `Disconnected` about 1 s after the rejection, not at its netcode timeout.
- [x] Every shared message and replicated component registers through `ProtocolRegistrationExt` (`register_protocol_message`, `protocol_component`), which records its layout. `ProtocolPlugin::finish` panics naming any other type registered without one; lightyear's own types (names starting `lightyear`) are exempt.
- [x] A peer that sends no fingerprint within `ProtocolCheckTimeout` (default 10 s) is rejected the same way; builds before this check never send one.
- [x] A matching fingerprint inserts `ProtocolVerified`. Hosts treat a connection as usable only after `ProtocolVerified`.
- [x] Client hosts install `defer_lightyear_protocol_check` as the app error handler so lightyear's own check warns instead of panicking and this check reports the difference.
- [x] The check's channel and message register before every other shared registration, so their ids agree between builds whose later registrations differ.

## How it works

- Lightyear 0.28's own `ProtocolCheck` never sees component changes: `ComponentRegistry`'s hasher is not fed since replication moved to replicon, and lightyear runs replicon with `AuthMethod::None`, so replicon's `ProtocolHash` goes unchecked. That check runs only on the client and panics through Bevy's default error handler on message/channel differences.
- Lightyear netcode (0.28 through 0.30.1 and `main` of 2026-09-14) has no per-client server-side disconnect: `NetcodeServerPlugin` observes only `Stop`. `vendor/lightyear_netcode` adds a `Disconnect` observer for a netcode `ClientOf` that sends the netcode disconnect packets and inserts `Disconnecting`, as `stop` does per client (`vendor/lightyear_netcode/UPSTREAM.md`). Both sides drop a rejected link with `Disconnect`; game-server patches `lightyear_netcode` to this copy.
- A layout is the serde shape the payload encoding follows: struct fields with names and order, tuple lengths, primitives, and every enum variant reachable from the type. `layout_tracer.rs` deserializes the type from a tracer that visits one element of each sequence, option and map; each pass takes an enum's next unvisited variant, repeating until all were visited. Type paths are not part of a layout (the name registries cover them). A `Deserialize` that validates zero/empty primitives, uses `deserialize_any`, or is recursive fails at startup.

## Tests asserting this spec

- `tests/protocol_check.rs`: server and client apps over real Netcode/UDP on localhost; matching registries verify, a client-only component, swapped component order, a server-only message and a client-only channel are rejected by both sides, and a missing fingerprint is rejected by each side. A server rejecting a client without the check disconnects it within 3 s (the netcode timeout would take 4.8 s). A component and a message that differ only by an added field (`tests/fixtures/layout-v1`/`-v2`, same crate name, so same type names) are rejected by both sides.
- `src/protocol/protocol_layout_tests.rs`: every real shared message and component traces a layout; a message registered around `ProtocolRegistrationExt` stops `finish`; a nested enum variant or variant field change alters the layout, identical shapes in different modules do not.

## Known gaps (current cycle)

- [ ] A client built before this check gets no reason from the server; it is disconnected about 1 s after the rejection with none.
- [ ] Custom (non-derived) `Serialize`/`Deserialize` impls are traced only through their `Deserialize`; a serializer that writes a different shape than its deserializer reads is not detected.
