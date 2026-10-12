use super::{
    DecorTransform, HousingCommand, HousingReply, HousingRequest, HousingState, StoredDecor,
};

#[test]
fn housing_wire_preserves_real_decor_identity_quantities_and_requested_transform() {
    let request = HousingRequest {
        request_id: 7,
        command: HousingCommand::PlaceDecor {
            house_id: 42,
            decor_id: 1693,
            transform: DecorTransform {
                position: [1.25, 2.0, -3.0],
                yaw: std::f32::consts::FRAC_PI_2,
            },
        },
    };
    let encoded = bincode::serde::encode_to_vec(&request, bincode::config::standard()).unwrap();
    let (decoded, consumed): (HousingRequest, _) =
        bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
    assert_eq!(decoded, request);
    assert_eq!(consumed, encoded.len());
    let reply = HousingReply {
        request_id: 7,
        state: HousingState {
            build_key: "dcfc90fffd79ba00406ae46f5f657592".into(),
            opened_cornerstone: Some(417),
            stored: vec![StoredDecor {
                decor_id: 1693,
                quantity: 1,
            }],
            ..HousingState::default()
        },
        error: Some("Only the owner may edit this house.".into()),
    };
    let encoded = bincode::serde::encode_to_vec(&reply, bincode::config::standard()).unwrap();
    let (decoded, consumed): (HousingReply, _) =
        bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
    assert_eq!(decoded, reply);
    assert_eq!(consumed, encoded.len());
}
