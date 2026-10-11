//! Retail ChallengesKeystoneFrame and ScenarioObjectiveTracker snapshots.
//! Unsourced policy is represented by absence, never a fabricated rating/rotation.
use super::{InstanceChannel, ProtocolRegistrationExt};
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, bitcode::Encode, bitcode::Decode, Clone, Debug, PartialEq, Eq)]
pub struct KeystoneInfo {
    pub item_guid: u64,
    pub item_id: u32,
    pub challenge_id: u32,
    pub level: u32,
    pub affixes: Vec<u32>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ChallengeRequest {
    Query,
    Slot { font: u64, item_guid: u64 },
    Remove,
    Start,
    Abandon,
}
#[derive(
    Serialize, Deserialize, bitcode::Encode, bitcode::Decode, Clone, Copy, Debug, PartialEq, Eq,
)]
pub enum ChallengePhase {
    Slotted,
    Countdown,
    Running,
    Completed,
    Abandoned,
}
#[derive(Serialize, Deserialize, bitcode::Encode, bitcode::Decode, Clone, Debug, PartialEq, Eq)]
pub struct ChallengeBoss {
    pub encounter_id: u32,
    pub name: String,
    pub defeated: bool,
}
#[derive(Serialize, Deserialize, bitcode::Encode, bitcode::Decode, Clone, Debug, PartialEq, Eq)]
pub struct ChallengeRun {
    pub run_id: u64,
    pub map_id: u32,
    pub instance_id: u32,
    pub name: String,
    pub key: KeystoneInfo,
    pub phase: ChallengePhase,
    pub countdown_ms: u64,
    pub elapsed_ms: u64,
    pub time_limit_ms: u64,
    pub deaths: u32,
    pub time_lost_ms: u64,
    pub forces: u64,
    pub forces_required: u64,
    pub bosses: Vec<ChallengeBoss>,
    pub upgrade_levels: u32,
    pub resulting_level: Option<u32>,
    pub rating: Option<u32>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ChallengeSnapshot {
    pub keys: Vec<KeystoneInfo>,
    pub run: Option<ChallengeRun>,
    pub error: Option<String>,
}

pub(super) fn register(app: &mut App) {
    app.register_protocol_message::<ChallengeRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<ChallengeSnapshot>()
        .add_direction(NetworkDirection::ServerToClient);
}
// Messages use the existing reliable, ordered InstanceChannel.
const _: Option<InstanceChannel> = None;
