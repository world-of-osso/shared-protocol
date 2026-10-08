use super::*;
use crate::ProtocolPlugin;
use bevy::state::app::StatesPlugin;
use core::time::Duration;
use lightyear::prelude::client::ClientPlugins;
use std::panic::{AssertUnwindSafe, catch_unwind};

fn protocol_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_millis(10),
    });
    app.add_plugins(ProtocolPlugin);
    app
}

/// Registered types outside lightyear, whose own types carry no layout.
fn shared_types<K: TypeKind>(mapper: &TypeMapper<K>) -> usize {
    (0..)
        .map_while(|net_id| mapper.kind(net_id))
        .filter(|kind| !mapper.name(kind).unwrap().starts_with("lightyear"))
        .count()
}

#[test]
fn every_shared_message_and_component_has_a_traced_layout() {
    let mut app = protocol_app();
    app.finish();
    let layouts = app.world().resource::<ProtocolLayouts>();
    let messages = shared_types(&app.world().resource::<MessageRegistry>().kind_map);
    let components = shared_types(&app.world().resource::<ComponentRegistry>().kind_map);
    assert!(messages > 100, "only {messages} shared messages registered");
    assert!(components > 20, "only {components} shared components");
    assert_eq!(layouts.messages.len(), messages);
    assert_eq!(layouts.components.len(), components);
    assert!(app.world().get_resource::<LocalLayouts>().is_some());
}

#[derive(Serialize, serde::Deserialize, Clone, Debug)]
struct StrayMessage(u32);

#[test]
fn message_registered_without_its_layout_stops_the_app_at_finish() {
    let mut app = protocol_app();
    app.register_message::<StrayMessage>();
    let panic = catch_unwind(AssertUnwindSafe(|| app.finish())).expect_err("finish accepted");
    let message = panic
        .downcast_ref::<String>()
        .expect("panic message")
        .clone();
    assert!(
        message.contains(
            "StrayMessage is registered without its layout; use register_protocol_message"
        ),
        "{message}"
    );
}

mod before {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Probe {
        pub id: u32,
        pub outer: Vec<Outer>,
    }

    #[derive(Serialize, Deserialize)]
    pub enum Outer {
        Plain,
        Wrapped(Option<Inner>),
    }

    #[derive(Serialize, Deserialize)]
    pub enum Inner {
        Low,
        High { level: u8 },
    }
}

mod same {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Probe {
        pub id: u32,
        pub outer: Vec<Outer>,
    }

    #[derive(Serialize, Deserialize)]
    pub enum Outer {
        Plain,
        Wrapped(Option<Inner>),
    }

    #[derive(Serialize, Deserialize)]
    pub enum Inner {
        Low,
        High { level: u8 },
    }
}

/// `Inner` gains a variant; it is only reachable through `Outer::Wrapped`.
mod added_nested_variant {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Probe {
        pub id: u32,
        pub outer: Vec<Outer>,
    }

    #[derive(Serialize, Deserialize)]
    pub enum Outer {
        Plain,
        Wrapped(Option<Inner>),
    }

    #[derive(Serialize, Deserialize)]
    pub enum Inner {
        Low,
        High { level: u8 },
        Max,
    }
}

/// `Inner::High` gains a field.
mod added_variant_field {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Probe {
        pub id: u32,
        pub outer: Vec<Outer>,
    }

    #[derive(Serialize, Deserialize)]
    pub enum Outer {
        Plain,
        Wrapped(Option<Inner>),
    }

    #[derive(Serialize, Deserialize)]
    pub enum Inner {
        Low,
        High { level: u8, cap: u8 },
    }
}

#[test]
fn identical_shapes_hash_equal_across_modules() {
    assert_eq!(layout_hash::<before::Probe>(), layout_hash::<same::Probe>());
}

#[test]
fn nested_enum_changes_change_the_layout() {
    let before = layout_hash::<before::Probe>();
    assert_ne!(before, layout_hash::<added_nested_variant::Probe>());
    assert_ne!(before, layout_hash::<added_variant_field::Probe>());
}

#[test]
fn auction_query_layout_traces_subcategory_alternatives_and_new_sorts() {
    use crate::protocol::{
        AuctionItemFilter, AuctionSearchQuery, AuctionSortField, QueryAuctionBrowse, QueryAuctions,
    };
    #[derive(Serialize, serde::Deserialize)]
    struct BeforeQuery {
        item_id: Option<u32>,
        class_id: Option<u8>,
        text: String,
        page: u32,
        page_size: u32,
        min_level: Option<u16>,
        max_level: Option<u16>,
        quality: Option<u8>,
        usable_only: bool,
        sort_field: BeforeSort,
        sort_dir: crate::protocol::AuctionSortDir,
        faction: u8,
    }
    #[derive(Serialize, serde::Deserialize)]
    enum BeforeSort {
        Name,
        MinBid,
        Buyout,
        TimeLeft,
        Quality,
        RequiredLevel,
    }
    #[derive(Serialize, serde::Deserialize)]
    struct WithFilters {
        item_id: Option<u32>,
        class_id: Option<u8>,
        subcategory_filters: Vec<AuctionItemFilter>,
        text: String,
        page: u32,
        page_size: u32,
        min_level: Option<u16>,
        max_level: Option<u16>,
        quality: Option<u8>,
        usable_only: bool,
        sort_field: BeforeSort,
        sort_dir: crate::protocol::AuctionSortDir,
        faction: u8,
    }
    assert_ne!(layout_hash::<BeforeQuery>(), layout_hash::<WithFilters>());
    assert_ne!(
        layout_hash::<WithFilters>(),
        layout_hash::<AuctionSearchQuery>()
    );
    assert_ne!(
        layout_hash::<BeforeSort>(),
        layout_hash::<AuctionSortField>()
    );
    assert_eq!(
        layout_hash::<QueryAuctionBrowse>(),
        layout_hash::<QueryAuctions>()
    );
}
