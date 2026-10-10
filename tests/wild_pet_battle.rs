use shared::pet_battle::{PetJournal, PetQuality};
use shared::protocol::*;

fn round_trip<T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug>(
    value: T,
) {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&value, config).unwrap();
    let (decoded, read): (T, usize) = bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(value, decoded);
}

#[test]
fn wild_pet_battle_wire_preserves_round_actions_and_end_rewards() {
    for action in [
        WildPetBattleAction::Ability(1),
        WildPetBattleAction::Ability(3),
        WildPetBattleAction::Swap(2),
        WildPetBattleAction::Pass,
        WildPetBattleAction::Forfeit,
        WildPetBattleAction::Trap,
    ] {
        round_trip(WildPetBattleActionRequest {
            battle_id: 0x100000045,
            round: 9,
            action,
        });
    }
    round_trip(StartWildPetBattle {
        creature: 0x100000077,
    });
    round_trip(SetBattlePetLoadout {
        slots: [Some(0x100000041), None, Some(0x100000043)],
    });
    let state = WildPetBattleSnapshot {
        battle_id: 0x100000045,
        round: 9,
        active: [0, 0],
        wild_creature: 0x100000077,
        teams: [
            vec![BattlePetSnapshot {
                instance_id: Some(0x100000041),
                species_id: 39,
                name: "Mechanical Squirrel".into(),
                display_id: 7937,
                family: 9,
                level: 7,
                health: 110,
                max_health: 480,
                power: 52,
                speed: 61,
                abilities: std::array::from_fn(|_| BattleAbilitySnapshot {
                    id: 110,
                    name: "Bite".into(),
                    icon: 132139,
                    cooldown: 2,
                    usable: false,
                }),
                auras: vec![BattleAuraSnapshot {
                    ability_id: 194,
                    rounds_remaining: 2,
                }],
            }],
            vec![],
        ],
        can_trap: false,
        turn_time_ms: 0,
        replacement_required: false,
    };
    round_trip(WildPetBattleUpdate::Start(state.clone()));
    round_trip(WildPetBattleUpdate::State(state.clone()));
    round_trip(WildPetBattleUpdate::Round {
        state,
        combat_text: vec!["Bite dealt 52 damage".into()],
    });
    round_trip(WildPetBattleUpdate::End {
        battle_id: 0x100000045,
        outcome: WildPetBattleOutcome::Won,
        rewards: vec![BattlePetXpReward {
            instance_id: 0x100000041,
            xp_gained: 50,
            level: 8,
            xp: 12,
        }],
        captured_pet_id: Some(0x100000099),
        combat_text: vec!["Captured Rabbit".into()],
    });
}

#[test]
fn wild_pet_battle_loadout_is_owned_distinct_and_clears_released_pet() {
    let mut journal = PetJournal::default();
    let first = journal
        .add_with_guid(39, 1, PetQuality::Common, 3, 0x100000041)
        .unwrap();
    let second = journal
        .add_with_guid(40, 1, PetQuality::Common, 3, 0x100000042)
        .unwrap();
    let third = journal
        .add_with_guid(41, 1, PetQuality::Common, 3, 0x100000043)
        .unwrap();
    assert_eq!(
        journal.battle_slots,
        [Some(first), Some(second), Some(third)]
    );
    assert!(
        journal
            .set_battle_slots([Some(first), Some(first), None])
            .is_err()
    );
    assert!(journal.set_battle_slots([Some(900), None, None]).is_err());
    journal
        .set_battle_slots([Some(third), Some(second), Some(first)])
        .unwrap();
    journal.remove(second).unwrap();
    assert_eq!(journal.battle_slots, [Some(third), None, Some(first)]);
    let bytes = serde_json::to_vec(&journal).unwrap();
    assert_eq!(
        serde_json::from_slice::<PetJournal>(&bytes).unwrap(),
        journal
    );
}
