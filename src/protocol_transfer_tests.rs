use super::*;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::{AppMessageExt, ComponentRegistry};
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
fn transfer_messages_round_trip() {
    // areatrigger_teleport 101: Stormwind Stockade entrance, world_safe_locs 3599
    // (56.6821, 0.62376, -19.2691) in Bevy space.
    assert_wire_round_trip(&NewWorld {
        map_id: 34,
        map_directory: "stormwindjail".into(),
        position: [56.6821, -19.2691, -0.62376],
        facing: 6.2715,
    });
    assert_wire_round_trip(&WorldPortAck);
    assert_wire_round_trip(&TransferAborted {
        map_id: 34,
        reason: TransferAbortReason::MaxPlayers,
    });
    assert_wire_round_trip(&TransferAborted {
        map_id: 34,
        reason: TransferAbortReason::Condition(
            "You must be at least level 10 to enter Stormwind Stockade.".into(),
        ),
    });
}

#[test]
fn transfer_abort_reasons_carry_retail_global_strings() {
    assert_eq!(
        TransferAbortReason::MaxPlayers.text("The Stockade"),
        "Transfer Aborted: instance is full"
    );
    assert_eq!(
        TransferAbortReason::MapNotAllowed.text("The Stockade"),
        "Map cannot be entered at this time."
    );
    assert_eq!(
        TransferAbortReason::Difficulty(2).text("Grim Batol"),
        "Heroic difficulty mode is not available for Grim Batol."
    );
    assert_eq!(
        TransferAbortReason::Difficulty(23).text("Grim Batol"),
        "This difficulty mode is not available for Grim Batol."
    );
    assert_eq!(
        TransferAbortReason::LockedToDifferentInstance.text("Grim Batol"),
        "You are already locked to Grim Batol."
    );
    // PlayerCondition 74354, the Stockade's MapDifficultyXCondition, as the server resolved it.
    assert_eq!(
        TransferAbortReason::Condition(
            "You must be at least level 10 to enter Stormwind Stockade.".into()
        )
        .text("Stormwind Stockade"),
        "You must be at least level 10 to enter Stormwind Stockade."
    );
}

#[test]
fn protocol_plugin_registers_transfer_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<NewWorld>());
    assert!(app.is_message_registered::<WorldPortAck>());
    assert!(app.is_message_registered::<TransferAborted>());
    let components = app.world().resource::<ComponentRegistry>();
    assert!(components.is_registered::<crate::components::WorldArrival>());
}

#[test]
fn a_player_counts_no_map_arrival_until_the_server_bumps_it() {
    let mut world = World::new();
    let player = world.spawn(test_player()).id();
    assert_eq!(
        world.get::<crate::components::WorldArrival>(player),
        Some(&crate::components::WorldArrival(0))
    );
    assert_wire_round_trip(&crate::components::WorldArrival(3));
}

fn test_player() -> crate::components::Player {
    crate::components::Player {
        name: "Osso".into(),
        race: 1,
        class: 2,
        appearance: Default::default(),
    }
}
