# Trading Post wire

Local Mainline PerksProgram API documentation defines categories, authored prices, preview data, owned/refundable state and one frozen item. The server owns progress, Tender, collection grants and refund expiry. Forever consumes exactly the same messages.

## What it must do

- [ ] Query, purchase, refund and freeze/clear requests carry only authored vendor IDs; never client-provided balance, price or grant.
- [ ] Snapshot carries actual active-vendor identity, balance, points, monthly expiry/clock, thresholds, categories, authored item/preview/UI group metadata, owned/receipt state, zero refundable seconds when disabled and explicit source/grant gaps.
- [ ] Registered messages travel server/client through existing channels.
- [x] `TradingPostItem.usable` carries the querying player's evaluated class/race/level item restrictions for Retail's Useable Only filter. `None` preserves unavailable source/player facts; it is not inferred from grant availability and does not prohibit account-wide purchases.

## How it works

- [Client contract](../../../../game-engine/docs/specs/trading-post.md).

## Implementation inventory

- `src/protocol/trading_post_messages.rs`: requests and server snapshots.
- `src/protocol.rs`: registration/export.

## Tests asserting this spec

- `trading_post_wire_preserves_authored_price_and_unavailable_refund`: JSON round-trip.

## Known gaps (current cycle)

- [ ] Real UDP transport/native integration proof pending.

## Out of scope

No invented seasonal rotation or refund policy.
