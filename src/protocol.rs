use bevy::prelude::*;

pub use crate::protocol_snapshots::*;

use crate::components::{
    ActiveSpec, CombatRatings, CombatStatus, CreatureClassification, CreatureMotion, DerivedStats,
    EquipmentAppearance, Gold, GuildMembership, Health, Mana, ModelDisplay, Mounted,
    MovementControl, MovementSpeed, Npc, Player, PlayerMotion, PlayerStandState, Position,
    PresenceStatus, Rotation, UnitAuras, UnitFactionTemplate, UnitFlags, UnitLevel, UnitPose,
    UnitPowers, UnitRunes, UnitStats, UnitSummonedBy, UnitTap, UnitTarget, UnitThreatList,
    UnitVignette, WorldArrival, Zone,
};

mod achievement_catalog_messages;
mod bank_messages;
mod channels;
mod core_messages;
mod damage_meter_messages;
mod encounter_messages;
mod experience_messages;
mod gameplay_messages;
mod group_messages;
mod guild_bank_messages;
mod guild_rank_messages;
mod instance_messages;
mod interaction_messages;
mod inventory_messages;
mod layout_tracer;
mod loot_messages;
mod mail_messages;
mod merchant_messages;
mod mirror_timer_messages;
mod pet_messages;
mod protocol_check;
mod protocol_layout;
mod quest_messages;
mod registration;
mod spell_messages;
mod taxi_messages;
mod threat_messages;
mod tooltip_messages;
mod trainer_messages;
mod transfer_messages;
mod world_time_messages;

pub use achievement_catalog_messages::*;
pub use bank_messages::*;
pub use channels::*;
pub use core_messages::*;
pub use damage_meter_messages::*;
pub use encounter_messages::*;
pub use experience_messages::*;
pub use gameplay_messages::*;
pub use group_messages::*;
pub use guild_bank_messages::*;
pub use guild_rank_messages::*;
pub use instance_messages::*;
pub use interaction_messages::*;
pub use inventory_messages::*;
pub use loot_messages::*;
pub use mail_messages::*;
pub use merchant_messages::*;
pub use mirror_timer_messages::*;
pub use pet_messages::*;
pub use protocol_check::{
    ProtocolCheckChannel, ProtocolCheckTimeout, ProtocolFingerprint, ProtocolRejected,
    ProtocolVerified, defer_lightyear_protocol_check,
};
pub use protocol_layout::ProtocolRegistrationExt;
pub use quest_messages::*;
pub use spell_messages::*;
pub use taxi_messages::*;
pub use threat_messages::*;
pub use tooltip_messages::*;
pub use trainer_messages::*;
pub use transfer_messages::*;
pub use world_time_messages::*;

use registration::{register_channels, register_messages};

/// Registers shared protocol: components for replication and channels.
/// Must be added AFTER `ServerPlugins`/`ClientPlugins` but BEFORE any entity is spawned.
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // First, so the check's message and channel ids match across differing registries.
        protocol_check::register_protocol_check(app);
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
        guild_rank_messages::register_guild_rank_protocol(app);
        trainer_messages::register_trainer_protocol(app);
        loot_messages::register_loot_protocol(app);
        mail_messages::register_mail_protocol(app);
        taxi_messages::register_taxi_protocol(app);
        tooltip_messages::register_tooltip_protocol(app);
        transfer_messages::register_transfer_protocol(app);
        encounter_messages::register_encounter_protocol(app);
        instance_messages::register_instance_protocol(app);
        mirror_timer_messages::register_mirror_timer_protocol(app);
        world_time_messages::register_world_time_protocol(app);
    }

    fn finish(&self, app: &mut App) {
        protocol_check::finish_protocol_check(app);
    }
}

fn register_replicated_components(app: &mut App) {
    app.protocol_component::<Position>().replicate();
    app.protocol_component::<Health>().replicate();
    app.protocol_component::<Mana>().replicate();
    app.protocol_component::<Gold>().replicate();
    app.protocol_component::<Player>().replicate();
    app.protocol_component::<ActiveSpec>().replicate();
    app.protocol_component::<Npc>().replicate();
    app.protocol_component::<ModelDisplay>().replicate();
    app.protocol_component::<Rotation>().replicate();
    app.protocol_component::<MovementSpeed>().replicate();
    app.protocol_component::<CombatStatus>().replicate();
    app.protocol_component::<Mounted>().replicate();
    app.protocol_component::<MovementControl>().replicate();
    app.protocol_component::<WorldArrival>().replicate();
    app.protocol_component::<Zone>().replicate();
    app.protocol_component::<GuildMembership>().replicate();
    app.protocol_component::<PresenceStatus>().replicate();
    app.protocol_component::<EquipmentAppearance>().replicate();
    app.protocol_component::<crate::casting::CastState>()
        .replicate();
    app.protocol_component::<UnitPowers>().replicate();
    app.protocol_component::<UnitAuras>().replicate();
    app.protocol_component::<UnitLevel>().replicate();
    app.protocol_component::<crate::level_scaling::LevelScaling>()
        .replicate();
    app.protocol_component::<UnitFactionTemplate>().replicate();
    app.protocol_component::<UnitFlags>().replicate();
    app.protocol_component::<UnitTarget>().replicate();
    app.protocol_component::<CreatureMotion>().replicate();
    app.protocol_component::<PlayerMotion>().replicate();
    app.protocol_component::<UnitPose>().replicate();
    app.protocol_component::<UnitThreatList>().replicate();
    app.protocol_component::<UnitTap>().replicate();
    app.protocol_component::<UnitRunes>().replicate();
    // The character sheet's stats: the server shows them to the owning client only.
    app.protocol_component::<UnitStats>().replicate();
    app.protocol_component::<CombatRatings>().replicate();
    app.protocol_component::<DerivedStats>().replicate();
    app.protocol_component::<PlayerStandState>().replicate();
    app.protocol_component::<crate::death::DeathState>()
        .replicate();
    app.protocol_component::<CreatureClassification>()
        .replicate();
    app.protocol_component::<UnitVignette>().replicate();
    app.protocol_component::<UnitSummonedBy>().replicate();
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

#[cfg(test)]
#[path = "protocol_world_time_tests.rs"]
mod world_time_tests;

#[cfg(test)]
#[path = "protocol_damage_meter_tests.rs"]
mod damage_meter_tests;

#[cfg(test)]
#[path = "protocol_pet_tests.rs"]
mod pet_tests;

#[cfg(test)]
#[path = "protocol_player_motion_tests.rs"]
mod player_motion_tests;
