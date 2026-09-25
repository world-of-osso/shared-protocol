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
fn merchant_messages_round_trip() {
    // Brother Danil (152) sells Refreshing Spring Water (159): 5 copper for 5.
    assert_wire_round_trip(&VendorInventory {
        npc: 4_294_967_301,
        can_repair: false,
        items: vec![VendorItem {
            slot: 0,
            item_id: 159,
            name: "Refreshing Spring Water".into(),
            quality: 1,
            price: 5,
            stack_count: 5,
            max_stack: 20,
            num_available: Some(3),
            usable: true,
        }],
    });
    assert_wire_round_trip(&BuybackList {
        items: vec![BuybackItem {
            slot: 0,
            item_id: 2379,
            name: "Tarnished Chain Vest".into(),
            quality: 1,
            count: 1,
            price: 17,
        }],
    });
    assert_wire_round_trip(&BuyItem {
        npc: 7,
        slot: 1,
        item_id: 2379,
        count: 2,
    });
    assert_wire_round_trip(&SellItem {
        npc: 7,
        item_guid: 90,
        count: 0,
    });
    assert_wire_round_trip(&BuybackItemRequest { npc: 7, slot: 11 });
    assert_wire_round_trip(&RepairItem {
        npc: 7,
        item_guid: None,
    });
    assert_wire_round_trip(&MerchantFailed {
        npc: 7,
        error: MerchantError::SoldOut,
    });
}

#[test]
fn merchant_errors_use_retail_wording() {
    assert_eq!(
        MerchantError::NotEnoughMoney.message(),
        "You don't have enough money."
    );
    assert_eq!(
        MerchantError::NotInterested.message(),
        "The merchant doesn't want that item."
    );
    assert_eq!(
        MerchantError::SoldOut.message(),
        "That item is currently sold out."
    );
}

#[test]
fn protocol_plugin_registers_merchant_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<VendorInventory>());
    assert!(app.is_message_registered::<BuybackList>());
    assert!(app.is_message_registered::<MerchantFailed>());
    assert!(app.is_message_registered::<BuyItem>());
    assert!(app.is_message_registered::<SellItem>());
    assert!(app.is_message_registered::<BuybackItemRequest>());
    assert!(app.is_message_registered::<RepairItem>());
}
