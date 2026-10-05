//! Creature threat tables: TrinityCore a352b1fa ThreatManager.cpp:829-875
//! (`ThreatClear`, `ThreatUpdate`, `HighestThreatUpdate`). An empty table clears it.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatUpdate {
    pub creature: u64,
    pub victim: Option<u64>,
    /// All units on this creature's threat table, descending raw threat.
    pub entries: Vec<ThreatUnit>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatUnit {
    pub unit: u64,
    pub name: String,
    pub class_id: u8,
    pub raw_threat: f32,
    /// UnitDetailedThreatSituation status: 0 low, 1 gaining, 2 insecure tank, 3 tank.
    pub status: u8,
    /// Relative to the creature's current victim (can exceed 100).
    pub raw_percent: f32,
    /// Relative to the 110% melee / 130% ranged aggro threshold, capped at 100.
    pub scaled_percent: f32,
}
