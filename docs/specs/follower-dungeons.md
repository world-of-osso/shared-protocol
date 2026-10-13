# Follower dungeon catalog

- `QueryFollowerDungeons` requests the server's prepared follower-difficulty dungeon entries over reliable `LfgChannel`.
- `FollowerDungeons` carries each entry's authored LFG ID, name and map ID. Only entries with prepared population and an authored entrance are published. An empty catalog is valid and must not create synthetic choices.
- Catalog membership is not admission. Existing `QueueForLfg` carries the selected role and dungeon ID; `LfgStateUpdate` remains authoritative for acceptance/refusal and the companion match roster.
- No skin-specific protocol or persisted NPC characters.
