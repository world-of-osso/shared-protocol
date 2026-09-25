use bevy::prelude::*;
use lightyear::prelude::AppComponentExt;

pub use crate::protocol_snapshots::*;

use crate::components::{
    CombatStatus, EquipmentAppearance, Gold, GuildMembership, Health, Mana, ModelDisplay, Mounted,
    MovementSpeed, Npc, Player, Position, PresenceStatus, Rotation, UnitAuras, UnitFactionTemplate,
    UnitLevel, UnitPowers, UnitTarget, Zone,
};

mod channels;
mod core_messages;
mod experience_messages;
mod gameplay_messages;
mod group_messages;
mod interaction_messages;
mod inventory_messages;
mod merchant_messages;
mod quest_messages;
mod registration;
mod spell_messages;

pub use channels::*;
pub use core_messages::*;
pub use experience_messages::*;
pub use gameplay_messages::*;
pub use group_messages::*;
pub use interaction_messages::*;
pub use inventory_messages::*;
pub use merchant_messages::*;
pub use quest_messages::*;
pub use spell_messages::*;

use registration::{register_channels, register_messages};

/// Registers shared protocol: components for replication and channels.
/// Must be added AFTER `ServerPlugins`/`ClientPlugins` but BEFORE any entity is spawned.
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        register_replicated_components(app);
        register_messages(app);
        register_channels(app);
        inventory_messages::register_inventory_protocol(app);
        quest_messages::register_quest_protocol(app);
        interaction_messages::register_interaction_protocol(app);
        experience_messages::register_experience_protocol(app);
        merchant_messages::register_merchant_protocol(app);
    }
}

fn register_replicated_components(app: &mut App) {
    app.component::<Position>().replicate();
    app.component::<Health>().replicate();
    app.component::<Mana>().replicate();
    app.component::<Gold>().replicate();
    app.component::<Player>().replicate();
    app.component::<Npc>().replicate();
    app.component::<ModelDisplay>().replicate();
    app.component::<Rotation>().replicate();
    app.component::<MovementSpeed>().replicate();
    app.component::<CombatStatus>().replicate();
    app.component::<Mounted>().replicate();
    app.component::<Zone>().replicate();
    app.component::<GuildMembership>().replicate();
    app.component::<PresenceStatus>().replicate();
    app.component::<EquipmentAppearance>().replicate();
    app.component::<crate::casting::CastState>().replicate();
    app.component::<UnitPowers>().replicate();
    app.component::<UnitAuras>().replicate();
    app.component::<UnitLevel>().replicate();
    app.component::<UnitFactionTemplate>().replicate();
    app.component::<UnitTarget>().replicate();
}

#[cfg(test)]
#[path = "protocol_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "protocol_spell_tests.rs"]
mod spell_tests;

#[cfg(test)]
#[path = "protocol_group_tests.rs"]
mod group_tests;

#[cfg(test)]
#[path = "protocol_inventory_tests.rs"]
mod inventory_tests;

#[cfg(test)]
#[path = "protocol_quest_tests.rs"]
mod quest_tests;

#[cfg(test)]
#[path = "protocol_interaction_tests.rs"]
mod interaction_tests;

#[cfg(test)]
#[path = "protocol_merchant_tests.rs"]
mod merchant_tests;

#[cfg(test)]
#[path = "protocol_experience_tests.rs"]
mod experience_tests;
