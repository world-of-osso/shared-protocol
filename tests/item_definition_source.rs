use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use shared::components::EquipmentAppearance;
use shared::protocol::{
    BagContents, BagSlotItem, EquipmentSlot, EquipmentSnapshot, EquippedItem, InventoryDelta,
    InventorySlotChange, InventorySnapshot, ItemLocation, ItemStack,
};
use std::fmt::Debug;

fn wire_round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) -> T {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(value, config).unwrap();
    let (decoded, read): (T, usize) = bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(&decoded, value);
    decoded
}

fn stack_json(source: &str) -> Value {
    json!({
        "item_guid": 50, "item_id": 2947, "definition_source": source,
        "count": 1, "durability": null, "soulbound": false
    })
}

fn stack(source: &str) -> ItemStack {
    serde_json::from_value(stack_json(source)).unwrap()
}

#[test]
fn item_definition_source_colliding_ids_remain_distinct_on_wire() {
    let retail = stack("Retail");
    let forever = stack("Forever70205");
    let config = bincode::config::standard();
    assert_ne!(
        bincode::serde::encode_to_vec(&retail, config).unwrap(),
        bincode::serde::encode_to_vec(&forever, config).unwrap()
    );
    for (source, item) in [("Retail", retail), ("Forever70205", forever)] {
        let decoded = wire_round_trip(&item);
        assert_eq!(serde_json::to_value(decoded).unwrap(), stack_json(source));
    }
}

#[test]
fn item_definition_source_survives_owned_bag_equipment_and_transfer_delta() {
    for source in ["Retail", "Forever70205"] {
        let item = stack(source);
        let bags = wire_round_trip(&InventorySnapshot {
            bags: vec![BagContents {
                bag: 0,
                size: 16,
                items: vec![BagSlotItem {
                    slot: 3,
                    item: item.clone(),
                }],
            }],
        });
        assert_eq!(
            serde_json::to_value(&bags.bags[0].items[0].item).unwrap(),
            stack_json(source)
        );
        let equipment = wire_round_trip(&EquipmentSnapshot {
            items: vec![EquippedItem {
                slot: EquipmentSlot::MainHand,
                item: item.clone(),
            }],
        });
        assert_eq!(
            serde_json::to_value(&equipment.items[0].item).unwrap(),
            stack_json(source)
        );
        let transfer = wire_round_trip(&InventoryDelta {
            changes: vec![InventorySlotChange {
                location: ItemLocation::Bag { bag: 0, slot: 5 },
                item: Some(item),
            }],
        });
        assert_eq!(
            serde_json::to_value(&transfer.changes[0].item).unwrap(),
            stack_json(source)
        );
    }
}

#[test]
fn item_definition_source_equipment_appearance_preserves_colliding_ids() {
    let authored = json!({"entries": [
        {"slot": "MainHand", "item_id": 2947, "definition_source": "Retail",
         "display_info_id": null, "inventory_type": 25, "hidden": false},
        {"slot": "OffHand", "item_id": 2947, "definition_source": "Forever70205",
         "display_info_id": null, "inventory_type": 25, "hidden": false},
        {"slot": "Head", "item_id": null, "definition_source": null,
         "display_info_id": 12345, "inventory_type": 1, "hidden": false}
    ]});
    let appearance: EquipmentAppearance = serde_json::from_value(authored.clone()).unwrap();
    let serde_decoded = wire_round_trip(&appearance);
    assert_eq!(serde_json::to_value(serde_decoded).unwrap(), authored);
    let bytes = bitcode::encode(&appearance);
    let decoded: EquipmentAppearance = bitcode::decode(&bytes).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), authored);
}

#[test]
fn item_definition_source_missing_or_unknown_owned_source_is_rejected() {
    let mut missing = stack_json("Retail");
    missing.as_object_mut().unwrap().remove("definition_source");
    assert!(serde_json::from_value::<ItemStack>(missing).is_err());
    assert!(serde_json::from_value::<ItemStack>(stack_json("Forever70206")).is_err());
    let mut null = stack_json("Retail");
    null["definition_source"] = Value::Null;
    assert!(serde_json::from_value::<ItemStack>(null).is_err());
}

#[test]
fn item_definition_source_trade_and_auction_views_preserve_owned_identity() {
    use shared::protocol::{AuctionInventoryItem, TradeItemSnapshot};
    for source in ["Retail", "Forever70205"] {
        let trade = json!({
            "item_guid": 50, "item_id": 2947, "definition_source": source,
            "name": "Authored item", "quality": 1, "stack_count": 1
        });
        let value: TradeItemSnapshot = serde_json::from_value(trade.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(wire_round_trip(&value)).unwrap(),
            trade
        );
        let auction = json!({
            "item_guid": 50, "item_id": 2947, "definition_source": source,
            "name": "Authored item", "quality": 1, "stack_count": 1,
            "required_level": 1, "vendor_sell_price": 10
        });
        let value: AuctionInventoryItem = serde_json::from_value(auction.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(wire_round_trip(&value)).unwrap(),
            auction
        );
    }
}

#[test]
fn item_definition_source_appearance_requires_explicit_source_or_null() {
    let missing = json!({"entries": [{
        "slot": "Head", "item_id": null, "display_info_id": 12345,
        "inventory_type": 1, "hidden": false
    }]});
    assert!(serde_json::from_value::<EquipmentAppearance>(missing).is_err());
    let unknown = json!({"entries": [{
        "slot": "MainHand", "item_id": 2947, "definition_source": "Forever70206",
        "display_info_id": null, "inventory_type": 25, "hidden": false
    }]});
    assert!(serde_json::from_value::<EquipmentAppearance>(unknown).is_err());
}
