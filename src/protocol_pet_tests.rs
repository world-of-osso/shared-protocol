use super::*;
use crate::components::UnitSummonedBy;
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::{AppMessageExt, ComponentRegistry};

fn assert_wire_round_trip<T>(value: &T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(value, config).unwrap();
    let (decoded, read): (T, usize) = bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(&decoded, value);
}

#[test]
fn packed_action_buttons_are_trinitycore_unit_action_buttons() {
    // CharmInfo::InitPetActionBar: slot 0 is MAKE_UNIT_ACTION_BUTTON(COMMAND_ATTACK, ACT_COMMAND).
    assert_eq!(pet_action_button(COMMAND_ATTACK, ACT_COMMAND), 0x0700_0002);
    assert_eq!(pet_action_button(REACT_ASSIST, ACT_REACTION), 0x0600_0003);
    // Bite 17253, autocast on: ACT_ENABLED.
    let bite = pet_action_button(17_253, ACT_ENABLED);
    assert_eq!(bite, 0xC100_4365);
    assert_eq!(pet_action_button_action(bite), 17_253);
    assert_eq!(pet_action_button_type(bite), ACT_ENABLED);
}

#[test]
fn pet_messages_round_trip_on_the_wire() {
    let mut action_buttons = [0; PET_ACTION_BAR_SLOTS];
    action_buttons[0] = pet_action_button(COMMAND_ATTACK, ACT_COMMAND);
    action_buttons[3] = pet_action_button(61_684, ACT_ENABLED);
    action_buttons[7] = pet_action_button(REACT_ASSIST, ACT_REACTION);
    assert_wire_round_trip(&PetSpells {
        pet: 0x0000_0001_0000_0042,
        command_state: COMMAND_FOLLOW,
        react_state: REACT_ASSIST,
        action_buttons,
    });
    assert_wire_round_trip(&PetClearSpells);
    assert_wire_round_trip(&PetAction {
        pet: 0x0000_0001_0000_0042,
        action: pet_action_button(COMMAND_ATTACK, ACT_COMMAND),
        target: Some(0x0000_0001_0000_0077),
        position: None,
    });
    assert_wire_round_trip(&PetAction {
        pet: 0x0000_0001_0000_0042,
        action: pet_action_button(COMMAND_MOVE_TO, ACT_COMMAND),
        target: None,
        position: Some([-8_913.5, 82.25, -554.0]),
    });
    assert_wire_round_trip(&PetSpellAutocast {
        pet: 0x0000_0001_0000_0042,
        spell: 17_253,
        enabled: false,
    });
    assert_wire_round_trip(&UnitSummonedBy(0x0000_0001_0000_0010));
}

#[test]
fn protocol_plugin_registers_pet_component_and_messages() {
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);

    let components = app.world().resource::<ComponentRegistry>();
    assert!(components.is_registered::<UnitSummonedBy>());
    assert!(app.is_message_registered::<PetSpells>());
    assert!(app.is_message_registered::<PetClearSpells>());
    assert!(app.is_message_registered::<PetAction>());
    assert!(app.is_message_registered::<PetSpellAutocast>());
}
