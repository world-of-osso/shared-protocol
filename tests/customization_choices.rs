use serde_json::json;
use shared::components::CharacterAppearance;
use shared::protocol::{CreateCharacter, LoginResponse};

fn appearance_json() -> serde_json::Value {
    json!({
        "sex": 1, "skin_color": 2, "face": 3, "eye_color": 4,
        "hair_style": 5, "hair_color": 6, "facial_style": 7,
        "customization_choices": [
            {"option_id": 87, "choice_id": 1734},
            {"option_id": 214, "choice_id": 6109}
        ]
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
