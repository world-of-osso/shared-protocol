//! The realm's game time (TrinityCore `SMSG_LOGIN_SET_TIME_SPEED`, MiscPackets.cpp:48-57,
//! sent by `Player::SendInitialPacketsBeforeAddToMap`, Player.cpp:24988-24996): the client's
//! time of day, which selects the LightData keyframes of the sky, fog and scene light.

use crate::protocol::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

/// Reliable ordered channel for the game time, server to client.
pub struct WorldTimeChannel;

/// TrinityCore `TimeSpeed` 0.01666667: game minutes per real second, so game time runs at
/// real time.
pub const GAME_TIME_SPEED: f32 = 0.016_666_67;

/// `SMSG_LOGIN_SET_TIME_SPEED`: the game time at sending and how fast it advances.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct LoginSetTimeSpeed {
    /// `GameTime::GetWowTime`: the realm's local wall-clock time (UTC plus the server's
    /// zone offset), in seconds since the Unix epoch.
    pub game_time_seconds: i64,
    /// Game minutes per real second (`GAME_TIME_SPEED`).
    pub new_speed: f32,
}

impl LoginSetTimeSpeed {
    /// Seconds since local midnight of `game_time_seconds`.
    pub fn second_of_day(&self) -> u32 {
        self.game_time_seconds.rem_euclid(86_400) as u32
    }
}

pub(super) fn register_world_time_protocol(app: &mut App) {
    app.add_channel::<WorldTimeChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<LoginSetTimeSpeed>()
        .add_direction(NetworkDirection::ServerToClient);
}
