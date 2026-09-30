# lightyear_netcode

- Version: `0.28.0`
- Crates.io checksum: `7f009241f6954e89fc2417f10488deee00cca08028e4eb20b7ad8c4954fc8832`
- Upstream commit: `28e823d9df394c193dfc09f8eb891b77424e81c5`
- License: MIT OR Apache-2.0

Vendored for a per-client server disconnect. Upstream `NetcodeServerPlugin` observes only
`Stop` (every client); `Disconnect` on a server-side `ClientOf` has no observer, so the
client keeps its netcode session until the connection times out. `disconnect_client`
handles it the way `stop` handles each client. Unchanged through 0.30.1 and `main` as of
2026-09-14 (`crates/connection/netcode/src/server_plugin.rs`).

Retire when upstream handles `Disconnect` on a server-side `ClientOf`.
