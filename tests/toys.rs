use serde::{Serialize, de::DeserializeOwned};
use shared::protocol::*;

fn roundtrip<T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug>(value: T) {
    let bytes = bincode::serde::encode_to_vec(&value, bincode::config::standard()).unwrap();
    let (decoded, consumed): (T, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(decoded, value);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn toys_wire_roundtrips_real_item_and_account_flags() {
    roundtrip(UseToy {
        item_id: 1973,
        destination: None,
        witness: None,
    });
    roundtrip(UseToy {
        item_id: 89869,
        destination: Some([-8914.57, 80.81, 131.61]),
        witness: None,
    });
    roundtrip(UseToy {
        item_id: 122120,
        destination: None,
        witness: Some(WitnessRay {
            start: [-8908.2, 83.7, 146.6],
            end: [-8896.5, 82.1, 173.8],
        }),
    });
    roundtrip(SetToyFavourite {
        item_id: 1973,
        favourite: true,
    });
    roundtrip(ToyCollectionUpdate {
        toys: vec![ToySnapshot {
            item_id: 1973,
            name: "Orb of Deception".into(),
            icon_file_data_id: 134334,
            expansion_id: 0,
            flags: 0,
            source_type: 0,
            source_text: "World Drop".into(),
            spell_id: Some(16739),
            requires_target_witness: false,
            learned: true,
            favourite: true,
            unavailable_reason: Some("unsupported aura 4".into()),
        }],
    });
    for operation in [
        ToyOperation::Learn,
        ToyOperation::Favourite,
        ToyOperation::Use,
    ] {
        for error in [None, Some("not learned".into())] {
            roundtrip(ToyResult {
                item_id: 1973,
                operation,
                spell_id: Some(16739),
                error,
            });
        }
    }
}
