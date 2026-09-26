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
fn tooltip_messages_round_trip() {
    // Defias Thug (38): the Pitted Defias Shortsword (2057) teaches an appearance,
    // the Red Burlap Bandana (752) quest item does not.
    assert_wire_round_trip(&CreatureTooltipQuery { entry: 38 });
    assert_wire_round_trip(&CreatureTooltip {
        entry: 38,
        subname: String::new(),
        creature_type: 7,
        drops: vec![
            TooltipDrop {
                item: TooltipItem {
                    item_id: 752,
                    name: "Red Burlap Bandana".into(),
                    quality: 1,
                    appearance_id: None,
                },
                chance: 60.0,
            },
            TooltipDrop {
                item: TooltipItem {
                    item_id: 2057,
                    name: "Pitted Defias Shortsword".into(),
                    quality: 1,
                    appearance_id: Some(1_234),
                },
                chance: 2.0,
            },
        ],
        vendor_items: Vec::new(),
    });
    assert_wire_round_trip(&AppearanceCollectionUpdate {
        appearances: vec![1_234, 5_678],
    });
}

#[test]
fn protocol_plugin_registers_tooltip_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<CreatureTooltipQuery>());
    assert!(app.is_message_registered::<CreatureTooltip>());
    assert!(app.is_message_registered::<AppearanceCollectionUpdate>());
}
