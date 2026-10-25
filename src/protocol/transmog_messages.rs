//! Authoritative Transmogrifier service. Sources are ItemModifiedAppearance IDs;
//! visuals are ItemAppearance IDs, never ItemDisplayInfo IDs. A quote names each
//! equipped GUID so a pending change cannot silently apply to replacement gear.
#[cfg(feature = "transport")]
use bevy::prelude::*;
#[cfg(feature = "transport")]
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

use super::EquipmentSlot;
#[cfg(feature = "transport")]
use super::ProtocolRegistrationExt;

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
    /// Retail Enum.TransmogSource, authored ItemModifiedAppearance.TransmogSourceTypeEnum.
    pub source_type: u8,
    pub appearance_id: u32,
    pub item_id: u32,
    pub display_id: u32,
    pub icon_fdid: u32,
    pub name: String,
    pub inventory_type: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransmogSlot {
    pub slot: EquipmentSlot,
    pub item_guid: u64,
    pub item_id: u32,
    pub definition_source: crate::item_data::ItemDefinitionSource,
    pub inventory_type: u8,
    pub applied: TransmogChoice,
    /// Target cost in copper; None means local data cannot define this price.
    pub cost: Option<u64>,
    pub can_hide: bool,
    pub sources: Vec<TransmogSource>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct TransmogState {
    pub npc: u64,
    /// Paid Apply still validates only these collected eligible choices.
    pub slots: Vec<TransmogSlot>,
    /// Started sets; their primary appearances carry the collection states.
    pub sets: Vec<super::TransmogSet>,
    pub outfit_entries: Vec<super::TransmogOutfitEntry>,
    pub outfits: Vec<super::TransmogOutfit>,
    pub custom_sets: Vec<super::TransmogCustomSet>,
    pub illusions: Vec<super::TransmogIllusion>,
    pub outfit_slots: Vec<super::TransmogOutfitSlot>,
    pub outfit_options: Vec<super::TransmogOutfitOption>,
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

#[cfg(feature = "transport")]
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
    app.register_protocol_message::<super::SaveTransmogCustomSet>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<super::SaveTransmogOutfit>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<super::TransmogOutfitFailed>()
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
                definition_source: crate::item_data::ItemDefinitionSource::Retail,
                inventory_type: 20,
                applied: TransmogChoice::Source(821),
                cost: None,
                can_hide: true,
                sources: vec![TransmogSource {
                    source_id: 821,
                    source_type: 3,
                    appearance_id: 154,
                    item_id: 2488,
                    display_id: 1542,
                    icon_fdid: 135274,
                    name: "Gladius".into(),
                    inventory_type: 13,
                }],
            }],
            ..Default::default()
        };
        // Enum.TransmogSource.Vendor = 3 for authentic Gladius source821.
        let json = serde_json::to_value(&state).unwrap();
        assert_eq!(json["slots"][0]["sources"][0]["source_type"], 3);
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
