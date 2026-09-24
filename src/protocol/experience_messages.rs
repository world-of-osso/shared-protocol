//! Owner-only experience messages (Retail `SMSG_LOG_XP_GAIN` and the
//! `ActivePlayerData` `XP`/`NextLevelXP`/rested fields). The server sends
//! `PlayerXpUpdate` on enter world, after every gain and after a level-up, and
//! `LogXpGain` for each gain. Both use `ExperienceChannel`.

use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

/// Reliable ordered channel for experience messages, server to client.
pub struct ExperienceChannel;

/// The player's XP bar: XP into the current level, XP the level needs
/// (0 at the level cap) and the rested pool.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerXpUpdate {
    pub xp: u32,
    pub next_level_xp: u32,
    pub rested_xp: u32,
}

/// Why XP was gained (Retail `PlayerLogXPReason`, plus quest turn-ins).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum XpGainReason {
    Kill,
    Quest,
    NoKill,
}

/// One XP gain (Retail `LogXPGain`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct LogXpGain {
    /// Killed unit (server entity bits) for kill XP.
    pub victim: Option<u64>,
    /// Total gained, rested bonus included.
    pub original: u32,
    /// Gain before the rested bonus.
    pub amount: u32,
    /// Group XP rate (1.0 solo).
    pub group_bonus: f32,
    pub reason: XpGainReason,
}

pub(super) fn register_experience_protocol(app: &mut App) {
    app.add_channel::<ExperienceChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<PlayerXpUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<LogXpGain>()
        .add_direction(NetworkDirection::ServerToClient);
}
