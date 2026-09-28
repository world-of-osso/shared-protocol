use bevy::prelude::*;
use lightyear::prelude::AppComponentExt;

pub use crate::protocol_snapshots::*;

use crate::components::{
    CombatStatus, CreatureMotion, EquipmentAppearance, Gold, GuildMembership, Health, Mana,
    ModelDisplay, Mounted, MovementControl, MovementSpeed, Npc, Player, Position, PresenceStatus,
    Rotation, UnitAuras, UnitFactionTemplate, UnitFlags, UnitLevel, UnitPowers, UnitTarget,
    WorldArrival, Zone,
};

mod bank_messages;
mod channels;
mod core_messages;
mod encounter_messages;
mod experience_messages;
mod gameplay_messages;
mod group_messages;
mod guild_bank_messages;
mod instance_messages;
mod interaction_messages;
mod inventory_messages;
mod loot_messages;
mod mail_messages;
mod merchant_messages;
mod mirror_timer_messages;
mod quest_messages;
mod registration;
mod spell_messages;
mod taxi_messages;
mod tooltip_messages;
mod trainer_messages;
mod transfer_messages;

pub use bank_messages::*;
pub use channels::*;
pub use core_messages::*;
pub use encounter_messages::*;
pub use experience_messages::*;
pub use gameplay_messages::*;
pub use group_messages::*;
pub use guild_bank_messages::*;
pub use instance_messages::*;
pub use interaction_messages::*;
pub use inventory_messages::*;
pub use loot_messages::*;
pub use mail_messages::*;
pub use merchant_messages::*;
pub use mirror_timer_messages::*;
pub use quest_messages::*;
pub use spell_messages::*;
pub use taxi_messages::*;
pub use tooltip_messages::*;
pub use trainer_messages::*;
pub use transfer_messages::*;

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
        bank_messages::register_bank_protocol(app);
        guild_bank_messages::register_guild_bank_protocol(app);
        trainer_messages::register_trainer_protocol(app);
        loot_messages::register_loot_protocol(app);
        mail_messages::register_mail_protocol(app);
        taxi_messages::register_taxi_protocol(app);
        tooltip_messages::register_tooltip_protocol(app);
        transfer_messages::register_transfer_protocol(app);
        encounter_messages::register_encounter_protocol(app);
        instance_messages::register_instance_protocol(app);
        mirror_timer_messages::register_mirror_timer_protocol(app);
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
    app.component::<MovementControl>().replicate();
    app.component::<WorldArrival>().replicate();
    app.component::<Zone>().replicate();
    app.component::<GuildMembership>().replicate();
    app.component::<PresenceStatus>().replicate();
    app.component::<EquipmentAppearance>().replicate();
    app.component::<crate::casting::CastState>().replicate();
    app.component::<UnitPowers>().replicate();
    app.component::<UnitAuras>().replicate();
    app.component::<UnitLevel>().replicate();
    app.component::<crate::level_scaling::LevelScaling>()
        .replicate();
    app.component::<UnitFactionTemplate>().replicate();
    app.component::<UnitFlags>().replicate();
    app.component::<UnitTarget>().replicate();
    app.component::<CreatureMotion>().replicate();
    app.component::<UnitPose>().replicate();
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
#[path = "protocol_trainer_tests.rs"]
mod trainer_tests;

#[cfg(test)]
#[path = "protocol_experience_tests.rs"]
mod experience_tests;

#[cfg(test)]
#[path = "protocol_bank_tests.rs"]
mod bank_tests;

#[cfg(test)]
#[path = "protocol_loot_tests.rs"]
mod loot_tests;

#[cfg(test)]
#[path = "protocol_taxi_tests.rs"]
mod taxi_tests;

#[cfg(test)]
#[path = "protocol_mail_tests.rs"]
mod mail_tests;

#[cfg(test)]
#[path = "protocol_tooltip_tests.rs"]
mod tooltip_tests;

#[cfg(test)]
#[path = "protocol_transfer_tests.rs"]
mod transfer_tests;

#[cfg(test)]
#[path = "protocol_encounter_tests.rs"]
mod encounter_tests;

#[cfg(test)]
#[path = "protocol_instance_tests.rs"]
mod instance_tests;

#[cfg(test)]
#[path = "protocol_mirror_timer_tests.rs"]
mod mirror_timer_tests;
