//! Mainline Trading Post authority; no client balance, price or collection grants.
use super::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TradingPostRequest {
    Query,
    Purchase { vendor_item_id: u32 },
    Refund { vendor_item_id: u32 },
    Freeze { vendor_item_id: Option<u32> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TradingPostCategory {
    pub id: u32,
    pub name: String,
    pub model_scene_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TradingPostItem {
    pub id: u32,
    pub category_id: u32,
    pub item_id: u32,
    pub name: String,
    pub description: String,
    pub icon_fdid: u32,
    pub quality: u8,
    pub price: u32,
    pub purchased: bool,
    pub collected: bool,
    pub refund_seconds: u32,
    pub ui_group_name: String,
    pub ui_group_priority: i32,
    pub model_scene_id: u32,
    pub creature_display_id: u32,
    /// Actual item IDs, including the members of an authored ensemble/arsenal.
    pub preview_item_ids: Vec<u32>,
    pub grant_unavailable: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TradingPostThreshold {
    pub id: u32,
    pub points: u32,
    pub tender: u32,
    pub claimed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TradingPostSnapshot {
    /// An actual active vendor interaction; Query outside a vendor cannot open one.
    pub npc_net_id: Option<u64>,
    pub tender: u32,
    pub points: u32,
    pub now_unix: i64,
    pub month_end_unix: i64,
    pub frozen_item_id: Option<u32>,
    pub categories: Vec<TradingPostCategory>,
    pub items: Vec<TradingPostItem>,
    pub thresholds: Vec<TradingPostThreshold>,
    pub source_gaps: Vec<String>,
    pub error: Option<String>,
}

pub(super) fn register(app: &mut App) {
    app.register_protocol_message::<TradingPostRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<TradingPostSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trading_post_wire_preserves_authored_price_and_unavailable_refund() {
        let row = TradingPostItem {
            id: 4,
            category_id: 1,
            item_id: 189869,
            name: "Distinguished Warcloak".into(),
            description: String::new(),
            icon_fdid: 0,
            quality: 1,
            price: 100,
            purchased: true,
            collected: true,
            refund_seconds: 0,
            ui_group_name: "New Arrivals".into(),
            ui_group_priority: 1000,
            model_scene_id: 583,
            creature_display_id: 0,
            preview_item_ids: vec![189869],
            grant_unavailable: None,
        };
        let bytes = serde_json::to_vec(&row).unwrap();
        assert_eq!(
            serde_json::from_slice::<TradingPostItem>(&bytes).unwrap(),
            row
        );
        for command in [
            TradingPostRequest::Query,
            TradingPostRequest::Purchase { vendor_item_id: 4 },
            TradingPostRequest::Refund { vendor_item_id: 4 },
            TradingPostRequest::Freeze {
                vendor_item_id: Some(4),
            },
            TradingPostRequest::Freeze {
                vendor_item_id: None,
            },
        ] {
            let bytes = serde_json::to_vec(&command).unwrap();
            assert_eq!(
                serde_json::from_slice::<TradingPostRequest>(&bytes).unwrap(),
                command
            );
        }
    }
}
