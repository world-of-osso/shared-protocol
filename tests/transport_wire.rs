use shared::components::{TransportPassenger, TransportState};
use shared::protocol::PlayerInput;

#[test]
fn transport_route_and_local_passenger_frame_survive_the_replication_wire() {
    let state = TransportState {
        spawn_id: 35,
        path_id: 6519,
        interior_map: 1856,
        knots: vec![
            [1261.6598, 2588.9307, 297.08664],
            [1193.9063, 2677.0313, 297.08664],
            [1179.8091, 2785.4949, 297.08862],
            [1235.5313, 2874.0051, 297.08862],
        ],
        speed: 4.0,
        elapsed_ms: 30_000,
    };
    let decoded: TransportState = bitcode::decode(&bitcode::encode(&state)).unwrap();
    assert_eq!(decoded, state);
    let passenger = TransportPassenger {
        transport: 227900,
        position: [-29.8623, 0.520232, 20.5534],
        orientation: 0.0,
    };
    let decoded: TransportPassenger = bitcode::decode(&bitcode::encode(&passenger)).unwrap();
    assert_eq!(decoded, passenger);
}

#[test]
fn transport_movement_retains_the_local_frame_and_disembark_uses_world_coordinates() {
    let input = PlayerInput {
        direction: [1.0, 0.0, 0.0],
        facing_yaw: 1.25,
        jumping: false,
        running: true,
        swimming: false,
        flying: false,
        position: [1200.0, 317.64004, -2677.0],
        epoch: 9,
        transport: Some(TransportPassenger {
            transport: 227900,
            position: [-28.8623, 0.520232, 20.5534],
            orientation: 0.25,
        }),
    };
    for transport in [input.transport, None] {
        let sent = PlayerInput {
            transport,
            ..input.clone()
        };
        let encoded = bincode::serde::encode_to_vec(&sent, bincode::config::standard()).unwrap();
        let (received, used): (PlayerInput, usize) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(used, encoded.len());
        assert_eq!(received.transport, transport);
        assert_eq!(received.position, sent.position);
        assert_eq!(received.epoch, 9);
    }
}
