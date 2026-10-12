# Housing wire contract

Requests use reliable ordered HousingChannel. Authenticate account and selected character server-side; no caller supplies an owner account, purchase price, storage quantity or private-copy ID.

- Static catalogs remain local and build-qualified. Replies identify their catalog build key; client/server mismatches must be refused.
- Plot list, neighborhood entry, cornerstone purchase, house entry/exit and account ownership are authoritative. Raw PlotIndex remains literal; plot costs follow Retail cornerstone MoneyFrame/GetMoney copper units.
- Account storage contains actual acquired counts, not StartingQuantity or catalog membership. One bounded authentic vendor-item/DB2 collect-effect redemption is allowed; consume the owned item and grant storage atomically.
- Basic edits identify a house, stable decor instance and finite house-local Bevy position/yaw. Scale stays authored. Only the owner edits; an allowed visitor receives the same layout, never edit rights or the owner's private account storage.
- Accepted edits are durable; reload returns the persisted layout. Remove restores the stored count. Move/rotate do not change quantity.

HouseAccess carries the literal Anyone, Neighbors, Guild, Friends and Party selections from Retail HouseSettings. SaveHouseAccess changes only house admission; plot access, blueprint access and owner assignment are not changed. An unset setting is unknown, not an invented Retail default or visitor permission. Clients never supply relationship claims; the server derives them from ownership, persisted social state and the current authoritative party. First-house exterior/interior grants remain unavailable until an authentic grant source is established; component membership and UnlockItemID=0 do not grant them.

References: local Retail Blizzard_HousingCornerstone.lua (plotCost→MoneyFrame/GetMoney); Blizzard_HousingHouseSettings.lua (HouseAccessAnyone); generated C_Housing*, C_HousingNeighborhood and C_HousingBasicMode documentation. Public IDs are independent of private account IDs and runtime map-copy IDs. Wire definitions alone do not prove gameplay, persistence or replication.
