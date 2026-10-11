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
    assert!(app.is_message_registered::<ProfessionSpecializationRequest>());
    assert!(app.is_message_registered::<ProfessionSpecializationStateUpdate>());
    assert!(app.is_message_registered::<CraftingOrderRequest>());
    assert!(app.is_message_registered::<CraftingOrderStateUpdate>());
}

#[test]
fn fixed_output_order_and_specialization_requests_round_trip() {
    assert_wire_round_trip(&ProfessionSpecializationRequest::Commit {
        skill_line: 2830,
        tree_id: 350,
        purchases: vec![ProfessionTraitPurchase {
            node_id: 28536,
            entry_id: 36321,
            ranks: 5,
        }],
    });
    for kind in [
        CraftingOrderKind::Public,
        CraftingOrderKind::Guild,
        CraftingOrderKind::Personal,
    ] {
        assert_wire_round_trip(&CraftingOrderRequest::Place {
            spell_id: 413496,
            kind,
            recipient: Some("Tailor".into()),
            commission: 12_345,
            reagents: vec![CraftingOrderReagent {
                item_id: 8170,
                quantity: 4,
                supplied: 4,
            }],
            duration_hours: 24,
            note: "For my guild".into(),
        });
    }
    for request in [
        CraftingOrderRequest::Query {
            skill_line: 2532,
            kind: CraftingOrderKind::Guild,
            mine: false,
        },
        CraftingOrderRequest::Claim {
            order_id: 4_294_967_301,
        },
        CraftingOrderRequest::Release {
            order_id: 4_294_967_301,
        },
        CraftingOrderRequest::Fulfill {
            order_id: 4_294_967_301,
        },
        CraftingOrderRequest::Cancel {
            order_id: 4_294_967_301,
        },
    ] {
        assert_wire_round_trip(&request);
    }
    assert_wire_round_trip(&CraftingOrderStateUpdate {
        snapshot: Some(CraftingOrdersSnapshot {
            skill_line: 2532,
            kind: CraftingOrderKind::Personal,
            mine: false,
            orders: vec![CraftingOrderSnapshot {
                id: 4_294_967_301,
                spell_id: 413496,
                item_id: 206434,
                count: 1,
                kind: CraftingOrderKind::Personal,
                customer: "Customer".into(),
                recipient: Some("Tailor".into()),
                crafter: Some("Tailor".into()),
                commission: 12_345,
                reagents: vec![],
                expires_at: 1_791_763_200,
                note: "For my guild".into(),
                status: CraftingOrderStatus::Claimed,
                can_claim: false,
                can_fulfill: true,
                can_cancel: false,
            }],
        }),
        message: Some("Claimed crafting order".into()),
        error: None,
    });
    assert_wire_round_trip(&ProfessionSpecializationStateUpdate {
        snapshot: Some(ProfessionSpecializationsSnapshot {
            skill_line: 2830,
            knowledge_currency_id: 2025,
            knowledge_available: 25,
            trees: vec![ProfessionSpecTreeSnapshot {
                tree_id: 350,
                name: "Leather Armor Crafting".into(),
                root_node_id: 28532,
                unlocked: true,
                nodes: vec![ProfessionSpecNodeSnapshot {
                    node_id: 28536,
                    entry_id: 36321,
                    name: "Leather Armor Crafting".into(),
                    description: "Learn an additional sub-specialization".into(),
                    icon: 134262,
                    spell_id: 0,
                    rank: 5,
                    granted_rank: 0,
                    max_rank: 30,
                    pos_x: -100,
                    pos_y: 200,
                    kind: 1,
                    available: true,
                    costs: vec![ProfessionSpecCost {
                        trait_currency_id: 9,
                        amount: 1,
                    }],
                    path: Some(ProfessionSpecPath {
                        left_group: 100,
                        right_group: 101,
                        trait_currency_id: 1451,
                    }),
                    perk: Some(ProfessionSpecPerk {
                        required_points: 5,
                        unlocks_subpath: true,
                    }),
                }],
            }],
        }),
        message: Some("Applied specialization".into()),
        error: None,
    });
    assert_wire_round_trip(&CraftingOrderStateUpdate {
        snapshot: None,
        message: None,
        error: Some("Recipe does not accept crafting orders".into()),
    });
    assert_wire_round_trip(&ProfessionSpecializationStateUpdate {
        snapshot: None,
        message: None,
        error: Some("Not enough knowledge".into()),
    });
}
