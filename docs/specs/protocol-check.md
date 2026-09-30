# Connect-time protocol check

`src/protocol/protocol_check.rs`, registered first by `ProtocolPlugin`, rejects a connection whose peer was built with different registries.

## What it must do

- [x] On `Connected`, each side sends a `ProtocolFingerprint`: lightyear's message and channel registry hashes and replicon's `ProtocolHash` (replication rules). Each covers type names in registration order.
- [x] A differing fingerprint rejects the connection on the receiving side: `ProtocolRejected(reason)` names every differing registry (message, component, channel), and the link drops 1 s later so the peer also receives this side's fingerprint.
- [x] A peer that sends no fingerprint within `ProtocolCheckTimeout` (default 10 s) is rejected the same way; builds before this check never send one.
- [x] A matching fingerprint inserts `ProtocolVerified`. Hosts treat a connection as usable only after `ProtocolVerified`.
- [x] The check's channel and message register before every other shared registration, so their ids agree between builds whose later registrations differ.

## How it works

- Lightyear 0.28's own `ProtocolCheck` never sees component changes: `ComponentRegistry`'s hasher is not fed since replication moved to replicon, and lightyear runs replicon with `AuthMethod::None`, so replicon's `ProtocolHash` goes unchecked. That check runs only on the client and panics through Bevy's default error handler on message/channel differences.
- Lightyear 0.28 netcode has no per-client server-side disconnect; `Disconnect` on a `ClientOf` is not observed. The server drops a rejected link by inserting `Disconnecting`, which ends payloads and keepalives; the client then times out unless its own check already disconnected it.

## Tests asserting this spec

- `tests/protocol_check.rs`: server and client apps over real Netcode/UDP on localhost; matching registries verify, a client-only component, swapped component order, a server-only message and a client-only channel are rejected by both sides, and a missing fingerprint is rejected by each side.

## Known gaps (current cycle)

- [ ] Hashes cover type names and order, not field layouts: changing a registered type's fields without renaming it is not detected.
- [ ] A client built before this check gets no reason from the server; it times out after the server drops the link.
