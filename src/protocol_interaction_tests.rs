use super::*;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::AppMessageExt;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

fn assert_wire_round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(value, config).unwrap();
    let (decoded, read): (T, usize) = bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(&decoded, value);
}

#[test]
fn azerothcore_npcflag_values_map_to_retail_flags() {
    // creature_template.npcflag of Auctioneer Fitch (8719), Innkeeper Farley (295)
    // and the Spirit Healer (6491) in data/world.db.
    assert_eq!(
        NpcFlags::from_azerothcore(2_097_152),
        NpcFlags(NpcFlags::AUCTIONEER)
    );
    let farley = NpcFlags::from_azerothcore(66_179);
    assert_eq!(
        farley,
        NpcFlags(
            NpcFlags::GOSSIP
                | NpcFlags::QUESTGIVER
                | NpcFlags::VENDOR
                | NpcFlags::VENDOR_FOOD
                | NpcFlags::INNKEEPER
        )
    );
    assert!(farley.contains(NpcFlags::VENDOR));
    assert!(!farley.contains(NpcFlags::TRAINER));
    assert_eq!(
        NpcFlags::from_azerothcore(16_385),
        NpcFlags(NpcFlags::GOSSIP | NpcFlags::SPIRIT_HEALER)
    );
    // Bits AzerothCore does not define are dropped instead of gaining a Retail meaning.
    assert_eq!(
        NpcFlags::from_azerothcore(0x8000_0000 | 0x0400_0000),
        NpcFlags(NpcFlags::MAILBOX)
    );
}

#[test]
fn interaction_messages_round_trip() {
    assert_wire_round_trip(&InteractNpc { npc: 4_294_967_301 });
    assert_wire_round_trip(&SelectGossipOption {
        npc: 7,
        option_id: 3,
    });
    assert_wire_round_trip(&CloseInteraction { npc: 7 });
    assert_wire_round_trip(&InteractionOpened {
        npc: 7,
        kind: InteractionKind::Gossip(GossipMenu {
            menu_id: 1291,
            text: "Welcome to my Inn, weary traveler. What can I do for you?".into(),
            options: vec![GossipMenuOption {
                option_id: 3,
                icon: 1,
                text: "I want to browse your goods.".into(),
            }],
        }),
    });
    assert_wire_round_trip(&InteractionOpened {
        npc: 8,
        kind: InteractionKind::Role(NpcRole::AuctionHouse),
    });
    assert_wire_round_trip(&InteractionFailed {
        npc: 8,
        error: InteractionError::TooFarAway,
    });
    assert_wire_round_trip(&InteractionClosed { npc: 8 });
    assert_wire_round_trip(&NpcFlags(NpcFlags::AUCTIONEER));
}

#[test]
fn interaction_errors_use_retail_wording() {
    assert_eq!(
        InteractionError::TooFarAway.message(),
        "You are too far away."
    );
    assert_eq!(
        InteractionError::TargetHostile.message(),
        "Target is hostile"
    );
}

#[test]
fn protocol_plugin_registers_interaction_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<InteractNpc>());
    assert!(app.is_message_registered::<SelectGossipOption>());
    assert!(app.is_message_registered::<CloseInteraction>());
    assert!(app.is_message_registered::<InteractionOpened>());
    assert!(app.is_message_registered::<InteractionFailed>());
    assert!(app.is_message_registered::<InteractionClosed>());
}

#[test]
fn summon_messages_round_trip() {
    // Elwynn Forest (zone 12), CONFIRM_SUMMON's full two minutes.
    assert_wire_round_trip(&SummonRequest {
        summoner: "Stonecaller".into(),
        zone_id: 12,
        time_left_ms: 120_000,
    });
    assert_wire_round_trip(&SummonResponse { accept: true });
}

#[test]
fn protocol_plugin_registers_summon_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<SummonRequest>());
    assert!(app.is_message_registered::<SummonResponse>());
}
