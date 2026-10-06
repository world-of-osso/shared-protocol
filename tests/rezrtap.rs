use shared::components::UnitTap;
use shared::protocol::{ResurrectionOffer, ResurrectionResponse};

#[test]
fn rezrtap_offer_and_responses_roundtrip() {
    let offer = ResurrectionOffer {
        caster: 42,
        caster_name: "Alicia".into(),
        spell_id: 7328,
        time_left_ms: 60_000,
    };
    let bytes = bincode::serde::encode_to_vec(&offer, bincode::config::standard()).unwrap();
    let (decoded, _): (ResurrectionOffer, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(decoded, offer);
    for accept in [false, true] {
        let response = ResurrectionResponse {
            caster: 42,
            spell_id: 7328,
            accept,
        };
        let bytes = bincode::serde::encode_to_vec(&response, bincode::config::standard()).unwrap();
        let (decoded, _): (ResurrectionResponse, usize) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
        assert_eq!(decoded, response);
    }
}

#[test]
fn rezrtap_tap_roundtrip_and_viewer_group() {
    let tap = UnitTap(vec![42, 43]);
    assert!(!tap.denied(42, &[]));
    assert!(!tap.denied(99, &[43]));
    assert!(tap.denied(99, &[100]));
    assert!(!UnitTap::default().denied(99, &[]));
    assert_eq!(
        bitcode::decode::<UnitTap>(&bitcode::encode(&tap)).unwrap(),
        tap
    );
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&tap, config).unwrap();
    let (decoded, _): (UnitTap, usize) = bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(decoded, tap);
    assert_eq!(
        bytes.len(),
        3,
        "two one-byte character IDs and vector length"
    );
}
