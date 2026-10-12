use super::{
    DecorTransform, HousingCommand, HousingReply, HousingRequest, HousingState,
    NeighborhoodResident, NeighborhoodRoster, StoredDecor,
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
                pitch: 0.25,
                roll: -0.5,
                scale: 1.25,
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
            bulletin_board: Some(NeighborhoodRoster {
                neighborhood_id: 1,
                residents: vec![NeighborhoodResident {
                    name: "Houseproof".into(),
                    online: true,
                    plot_index: 0,
                }],
            }),
            stored: vec![StoredDecor {
                decor_id: 1693,
                quantity: 1,
            }],
            placed: vec![StoredDecor {
                decor_id: 2330,
                quantity: 2,
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

#[test]
fn housing_plot_owner_relationship_and_source_plot_visit_roundtrip() {
    use super::{HousingPlot, HousingPlotOwnerType};
    let plot = HousingPlot {
        plot_id: 417,
        neighborhood_id: 1,
        plot_index: 0,
        name: String::new(),
        cornerstone: [2942.744, -214.26563, 113.11285],
        teleport: [2939.3438, -212.2554, 113.321],
        cost: 10_000_000,
        house_id: Some(1),
        owner_character_id: Some(32),
        owner_type: HousingPlotOwnerType::Friend,
    };
    let encoded = bincode::serde::encode_to_vec(&plot, bincode::config::standard()).unwrap();
    let (decoded, consumed): (HousingPlot, usize) =
        bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
    assert_eq!(decoded, plot);
    assert_eq!(consumed, encoded.len());
    let request = HousingRequest {
        request_id: 9,
        command: HousingCommand::VisitPlot { plot_id: 417 },
    };
    let bytes = bincode::serde::encode_to_vec(&request, bincode::config::standard()).unwrap();
    let (decoded, _): (HousingRequest, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(decoded, request);
}
