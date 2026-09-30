//! Owner-only mirror timer messages (TrinityCore `SMSG_START_MIRROR_TIMER`,
//! `SMSG_PAUSE_MIRROR_TIMER`, `SMSG_STOP_MIRROR_TIMER`, MiscPackets.cpp:358-385; Retail
//! `MIRROR_TIMER_START`, `_PAUSE`, `_STOP`): the fatigue, breath and feign-death bars.
//!
//! The server owns the values: a start carries the current value and how fast it changes,
//! and the client counts a running bar between messages (`GetMirrorTimerProgress`). All
//! three use `MirrorTimerChannel`, so a stop never overtakes the start before it.

use crate::protocol::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

/// Reliable ordered channel for mirror timer messages, server to client.
pub struct MirrorTimerChannel;

/// TrinityCore `MirrorTimerType` (Player.h): `FATIGUE_TIMER`.
pub const MIRROR_TIMER_FATIGUE: u8 = 0;
/// `BREATH_TIMER`.
pub const MIRROR_TIMER_BREATH: u8 = 1;
/// `FIRE_TIMER`; the Retail client shows it as the feign-death bar.
pub const MIRROR_TIMER_FEIGN_DEATH: u8 = 2;

/// `SMSG_START_MIRROR_TIMER`: `timer` shows `value_ms` of `max_value_ms`, changing by
/// `scale` milliseconds per elapsed millisecond (-1 drains, 10 refills) unless `paused`.
/// A start for a shown timer replaces its values.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct MirrorTimerStart {
    pub timer: u8,
    pub value_ms: i32,
    pub max_value_ms: i32,
    pub scale: f32,
    pub paused: bool,
    /// Spell the bar belongs to (feign death); 0 for breath and fatigue.
    pub spell_id: i32,
}

/// `SMSG_PAUSE_MIRROR_TIMER`: freeze (`paused`) or resume a shown timer.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MirrorTimerPause {
    pub timer: u8,
    pub paused: bool,
}

/// `SMSG_STOP_MIRROR_TIMER`: hide the timer.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MirrorTimerStop {
    pub timer: u8,
}

pub(super) fn register_mirror_timer_protocol(app: &mut App) {
    app.add_channel::<MirrorTimerChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<MirrorTimerStart>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<MirrorTimerPause>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<MirrorTimerStop>()
        .add_direction(NetworkDirection::ServerToClient);
}
