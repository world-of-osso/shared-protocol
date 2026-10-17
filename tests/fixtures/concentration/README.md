# Concentration fixture

Currency rows are unchanged Retail exports, not synthetic values. `CurrencyTypes.csv` source is identical in 12.1.5.70391 and 12.1.5.70448: SHA-256 `b4b20475ef1952fb457a289ef487a835ba6cce2e4e7c9af0e557628df4140009`.

Every nonzero concentration currency referenced by the fixture's ProfessionExpansion rows must be present: Leatherworking2830→3049 and Enchanting2874→3046. The loader validates the complete supplied catalog, not only the skill queried by a test.
