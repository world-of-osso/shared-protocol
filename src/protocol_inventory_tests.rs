use super::*;
use crate::components::EquipmentVisualSlot;
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

fn linen(count: u32) -> ItemStack {
    ItemStack {
        item_guid: 50,
        item_id: 2589,
        definition_source: crate::item_data::ItemDefinitionSource::Retail,
        count,
        durability: None,
        soulbound: false,
    }
}

#[test]
fn inventory_snapshots_and_delta_round_trip() {
    assert_wire_round_trip(&InventorySnapshot {
        bags: vec![
            BagContents {
                bag: BACKPACK_BAG,
                size: BACKPACK_SLOTS,
                items: vec![BagSlotItem {
                    slot: 3,
                    item: linen(20),
                }],
            },
            BagContents {
                bag: 1,
                size: 0,
                items: vec![],
            },
        ],
    });
    assert_wire_round_trip(&EquipmentSnapshot {
        items: vec![EquippedItem {
            slot: EquipmentSlot::MainHand,
            item: ItemStack {
                item_guid: 51,
                item_id: 25,
                definition_source: crate::item_data::ItemDefinitionSource::Retail,
                count: 1,
                durability: Some(ItemDurability {
                    current: 30,
                    max: 35,
                }),
                soulbound: true,
            },
        }],
    });
    assert_wire_round_trip(&InventoryDelta {
        changes: vec![
            InventorySlotChange {
                location: ItemLocation::Bag { bag: 0, slot: 3 },
                item: None,
            },
            InventorySlotChange {
                location: ItemLocation::Equipment(EquipmentSlot::Finger2),
                item: Some(linen(5)),
            },
        ],
    });
}

#[test]
fn inventory_requests_and_error_round_trip() {
    let bag = ItemLocation::Bag { bag: 0, slot: 1 };
    let hand = ItemLocation::Equipment(EquipmentSlot::MainHand);
    assert_wire_round_trip(&SwapItem {
        from: bag,
        to: hand,
    });
    assert_wire_round_trip(&SplitItem {
        from: bag,
        to: ItemLocation::Bag { bag: 0, slot: 2 },
        count: 5,
    });
    assert_wire_round_trip(&DestroyItem {
        location: bag,
        count: 0,
    });
    assert_wire_round_trip(&UseItem {
        location: bag,
        target: Some(42),
    });
    assert_wire_round_trip(&EquipItem { from: bag });
    assert_wire_round_trip(&SortBags);
    assert_wire_round_trip(&InventoryError {
        reason: InventoryErrorReason::WrongSlot,
        detail: Some("inventory type 5 cannot go in MainHand".into()),
    });
}

#[test]
fn inventory_errors_use_retail_wording() {
    assert_eq!(
        InventoryErrorReason::WrongSlot.message(),
        "That item does not go in that slot."
    );
    assert_eq!(
        InventoryErrorReason::CantEquipLevelI { level: 10 }.message(),
        "You must reach level 10 to use that item."
    );
    assert_eq!(
        InventoryErrorReason::InvFull.message(),
        "Inventory is full."
    );
}

#[test]
fn equipment_slots_use_retail_invslot_ids() {
    let ids: Vec<u8> = EquipmentSlot::ALL.iter().map(|s| s.inv_slot_id()).collect();
    assert_eq!(ids, (1..=19).collect::<Vec<u8>>());
    assert_eq!(EquipmentSlot::MainHand.inv_slot_id(), 16);
    assert_eq!(EquipmentSlot::Tabard.inv_slot_id(), 19);
}

#[test]
fn bag_slots_use_retail_container_inventory_ids() {
    let ids: Vec<u8> = EquipmentSlot::BAGS
        .iter()
        .map(|s| s.inv_slot_id())
        .collect();
    assert_eq!(ids, vec![31, 32, 33, 34, 35]);
    assert_eq!(EquipmentSlot::Bag3.bag_index(), Some(3));
    assert_eq!(EquipmentSlot::ReagentBag.bag_index(), Some(REAGENT_BAG));
    assert_eq!(EquipmentSlot::Head.bag_index(), None);
    assert_eq!(EquipmentSlot::from_bag_index(1), Some(EquipmentSlot::Bag1));
    assert_eq!(EquipmentSlot::from_bag_index(BACKPACK_BAG), None);
    assert_eq!(EquipmentSlot::from_bag_index(BAG_COUNT), None);
}

#[test]
fn inventory_types_map_to_retail_equipment_slots() {
    use EquipmentSlot::*;
    assert_eq!(EquipmentSlot::for_inventory_type(1), &[Head]);
    assert_eq!(EquipmentSlot::for_inventory_type(20), &[Chest]);
    assert_eq!(EquipmentSlot::for_inventory_type(11), &[Finger1, Finger2]);
    assert_eq!(EquipmentSlot::for_inventory_type(13), &[MainHand, OffHand]);
    assert_eq!(EquipmentSlot::for_inventory_type(17), &[MainHand]);
    assert_eq!(EquipmentSlot::for_inventory_type(26), &[MainHand]);
    assert_eq!(EquipmentSlot::for_inventory_type(14), &[OffHand]);
    assert_eq!(
        EquipmentSlot::for_inventory_type(18),
        &[Bag1, Bag2, Bag3, Bag4]
    );
    for non_equip in [0, 24, 27, 29, 255] {
        assert!(EquipmentSlot::for_inventory_type(non_equip).is_empty());
    }
}

#[test]
fn visual_slots_round_trip_through_equipment_slots() {
    use EquipmentVisualSlot as V;
    for visual in [
        V::Head,
        V::Shoulder,
        V::Back,
        V::Chest,
        V::Shirt,
        V::Tabard,
        V::Wrist,
        V::Hands,
        V::Waist,
        V::Legs,
        V::Feet,
        V::MainHand,
        V::OffHand,
    ] {
        assert_eq!(
            EquipmentSlot::from_visual_slot(visual).visual_slot(),
            Some(visual)
        );
    }
    assert_eq!(EquipmentSlot::Neck.visual_slot(), None);
    assert_eq!(EquipmentSlot::Trinket2.visual_slot(), None);
}

#[test]
fn protocol_plugin_registers_inventory_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<InventorySnapshot>());
    assert!(app.is_message_registered::<EquipmentSnapshot>());
    assert!(app.is_message_registered::<InventoryDelta>());
    assert!(app.is_message_registered::<InventoryError>());
    assert!(app.is_message_registered::<SwapItem>());
    assert!(app.is_message_registered::<SplitItem>());
    assert!(app.is_message_registered::<DestroyItem>());
    assert!(app.is_message_registered::<UseItem>());
    assert!(app.is_message_registered::<EquipItem>());
    assert!(app.is_message_registered::<SortBags>());
}
