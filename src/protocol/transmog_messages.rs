//! Authoritative Transmogrifier service. Sources are ItemModifiedAppearance IDs;
//! visuals are ItemAppearance IDs, never ItemDisplayInfo IDs. A quote names each
//! equipped GUID so a pending change cannot silently apply to replacement gear.
use bevy::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

use super::{EquipmentSlot, ProtocolRegistrationExt};

pub struct TransmogChannel;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TransmogChoice {
    #[default]
    Original,
    Source(u32),
    Hidden,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogSource {
    pub source_id: u32,
    pub appearance_id: u32,
    pub item_id: u32,
    pub display_id: u32,
    pub name: String,
    pub inventory_type: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogSlot {
    pub slot: EquipmentSlot,
    pub item_guid: u64,
    pub item_id: u32,
    pub applied: TransmogChoice,
    /// Target cost in copper; None means local data cannot define this price.
    pub cost: Option<u64>,
    pub can_hide: bool,
    pub sources: Vec<TransmogSource>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogState {
    pub npc: u64,
    pub slots: Vec<TransmogSlot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RequestTransmog {
    pub npc: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogChange {
    pub slot: EquipmentSlot,
    pub item_guid: u64,
    pub choice: TransmogChoice,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ApplyTransmog {
    pub npc: u64,
    pub changes: Vec<TransmogChange>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransmogError {
    NotInteracting,
    InvalidSlot,
    ItemChanged,
    AppearanceNotUsable,
    CostUnavailable,
    NotEnoughMoney,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogFailed {
    pub npc: u64,
    pub error: TransmogError,
}

pub(super) fn register_transmog_protocol(app: &mut App) {
    app.add_channel::<TransmogChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_protocol_message::<RequestTransmog>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<ApplyTransmog>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<TransmogState>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<TransmogFailed>()
        .add_direction(NetworkDirection::ServerToClient);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transmog_wire_preserves_source_guid_hide_and_unknown_cost() {
        let state = TransmogState {
            npc: 54442,
            slots: vec![TransmogSlot {
                slot: EquipmentSlot::Chest,
                item_guid: 9001,
                item_id: 2572,
                applied: TransmogChoice::Source(821),
                cost: None,
                can_hide: true,
                sources: vec![TransmogSource {
                    source_id: 821,
                    appearance_id: 154,
                    item_id: 2488,
                    display_id: 3667,
                    name: "Gladius".into(),
                    inventory_type: 13,
                }],
            }],
        };
        let bytes = bincode::serde::encode_to_vec(&state, bincode::config::standard()).unwrap();
        let (decoded, consumed): (TransmogState, usize) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
        assert_eq!(decoded, state);
        assert_eq!(consumed, bytes.len());
        let request = ApplyTransmog {
            npc: 54442,
            changes: vec![TransmogChange {
                slot: EquipmentSlot::Chest,
                item_guid: 9001,
                choice: TransmogChoice::Hidden,
            }],
        };
        let bytes = bincode::serde::encode_to_vec(&request, bincode::config::standard()).unwrap();
        let (decoded, _): (ApplyTransmog, usize) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
        assert_eq!(decoded, request);
    }
}
