//! Mainline Trading Post authority; no client balance, price or collection grants.
#[cfg(feature = "transport")]
use super::ProtocolRegistrationExt;
use bevy::prelude::*;
#[cfg(feature = "transport")]
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

/// The request whose authoritative reply this snapshot carries, not a request ID.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradingPostReplyKind {
    Query,
    Purchase,
    Refund,
    Freeze,
    Balance,
    Checkout,
}

impl TradingPostRequest {
    pub fn reply_kind(&self) -> TradingPostReplyKind {
        match self {
            Self::Query => TradingPostReplyKind::Query,
            Self::Purchase { .. } => TradingPostReplyKind::Purchase,
            Self::Refund { .. } => TradingPostReplyKind::Refund,
            Self::Freeze { .. } => TradingPostReplyKind::Freeze,
            Self::Balance => TradingPostReplyKind::Balance,
            Self::Checkout { .. } => TradingPostReplyKind::Checkout,
        }
    }
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
    /// Supplied by the server even for a refusal; catalog refresh is not purchase completion.
    pub reply_kind: TradingPostReplyKind,
}

impl TradingPostSnapshot {
    /// Apply `update` to the last snapshot: a complete catalog replaces it, a partial one
    /// keeps the previous rows/categories and replaces rows by vendor item ID.
    pub fn merge(previous: Option<&TradingPostSnapshot>, update: TradingPostSnapshot) -> Self {
        let Some(previous) = previous.filter(|_| !update.catalog_complete) else {
            return update;
        };
        let mut items = previous.items.clone();
        // Retained durations belong to the previous server timestamp, not the new delta.
        let elapsed = update
            .now_unix
            .saturating_sub(previous.now_unix)
            .clamp(0, i64::from(u32::MAX)) as u32;
        for row in &mut items {
            row.refund_seconds = row.refund_seconds.saturating_sub(elapsed);
        }
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

#[cfg(feature = "transport")]
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

    fn snapshot(
        tender: u32,
        items: Vec<TradingPostItem>,
        reply_kind: TradingPostReplyKind,
    ) -> TradingPostSnapshot {
        let complete = reply_kind == TradingPostReplyKind::Query;
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
            reply_kind,
        }
    }

    #[test]
    fn trading_post_reply_kinds_round_trip_and_survive_snapshot_merge() {
        let cases = [
            (TradingPostRequest::Query, TradingPostReplyKind::Query),
            (
                TradingPostRequest::Purchase { vendor_item_id: 4 },
                TradingPostReplyKind::Purchase,
            ),
            (
                TradingPostRequest::Refund { vendor_item_id: 4 },
                TradingPostReplyKind::Refund,
            ),
            (
                TradingPostRequest::Freeze {
                    vendor_item_id: Some(4),
                },
                TradingPostReplyKind::Freeze,
            ),
            (
                TradingPostRequest::Freeze {
                    vendor_item_id: None,
                },
                TradingPostReplyKind::Freeze,
            ),
            (TradingPostRequest::Balance, TradingPostReplyKind::Balance),
            (
                TradingPostRequest::Checkout {
                    vendor_item_ids: vec![4, 5],
                },
                TradingPostReplyKind::Checkout,
            ),
        ];
        for (request, kind) in cases {
            assert_eq!(request.reply_kind(), kind);
            let update = snapshot(500, Vec::new(), kind);
            let bytes = serde_json::to_vec(&update).unwrap();
            let received: TradingPostSnapshot = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(received, update);
            let previous = snapshot(600, vec![item(4, false)], TradingPostReplyKind::Query);
            let merged = TradingPostSnapshot::merge(Some(&previous), received);
            assert_eq!(merged.reply_kind, kind);
        }
    }

    #[test]
    fn trading_post_snapshot_rejects_missing_reply_kind() {
        let data = snapshot(500, Vec::new(), TradingPostReplyKind::Query);
        let mut value = serde_json::to_value(data).unwrap();
        value.as_object_mut().unwrap().remove("reply_kind");
        assert!(serde_json::from_value::<TradingPostSnapshot>(value).is_err());
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
    fn trading_post_partial_update_keeps_retained_refund_expiry_on_the_server_clock() {
        let mut full = snapshot(
            1000,
            vec![item(4, true), item(5, false)],
            TradingPostReplyKind::Query,
        );
        full.now_unix = 1790812900;
        full.items[0].refund_seconds = 60;
        let mut delta = snapshot(925, vec![item(5, true)], TradingPostReplyKind::Purchase);
        delta.now_unix = 1790812920;
        delta.items[0].refund_seconds = 600;
        let merged = TradingPostSnapshot::merge(Some(&full), delta);
        assert_eq!(
            merged.items[0].refund_seconds, 40,
            "retained row must not receive a fresh60s window"
        );
        assert_eq!(
            merged.items[1].refund_seconds, 600,
            "new row's duration is already relative to new server time"
        );
        let mut freeze = snapshot(925, Vec::new(), TradingPostReplyKind::Freeze);
        freeze.now_unix = 1790812959;
        freeze.frozen_item_id = Some(4);
        let merged = TradingPostSnapshot::merge(Some(&merged), freeze);
        assert_eq!(merged.items[0].refund_seconds, 1);
        assert_eq!(merged.items[1].refund_seconds, 561);
        let mut balance = snapshot(925, Vec::new(), TradingPostReplyKind::Balance);
        balance.now_unix = 1790812960;
        let expired = TradingPostSnapshot::merge(Some(&merged), balance);
        assert_eq!(expired.items[0].refund_seconds, 0);
        assert_eq!(expired.items[1].refund_seconds, 560);
    }

    #[test]
    fn trading_post_partial_update_replaces_only_changed_rows_of_full_catalog() {
        let full = snapshot(
            4000,
            vec![item(1, false), item(2, false)],
            TradingPostReplyKind::Query,
        );
        let merged = TradingPostSnapshot::merge(
            Some(&full),
            snapshot(3350, vec![item(2, true)], TradingPostReplyKind::Purchase),
        );
        assert_eq!(merged.tender, 3350);
        assert!(merged.catalog_complete);
        assert_eq!(merged.categories, full.categories);
        assert_eq!(merged.items, vec![item(1, false), item(2, true)]);
        let balance = TradingPostSnapshot::merge(
            Some(&merged),
            snapshot(3350, Vec::new(), TradingPostReplyKind::Balance),
        );
        assert_eq!(balance.items, merged.items);
        let without_base = TradingPostSnapshot::merge(
            None,
            snapshot(3350, vec![item(2, true)], TradingPostReplyKind::Purchase),
        );
        assert!(
            !without_base.catalog_complete,
            "a partial update never poses as the catalog"
        );
        let replaced = TradingPostSnapshot::merge(
            Some(&merged),
            snapshot(100, vec![item(1, true)], TradingPostReplyKind::Query),
        );
        assert_eq!(replaced.items, vec![item(1, true)]);
    }
}
