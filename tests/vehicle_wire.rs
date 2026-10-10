use shared::components::{Mounted, VehiclePassenger};
use shared::protocol::{BoardVehicle, ExitVehicle};

#[test]
fn vehicle_wire_preserves_sparse_seats_and_server_entity_bits() {
    let driver = 0x1234_5678_9abc_def0;
    let passenger = 0x4321_8765_cba9_0fed;
    let mut mount = Mounted {
        mount_display_id: 27237,
        vehicle_id: 312,
        ..Default::default()
    };
    mount.seats[2] = Some(passenger);
    let decoded: Mounted = bitcode::decode(&bitcode::encode(&mount)).unwrap();
    assert_eq!(decoded, mount);
    assert_eq!(decoded.seats[0], None);
    assert_eq!(decoded.seats[2], Some(passenger));
    let seat = VehiclePassenger {
        driver,
        seat_index: 2,
        seat_id: 2765,
    };
    assert_eq!(
        bitcode::decode::<VehiclePassenger>(&bitcode::encode(&seat)).unwrap(),
        seat
    );
    let request = BoardVehicle { driver };
    assert_eq!(
        serde_json::from_str::<BoardVehicle>(&serde_json::to_string(&request).unwrap()).unwrap(),
        request
    );
    assert_eq!(
        serde_json::from_str::<ExitVehicle>(&serde_json::to_string(&ExitVehicle).unwrap()).unwrap(),
        ExitVehicle
    );
}
