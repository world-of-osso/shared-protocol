use super::*;
use crate::loot::LootMode;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::AppMessageExt;

#[test]
fn protocol_plugin_registers_group_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<RespondGroupInvite>());
    assert!(app.is_message_registered::<LeaveGroup>());
    assert!(app.is_message_registered::<PromoteGroupLeader>());
    assert!(app.is_message_registered::<ConvertGroupToRaid>());
    assert!(app.is_message_registered::<SetRaidSubgroup>());
    assert!(app.is_message_registered::<SetGroupRole>());
    assert!(app.is_message_registered::<SetGroupLootMethod>());
    assert!(app.is_message_registered::<StartReadyCheck>());
    assert!(app.is_message_registered::<RespondReadyCheck>());
    assert!(app.is_message_registered::<GroupInvitePrompt>());
    assert!(app.is_message_registered::<ReadyCheckUpdate>());
    assert!(app.is_message_registered::<GroupRosterSnapshot>());
    assert!(app.is_message_registered::<GroupCommandResponse>());
    assert!(app.is_message_registered::<ConvertGroupToParty>());
    assert!(app.is_message_registered::<GroupInviteCancelled>());
    assert!(app.is_message_registered::<GroupMemberStates>());
}

#[test]
fn group_member_states_round_trip_on_the_wire() {
    use crate::components::{AuraView, Position, PowerEntry, PowerType};
    let states = GroupMemberStates {
        members: vec![GroupMemberState {
            name: "Alice".into(),
            health: 812,
            max_health: 1200,
            power: Some(PowerEntry {
                power: PowerType::Rage,
                current: 35,
                max: 100,
                partial: 0,
                regen_per_sec: 0.0,
            }),
            death: crate::death::DeathState::Ghost,
            position: Position {
                x: -8913.2,
                y: -130.5,
                z: 82.1,
            },
            debuffs: vec![AuraView {
                instance_id: 7,
                spell_id: 589,
                caster: Some(99),
                stacks: 1,
                charges: 0,
                duration_ms: 18_000,
                remaining_ms: 12_500,
                harmful: true,
                dispel_type: 1,
                flags: 0,
            }],
        }],
    };
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&states, config).unwrap();
    let (decoded, _): (GroupMemberStates, usize) =
        bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(decoded, states);
}

#[test]
fn roster_portrait_round_trips_on_the_wire() {
    let roster = GroupRosterSnapshot {
        is_raid: true,
        ready_count: 1,
        total_count: 2,
        members: vec![GroupMemberSnapshot {
            character_id: 42,
            name: "Alice".into(),
            role: GroupRoleSnapshot::Tank,
            is_leader: true,
            online: true,
            subgroup: 3,
            class: 2,
            level: 80,
            entity: Some(42),
            portrait: GroupPortraitAppearance {
                race: 52,
                appearance: crate::components::CharacterAppearance {
                    sex: 1,
                    skin_color: 3,
                    face: 2,
                    eye_color: 5,
                    hair_style: 7,
                    hair_color: 8,
                    facial_style: 4,
                    customization_choices: vec![crate::components::CustomizationChoiceSelection {
                        option_id: 101,
                        choice_id: 202,
                    }],
                    visage: Some(crate::components::FormAppearance {
                        skin_color: 6,
                        face: 4,
                        eye_color: 3,
                        hair_style: 2,
                        hair_color: 1,
                        facial_style: 5,
                        customization_choices: vec![
                            crate::components::CustomizationChoiceSelection {
                                option_id: 303,
                                choice_id: 404,
                            },
                        ],
                    }),
                },
                head: Some(crate::components::EquippedAppearanceEntry {
                    slot: crate::components::EquipmentVisualSlot::Head,
                    item_id: Some(19019),
                    display_info_id: Some(12345),
                    inventory_type: 1,
                    hidden: true,
                }),
            },
        }],
        loot_method: LootMode::RoundRobin,
    };
    let config = bincode::config::standard();
    let mut roster = roster;
    for head in [roster.members[0].portrait.head.clone(), None] {
        roster.members[0].portrait.head = head;
        let bytes = bincode::serde::encode_to_vec(&roster, config).unwrap();
        let (decoded, consumed): (GroupRosterSnapshot, usize) =
            bincode::serde::decode_from_slice(&bytes, config).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded, roster);
    }
}

#[test]
fn group_message_codes_use_retail_global_strings() {
    assert_eq!(
        GroupMessageCode::GroupFull.global_string_key(),
        "ERR_GROUP_FULL"
    );
    assert_eq!(
        GroupMessageCode::GroupFull.format(""),
        "Your party is full."
    );
    assert_eq!(
        GroupMessageCode::DeclineGroup.format("Bob"),
        "Bob declines your group invitation."
    );
    assert_eq!(
        GroupMessageCode::BadPlayerName.format("Nobody"),
        "Cannot find player 'Nobody'."
    );
    assert_eq!(
        GroupMessageCode::RaidConvertedToParty.global_string_key(),
        "ERR_RAID_CONVERTED_TO_PARTY"
    );
    assert_eq!(
        GroupMessageCode::RaidConvertedToParty.format(""),
        "Raid converted to Party"
    );
}

#[test]
fn protocol_plugin_registers_raid_target_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    assert!(app.is_message_registered::<SetRaidTarget>());
    assert!(app.is_message_registered::<RaidTargetIcons>());
}

#[test]
fn raid_target_icons_name_the_icon_of_a_unit() {
    let mut icons = RaidTargetIcons::default();
    icons.targets[RAID_TARGET_SKULL as usize - 1] = Some(4_294_967_337);
    icons.targets[0] = Some(77);

    assert_eq!(icons.icon_of(4_294_967_337), Some(RAID_TARGET_SKULL));
    assert_eq!(icons.icon_of(77), Some(1));
    assert_eq!(icons.icon_of(78), None);
}
