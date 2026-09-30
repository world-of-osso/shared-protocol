//! Flight master messages (Retail `FlightMapFrame` / `C_TaxiMap`, AzerothCore
//! `SMSG_SHOWTAXINODES` / `CMSG_ACTIVATETAXI` / `SMSG_NEW_TAXI_PATH`).
//!
//! Opening the flight master role discovers its node (`TaxiNodeDiscovered` the
//! first time) and sends `TaxiMap`: every node of the continent the player's
//! faction can see, with its state, and for reachable nodes the cost and the hops
//! of the cheapest route over known nodes. `ActivateTaxi` takes off; refusals come
//! back as `TaxiFailed`. The flight itself is `MovementControl` on the player.
//! All messages use `TaxiChannel`.

use crate::protocol::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

/// Reliable ordered channel for flight master messages, bidirectional.
pub struct TaxiChannel;

/// Retail `Enum.FlightPathState`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaxiNodeState {
    Current,
    Reachable,
    Unreachable,
}

/// One flight point (Retail `TaxiNodeInfo`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TaxiNodeInfo {
    /// `TaxiNodes.ID`.
    pub node: u32,
    pub name: String,
    /// WoW world coordinates (`TaxiNodes.Pos_0`, `Pos_1`).
    pub world_x: f32,
    pub world_y: f32,
    pub state: TaxiNodeState,
    /// Copper for the whole route; 0 unless `Reachable`.
    pub cost: u64,
    /// Node ids from the current node to this one, both included; empty unless
    /// `Reachable`.
    pub route: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TaxiMap {
    pub npc: u64,
    /// `TaxiNodes.ContinentID` of the current node.
    pub continent: u32,
    pub nodes: Vec<TaxiNodeInfo>,
}

/// Fly from the flight master's node to `destination` (`TakeTaxiNode`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivateTaxi {
    pub npc: u64,
    pub destination: u32,
}

/// The player learned a flight point (`ERR_NEWTAXIPATH`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaxiNodeDiscovered {
    pub node: u32,
}

/// Why the server refused a flight; `message` is the Retail UI error text.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaxiError {
    NoVendorNearby,
    NotVisited,
    NoPath,
    SameNode,
    NotEnoughMoney,
    InCombat,
    Busy,
}

impl TaxiError {
    pub fn message(self) -> &'static str {
        match self {
            Self::NoVendorNearby => "There is no taxi vendor nearby!", // ERR_TAXINOVENDORNEARBY
            Self::NotVisited => "You haven't reached that flight location on foot yet!", // ERR_TAXINOTVISITED
            Self::NoPath => "There is no direct path to that destination!", // ERR_TAXINOSUCHPATH
            Self::SameNode => "You are already there!",                     // ERR_TAXISAMENODE
            Self::NotEnoughMoney => "You don't have enough money!", // ERR_TAXINOTENOUGHMONEY
            Self::InCombat => "You cannot take a flight path while you are in combat.", // ERR_TAXIINCOMBAT
            Self::Busy => "You are busy and can't use the taxi service now.", // ERR_TAXIPLAYERBUSY
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaxiFailed {
    pub npc: u64,
    pub error: TaxiError,
}

pub(super) fn register_taxi_protocol(app: &mut App) {
    app.add_channel::<TaxiChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
    app.register_protocol_message::<TaxiMap>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<TaxiNodeDiscovered>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<TaxiFailed>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<ActivateTaxi>()
        .add_direction(NetworkDirection::ClientToServer);
}
