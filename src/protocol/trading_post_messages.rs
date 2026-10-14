//! Mainline Trading Post authority; no client balance, price or collection grants.
use super::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TradingPostRequest {
    Query,
    Purchase {
        vendor_item_id: u32,
    },
    Refund {
        vendor_item_id: u32,
    },
    Freeze {
        vendor_item_id: Option<u32>,
    },
    /// Traveler's Log: balance and thresholds only, no catalog rows.
    Balance,
    /// C_PerksProgram.RequestCartCheckout: authored vendor IDs only, never client prices.
    Checkout {
        vendor_item_ids: Vec<u32>,
    },
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
    /// Server-evaluated item class/race/level restrictions; None when source facts are unavailable.
    pub usable: Option<bool>,
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
    /// `items` is the whole catalog (vendor Query). Otherwise it holds only rows whose
    /// purchase/collection state changed; `merge` applies them to the last full catalog.
    pub catalog_complete: bool,
}

impl TradingPostSnapshot {
    /// Apply `update` to the last snapshot: a complete catalog replaces it, a partial one
    /// keeps the previous rows/categories and replaces rows by vendor item ID.
    pub fn merge(previous: Option<&TradingPostSnapshot>, update: TradingPostSnapshot) -> Self {
        let Some(previous) = previous.filter(|_| !update.catalog_complete) else {
            return update;
        };
        let mut items = previous.items.clone();
        for row in update.items {
            match items.iter_mut().find(|item| item.id == row.id) {
                Some(item) => *item = row,
                None => items.push(row),
            }
        }
        Self {
            items,
            categories: if update.categories.is_empty() {
                previous.categories.clone()
            } else {
                update.categories
            },
            catalog_complete: previous.catalog_complete,
            ..update
        }
    }
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
            usable: Some(true),
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
            TradingPostRequest::Balance,
            TradingPostRequest::Checkout {
                vendor_item_ids: vec![4, 473],
            },
        ] {
            let bytes = serde_json::to_vec(&command).unwrap();
            assert_eq!(
                serde_json::from_slice::<TradingPostRequest>(&bytes).unwrap(),
                command
            );
        }
    }

    fn snapshot(tender: u32, items: Vec<TradingPostItem>, complete: bool) -> TradingPostSnapshot {
        TradingPostSnapshot {
            npc_net_id: Some(9),
            tender,
            points: 0,
            now_unix: 0,
            month_end_unix: 0,
            frozen_item_id: None,
            categories: if complete {
                vec![TradingPostCategory {
                    id: 2,
                    name: "Mounts".into(),
                    model_scene_id: 418,
                }]
            } else {
                Vec::new()
            },
            items,
            thresholds: Vec::new(),
            source_gaps: Vec::new(),
            error: None,
            catalog_complete: complete,
        }
    }

    fn item(id: u32, purchased: bool) -> TradingPostItem {
        TradingPostItem {
            id,
            category_id: 2,
            item_id: 190000 + id,
            name: format!("Mount {id}"),
            description: String::new(),
            icon_fdid: 0,
            quality: 4,
            usable: Some(true),
            price: 650,
            purchased,
            collected: purchased,
            refund_seconds: 0,
            ui_group_name: String::new(),
            ui_group_priority: 0,
            model_scene_id: 0,
            creature_display_id: 0,
            preview_item_ids: Vec::new(),
            grant_unavailable: None,
        }
    }

    #[test]
    fn trading_post_partial_update_replaces_only_changed_rows_of_full_catalog() {
        let full = snapshot(4000, vec![item(1, false), item(2, false)], true);
        let merged =
            TradingPostSnapshot::merge(Some(&full), snapshot(3350, vec![item(2, true)], false));
        assert_eq!(merged.tender, 3350);
        assert!(merged.catalog_complete);
        assert_eq!(merged.categories, full.categories);
        assert_eq!(merged.items, vec![item(1, false), item(2, true)]);
        let balance = TradingPostSnapshot::merge(Some(&merged), snapshot(3350, Vec::new(), false));
        assert_eq!(balance.items, merged.items);
        let without_base =
            TradingPostSnapshot::merge(None, snapshot(3350, vec![item(2, true)], false));
        assert!(
            !without_base.catalog_complete,
            "a partial update never poses as the catalog"
        );
        let replaced =
            TradingPostSnapshot::merge(Some(&merged), snapshot(100, vec![item(1, true)], true));
        assert_eq!(replaced.items, vec![item(1, true)]);
    }
}
