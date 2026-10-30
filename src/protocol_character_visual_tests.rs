use super::*;
use crate::components::{CharacterAppearance, CharacterVisual};
use bevy_replicon::shared::protocol::ProtocolHasher;
use bevy_replicon::shared::replication::registry::ReplicationRegistry;
use bevy_replicon::shared::replication::rules::ReplicationRules;
use lightyear::prelude::ComponentRegistry;

#[test]
fn character_visual_round_trips_and_is_registered_for_replication() {
    let visual = CharacterVisual {
        race: 10,
        class: 2,
        appearance: CharacterAppearance {
            sex: 1,
            skin_color: 4,
            hair_style: 6,
            ..Default::default()
        },
    };
    let decoded: CharacterVisual = bitcode::decode(&bitcode::encode(&visual)).unwrap();
    assert_eq!(decoded, visual);
    let mut app = App::new();
    app.init_resource::<ProtocolHasher>()
        .init_resource::<ReplicationRules>()
        .init_resource::<ReplicationRegistry>();
    app.add_plugins(ProtocolPlugin);
    assert!(
        app.world()
            .resource::<ComponentRegistry>()
            .is_registered::<CharacterVisual>()
    );
}
