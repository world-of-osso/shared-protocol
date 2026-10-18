//! Received persistent-area geometry; lifetime is ordinary entity replication.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Active70448 SpellEffect88312: Lord Overheat's persistent damaging area.
pub const RAIN_OF_FIRE_SPELL_ID: u32 = 86_636;

/// The caster's dynamic object (`DYNAMICOBJECT_CASTER`, spell and effect radius).
/// Its replicated Position is the actual resolved destination. Removal, including
/// visibility loss, removes the area; clients must not predict its expiry.
#[derive(Component, Reflect, Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct DynamicObjectInfo {
    pub caster: u64,
    pub spell_id: u32,
    pub radius: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rainfire_dynamic_object_geometry_preserves_server_identity_on_wire() {
        let area = DynamicObjectInfo {
            caster: 4_294_739_329,
            spell_id: RAIN_OF_FIRE_SPELL_ID,
            radius: 5.0,
        };
        let bytes = bincode::serde::encode_to_vec(area, bincode::config::standard()).unwrap();
        let (received, count): (DynamicObjectInfo, usize) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
        assert_eq!(received, area);
        assert_eq!(count, bytes.len());
    }
}
