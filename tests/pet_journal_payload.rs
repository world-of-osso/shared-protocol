use shared::protocol::{
    OwnedPetBattleSnapshot, PetJournalAbility, PetJournalDetailsUpdate, SetBattlePetAbility,
};

#[test]
fn owned_pet_details_preserve_injury_choices_and_full_instance_guid_on_wire() {
    let update = PetJournalDetailsUpdate {
        locked: false,
        error: None,
        pets: vec![OwnedPetBattleSnapshot {
            pet_id: (1_u64 << 42) + 17,
            health: 73,
            max_health: 1563,
            power: 276,
            speed: 244,
            next_level_xp: 420,
            selected_abilities: [384, 455, 459],
            choices: vec![PetJournalAbility {
                id: 384,
                name: "Metal Fist".into(),
                icon: 133015,
                family: 9,
                slot: 1,
                required_level: 1,
            }],
        }],
    };
    let bytes = bincode::serde::encode_to_vec(&update, bincode::config::standard()).unwrap();
    let (decoded, consumed): (PetJournalDetailsUpdate, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(consumed, bytes.len());
    assert_eq!(decoded, update);
    assert_eq!(decoded.pets[0].health, 73);
    assert_eq!(decoded.pets[0].selected_abilities, [384, 455, 459]);
}

#[test]
fn ability_selection_request_and_locked_refusal_round_trip() {
    let request = SetBattlePetAbility {
        pet_id: (1_u64 << 42) + 17,
        slot: 2,
        ability_id: 455,
    };
    let bytes = bincode::serde::encode_to_vec(&request, bincode::config::standard()).unwrap();
    let (decoded, _): (SetBattlePetAbility, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(decoded, request);
    let refusal = PetJournalDetailsUpdate {
        locked: true,
        pets: vec![],
        error: Some("Pet battle is active".into()),
    };
    let bytes = bincode::serde::encode_to_vec(&refusal, bincode::config::standard()).unwrap();
    let (decoded, _): (PetJournalDetailsUpdate, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(decoded, refusal);
}
