use serde_json::json;
use shared::components::{CharacterAppearance, CustomizationChoiceSelection, FormAppearance};
use shared::protocol::{CreateCharacter, LoginResponse};

fn appearance_json() -> serde_json::Value {
    json!({
        "sex": 1, "skin_color": 2, "face": 3, "eye_color": 4,
        "hair_style": 5, "hair_color": 6, "facial_style": 7,
        "customization_choices": [
            {"option_id": 87, "choice_id": 1734},
            {"option_id": 214, "choice_id": 6109}
        ],
        "visage": null
    })
}

#[test]
fn customization_choices_survive_appearance_bitcode_roundtrip() {
    let expected = appearance_json();
    let appearance: CharacterAppearance = serde_json::from_value(expected.clone()).unwrap();
    let bytes = bitcode::encode(&appearance);
    let decoded: CharacterAppearance = bitcode::decode(&bytes).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), expected);
}

#[test]
fn customization_choices_preserve_u32_ids_and_more_than_byte_sized_counts() {
    let appearance = CharacterAppearance {
        customization_choices: (0..300)
            .map(|index| CustomizationChoiceSelection {
                option_id: 70_000 + index,
                choice_id: 4_000_000 + index,
            })
            .collect(),
        ..Default::default()
    };
    let decoded: CharacterAppearance = bitcode::decode(&bitcode::encode(&appearance)).unwrap();
    assert_eq!(decoded, appearance);
}

#[test]
fn customization_choices_survive_create_and_roster_wire_roundtrips() {
    let expected = appearance_json();
    let request: CreateCharacter = serde_json::from_value(json!({
        "name": "Elara", "race": 1, "class": 8, "appearance": expected
    }))
    .unwrap();
    let bytes = bitcode::serialize(&request).unwrap();
    let decoded: CreateCharacter = bitcode::deserialize(&bytes).unwrap();
    assert_eq!(serde_json::to_value(&decoded.appearance).unwrap(), expected);

    let response: LoginResponse = serde_json::from_value(json!({
        "success": true, "token": "test-session", "error": null,
        "characters": [{
            "character_id": 41, "name": decoded.name, "race": decoded.race,
            "class": decoded.class, "level": 1, "appearance": decoded.appearance,
            "equipment_appearance": {"entries": []}
        }]
    }))
    .unwrap();
    let bytes = bitcode::serialize(&response).unwrap();
    let decoded: LoginResponse = bitcode::deserialize(&bytes).unwrap();
    assert_eq!(decoded.characters.len(), 1);
    assert_eq!(
        serde_json::to_value(&decoded.characters[0].appearance).unwrap(),
        expected
    );
}

/// A Dracthyr's visage form travels with its dragon form in `CreateCharacter`.
#[test]
fn visage_form_survives_create_character_wire_roundtrip() {
    let appearance = CharacterAppearance {
        sex: 1,
        skin_color: 3,
        visage: Some(FormAppearance {
            skin_color: 2,
            face: 4,
            eye_color: 1,
            hair_style: 7,
            hair_color: 5,
            facial_style: 0,
            customization_choices: vec![CustomizationChoiceSelection {
                option_id: 2064,
                choice_id: 30012,
            }],
        }),
        ..Default::default()
    };
    let request = CreateCharacter {
        name: "Scalesong".into(),
        race: 52,
        class: 13,
        appearance: appearance.clone(),
    };
    let decoded: CreateCharacter =
        bitcode::deserialize(&bitcode::serialize(&request).unwrap()).unwrap();
    assert_eq!(decoded.appearance, appearance);
    let decoded: CharacterAppearance = bitcode::decode(&bitcode::encode(&appearance)).unwrap();
    assert_eq!(decoded, appearance);
    // JSON written before the field existed still reads, with no visage.
    let old: CharacterAppearance = serde_json::from_value(json!({
        "sex": 0, "skin_color": 0, "face": 0, "eye_color": 0, "hair_style": 0,
        "hair_color": 0, "facial_style": 0, "customization_choices": []
    }))
    .unwrap();
    assert_eq!(old.visage, None);
}
