# Authored transport passengers

Transport motion and passenger movement retain the actual ship frame. Ship identity, source path, interior map, route knots, speed and authoritative progress cross replication; an attached unit retains its transport entity and ship-local feet/facing. Movement reports preserve that local frame while aboard and use world coordinates after disembark. Existing ordinary movement remains unattached, not a legacy transport fallback.

Seething Shore source transports 35/36 use entries 278407/279254, TaxiPath 6519/6520, speed 4 and interior maps 1856/1857. Both routes run on battleground map 1803. Safe locations 6337/6380 are ship-relative, never replacement world-space ground starts. Routes use pinned TrinityCore 15e0ed76 Catmull-Rom timing, including acceleration, stop delays and node arrival/departure events. Zero-stop routes retain constant-speed motion. Teleports and cross-map legs remain unsupported. The acceleration and per-node metadata extend the positional bitcode wire format: mixed client/server versions are incompatible.

Pure route and serialization proofs do not certify server attachment, native carrying, disembark, parachutes or live battleground entry. Those remain integration acceptance gates.
