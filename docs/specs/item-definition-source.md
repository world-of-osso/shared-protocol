# Item definition source

Owned-item wire records distinguish Retail definitions from fixed Forever build 1.60.1.70205 definitions without renumbering authored item IDs or instance GUIDs. The shared enum lives at `shared::item_data::ItemDefinitionSource`.

## What it must do

- [x] Serialize exactly the supported sources `Retail` and `Forever70205`; equal authored IDs with different sources remain distinguishable after wire roundtrip.
- [x] Preserve source through `ItemStack`, owned bag/equipment snapshots and inventory deltas, including the receiving owner's post-transfer view.
- [x] Preserve source through replicated equipment appearance using both derived bitcode and serde wire encoding. An item-bearing entry pairs `item_id: Some(id)` with `definition_source: Some(source)`; display-only entries explicitly carry both as `None`.
- [x] Preserve owned source through trade views, auction sell-inventory/listing and browse items, and merchant buyback items.
- [x] Reject missing or unknown owned source; equipment appearance must explicitly supply source or null. Never infer source from race or substitute Retail on decode.

## How it works

- [Protocol agreement](protocol-check.md): coordinated client/server wire updates are intentional; obsolete schemas are unsupported.

## Implementation inventory

- `src/item_data.rs`: literal source enum, with serde, reflection and derived bitcode support.
- `src/protocol/inventory_messages.rs`: mandatory `ItemStack.definition_source`.
- `src/components.rs`: optional equipment appearance source paired with optional item ID; serde requires field presence.
- `src/protocol/gameplay_messages.rs`: mandatory source on `TradeItemSnapshot`, `AuctionInventoryItem` and `AuctionBrowseItem`; auction listings embed `AuctionInventoryItem`.
- `src/protocol/merchant_messages.rs`: mandatory `BuybackItem.definition_source` from the sold owned item.
- `src/transmog.rs`: display-only entries explicitly carry no item/source.

## Tests asserting this spec

- `tests/item_definition_source.rs`: same GUID/ID with different sources, bag/equipment/delta and appearance roundtrips, trade/auction/buyback views, missing/null/unknown-source rejection.
- Existing inventory, bank, mail and equipment appearance fixtures explicitly choose Retail.

Commerce source additions at `55cd79b` have retained 13 passing targeted wire tests (handoff evidence); independent final shared/client/server gates remain open.

Development proof at `391aace` (2026-10-05): `agent-run shared-item-source cargo test --locked --test item_definition_source` passed 6/6 after behavioral RED; `cargo test --locked --lib protocol::inventory_tests::inventory_snapshots_and_delta_round_trip` through the same runner passed 1/1 and compiled existing unit fixtures. No broad/final gate or client/server integration claim.

## Known gaps (current cycle)

- [ ] Final coordinated client/server integration remains open. Server immutable GUID markers, frozen pre-source character disk DTO and source-qualified runtime catalogs are implemented; engine source-local metadata and explicit IPC query source have bounded CPU proof. These are consumer responsibilities, not wire-roundtrip or native E2E acceptance. There is no ItemInfo query/response in this shared crate.
- [ ] Producers must keep optional equipment item/source fields paired; the wire record does not validate the pair.

## Out of scope

- Generic product/build registries, source inference, fallback compatibility, GUID/ID remapping.
- Auction filter semantics and `VendorItem`, `QuestRewardItem`, `LootContent` and `TooltipItem` without explicit item provenance. Known Retail producer lookup is deliberate, not fallback; unavailable Forever rewards remain a labelled gap.
- Client/server compilation, operations, deployment and end-to-end trade acceptance; main owns these gates.
