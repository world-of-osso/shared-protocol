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
        guild_repair_money: None,
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
            max_durability: None,
        }],
    });
    // Corina Steele (54) repairs and sells the Gladius (2488, 536 copper, durability 35);
    // a Member with 25g left of the guild's repair allowance.
    assert_wire_round_trip(&VendorInventory {
        npc: 4_294_967_302,
        can_repair: true,
        guild_repair_money: Some(250_000),
        items: vec![VendorItem {
            slot: 0,
            item_id: 2488,
            name: "Gladius".into(),
            quality: 1,
            price: 536,
            stack_count: 1,
            max_stack: 1,
            num_available: None,
            usable: true,
            max_durability: Some(35),
        }],
    });
    assert_wire_round_trip(&BuybackList {
        items: vec![BuybackItem {
            slot: 0,
            item_id: 2379,
            definition_source: crate::item_data::ItemDefinitionSource::Retail,
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
        destination: None,
    });
    // A merchant cursor dropped on backpack slot 5.
    assert_wire_round_trip(&BuyItem {
        npc: 7,
        slot: 0,
        item_id: 159,
        count: 1,
        destination: Some(ItemLocation::Bag { bag: 0, slot: 4 }),
    });
    assert_wire_round_trip(&SellAllJunkItems { npc: 7 });
    assert_wire_round_trip(&SellItem {
        npc: 7,
        item_guid: 90,
        count: 0,
    });
    assert_wire_round_trip(&BuybackItemRequest { npc: 7, slot: 11 });
    assert_wire_round_trip(&RepairItem {
        npc: 7,
        item_guid: None,
        guild_bank: false,
    });
    // MerchantGuildBankRepairButton: RepairAllItems(true).
    assert_wire_round_trip(&RepairItem {
        npc: 7,
        item_guid: None,
        guild_bank: true,
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
    assert_eq!(
        MerchantError::CantStack.message(),
        "This item cannot stack."
    );
    assert_eq!(
        MerchantError::GuildPermissions.message(),
        "You don't have permission to do that."
    );
    assert_eq!(
        MerchantError::GuildNotEnoughMoney.message(),
        "The guild bank does not have enough money"
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
    assert!(app.is_message_registered::<SellAllJunkItems>());
    assert!(app.is_message_registered::<BuybackItemRequest>());
    assert!(app.is_message_registered::<RepairItem>());
}
