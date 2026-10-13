# Authored transport passengers

Transport motion and passenger movement retain the actual ship frame. Ship identity, source path, interior map, route knots, speed and authoritative progress cross replication; an attached unit retains its transport entity and ship-local feet/facing. Movement reports preserve that local frame while aboard and use world coordinates after disembark. Existing ordinary movement remains unattached, not a legacy transport fallback.

Seething Shore source transports 35/36 use entries 278407/279254, TaxiPath 6519/6520, speed 4 and interior maps 1856/1857. Both routes run on battleground map 1803. Safe locations 6337/6380 are ship-relative, never replacement world-space ground starts. Constant-speed routes use pinned TrinityCore a352b1fa Catmull-Rom route timing and passenger coordinate transforms. Pause/teleport/event paths are outside current route support and must fail explicitly.

Pure route and serialization proofs do not certify server attachment, native carrying, disembark, parachutes or live battleground entry. Those remain integration acceptance gates.
