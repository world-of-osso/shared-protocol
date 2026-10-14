# lightyear_udp

- Version: `0.28.0`
- Crates.io checksum: `f5de693f3e82a67aeed2bdec3f29dad9394aaa5b8a63c6268131824e4d312a71`
- Upstream commit: `28e823d9df394c193dfc09f8eb891b77424e81c5`, `crates/io/udp`
- License: MIT OR Apache-2.0

Single-peer UDP sockets request a 4 MiB receive queue before linking and log both requested and effective `SO_RCVBUF`. Socket-option errors propagate through the existing link-start error boundary. An OS-capped effective size is reported, not assumed equal to the request or rejected as a fabricated minimum. No host sysctl changes, wire changes, extra retries, or timeout changes. Server listener code remains upstream.

Observed native client: the inherited 212992-byte kernel queue dropped 54,412 datagrams while server drops stayed zero; ordered Guide traffic held catalog/delta delivery behind missing fragments. A loopback regression sending 512 numbered 1200-byte packets before the next network drain received only packets0–91 with upstream code.

`socket2` provides portable borrowed-socket queue configuration (already present in consumer lockfiles). Primary API references: [SockRef receive-buffer methods](https://docs.rs/socket2/0.6.3/socket2/struct.SockRef.html#method.set_recv_buffer_size), [Linux socket(7) SO_RCVBUF](https://man7.org/linux/man-pages/man7/socket.7.html). Linux caps the request at `rmem_max` and returns a doubled bookkeeping-inclusive value; requested/effective byte counts are therefore distinct. This patch does not force privileged allocation. Burst pacing and Windows runtime measurement remain separate transport-owner gates. Other upstream code is unchanged. Retire when upstream provisions a queue sufficient for the same bounded burst or exposes equivalent configuration, preserving the loopback and full native delivery regressions.
