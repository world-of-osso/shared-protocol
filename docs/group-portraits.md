# Group portraits

`GroupMemberSnapshot.portrait` carries an authoritative head independent of entity replication. Player entries use canonical race, customization/visage and equipped head; NPC follower entries set `creature_display_id` to the spawned creature's actual template-selected display. The client selects its existing creature renderer when that field is present. No synthesized player race or replicated-entity fallback.

The added optional display field changes the traced serde layout of `GroupRosterSnapshot`, so the existing protocol handshake rejects mismatched builds. No existing field's meaning changes.

[Engine contract](../../game-engine/docs/specs/group-frames.md) · [Follower contract](../../game-server/docs/specs/follower-dungeons.md).
