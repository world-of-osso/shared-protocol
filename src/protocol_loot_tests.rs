use super::*;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::AppMessageExt;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

fn assert_wire_round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(value, config).unwrap();
    let (decoded, read): (T, usize) = bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(&decoded, value);
}

#[test]
fn loot_messages_round_trip() {
    // A Kobold Vermin (6) corpse with 3 copper and a Melted Candle (755).
    assert_wire_round_trip(&LootResponse {
        corpse: 4_294_967_301,
        auto: false,
        slots: vec![
            LootSlot {
                slot: 0,
                content: LootContent::Money { copper: 3 },
            },
            LootSlot {
                slot: 1,
                content: LootContent::Item {
                    item_id: 755,
                    name: "Melted Candle".into(),
                    quality: 0,
                    count: 1,
                },
            },
        ],
    });
    assert_wire_round_trip(&LootUnit {
        corpse: 7,
        auto: true,
    });
    assert_wire_round_trip(&LootSlotRequest { corpse: 7, slot: 1 });
    assert_wire_round_trip(&LootRelease { corpse: 7 });
    assert_wire_round_trip(&LootSlotRemoved { corpse: 7, slot: 1 });
    assert_wire_round_trip(&LootClosed { corpse: 7 });
    assert_wire_round_trip(&CorpseLootable {
        corpse: 7,
        lootable: false,
    });
    assert_wire_round_trip(&LootFailed {
        corpse: 7,
        error: LootError::InventoryFull,
    });
}

#[test]
fn loot_errors_use_retail_wording() {
    assert_eq!(
        LootError::TooFar.message(),
        "You are too far away to loot that corpse."
    );
    assert_eq!(
        LootError::NoPermission.message(),
        "You don't have permission to loot that corpse."
    );
    assert_eq!(LootError::InventoryFull.message(), "Inventory is full.");
}

#[test]
fn protocol_plugin_registers_loot_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<LootUnit>());
    assert!(app.is_message_registered::<LootSlotRequest>());
    assert!(app.is_message_registered::<LootRelease>());
    assert!(app.is_message_registered::<LootResponse>());
    assert!(app.is_message_registered::<LootSlotRemoved>());
    assert!(app.is_message_registered::<LootClosed>());
    assert!(app.is_message_registered::<CorpseLootable>());
    assert!(app.is_message_registered::<LootFailed>());
}
