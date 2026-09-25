use super::*;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::AppMessageExt;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::profession::ProfessionSkillLine;

fn assert_wire_round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(value, config).unwrap();
    let (decoded, read): (T, usize) = bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(&decoded, value);
}

#[test]
fn trainer_and_profession_messages_round_trip() {
    // Georgio Bolero (1346), TrinityCore trainer 163: Tailoring (264617) for 10 copper
    // at level 5, White Linen Shirt (2393) at Classic Tailoring (2540) rank 1.
    assert_wire_round_trip(&TrainerList {
        npc: 4_294_967_301,
        trainer_id: 163,
        greeting: "Greetings!".into(),
        services: vec![
            TrainerService {
                spell_id: 264617,
                cost: 10,
                state: TrainerServiceState::Available,
                req_level: 5,
                req_skill_line: 0,
                req_skill_rank: 0,
                req_abilities: vec![],
                profession: true,
            },
            TrainerService {
                spell_id: 2393,
                cost: 10,
                state: TrainerServiceState::Unavailable,
                req_level: 0,
                req_skill_line: 2540,
                req_skill_rank: 1,
                req_abilities: vec![],
                profession: false,
            },
        ],
    });
    assert_wire_round_trip(&TrainerBuySpell {
        npc: 7,
        spell_id: 264617,
    });
    assert_wire_round_trip(&TrainerBuyFailed {
        npc: 7,
        spell_id: 264617,
        reason: TrainerFailReason::NotEnoughMoney,
    });
    assert_wire_round_trip(&ProfessionSnapshot {
        lines: vec![
            ProfessionSkillLine {
                skill_line: 197,
                step: 4,
                rank: 1,
                max_rank: 75,
            },
            ProfessionSkillLine {
                skill_line: 2540,
                step: 1,
                rank: 1,
                max_rank: 300,
            },
        ],
        spells: vec![3908, 264616, 2963],
    });
    assert_wire_round_trip(&CraftRecipe {
        spell_id: 2963,
        casts: 3,
    });
}

#[test]
fn not_enough_money_uses_retail_wording() {
    assert_eq!(
        TrainerFailReason::NotEnoughMoney.message(),
        Some("You don't have enough money.")
    );
    assert_eq!(TrainerFailReason::Unavailable.message(), None);
}

#[test]
fn protocol_plugin_registers_trainer_and_profession_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<TrainerList>());
    assert!(app.is_message_registered::<TrainerBuySpell>());
    assert!(app.is_message_registered::<TrainerBuyFailed>());
    assert!(app.is_message_registered::<ProfessionSnapshot>());
    assert!(app.is_message_registered::<CraftRecipe>());
}
