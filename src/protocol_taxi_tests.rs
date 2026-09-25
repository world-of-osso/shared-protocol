use super::*;
use crate::components::MovementControl;
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
fn taxi_messages_round_trip() {
    // Stormwind (2) to Sentinel Hill (4) is TaxiPath 6, 5 copper.
    assert_wire_round_trip(&TaxiMap {
        npc: 4_294_967_301,
        continent: 0,
        nodes: vec![
            TaxiNodeInfo {
                node: 2,
                name: "Stormwind, Elwynn".into(),
                world_x: -8841.06,
                world_y: 489.66,
                state: TaxiNodeState::Current,
                cost: 0,
                route: vec![],
            },
            TaxiNodeInfo {
                node: 4,
                name: "Sentinel Hill, Westfall".into(),
                world_x: -10551.9,
                world_y: 1034.39,
                state: TaxiNodeState::Reachable,
                cost: 5,
                route: vec![2, 4],
            },
        ],
    });
    assert_wire_round_trip(&ActivateTaxi {
        npc: 7,
        destination: 4,
    });
    assert_wire_round_trip(&TaxiNodeDiscovered { node: 2 });
    assert_wire_round_trip(&TaxiFailed {
        npc: 7,
        error: TaxiError::NotEnoughMoney,
    });
    assert_wire_round_trip(&MovementControl {
        epoch: 3,
        controlled: true,
    });
}

#[test]
fn taxi_errors_use_retail_wording() {
    assert_eq!(
        TaxiError::NotEnoughMoney.message(),
        "You don't have enough money!"
    );
    assert_eq!(
        TaxiError::NotVisited.message(),
        "You haven't reached that flight location on foot yet!"
    );
}

#[test]
fn protocol_plugin_registers_taxi_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<TaxiMap>());
    assert!(app.is_message_registered::<TaxiNodeDiscovered>());
    assert!(app.is_message_registered::<TaxiFailed>());
    assert!(app.is_message_registered::<ActivateTaxi>());
}
