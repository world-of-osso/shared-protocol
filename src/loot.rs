//! Group loot method. Loot generation lives on the server; the loot window
//! messages are in `protocol::loot_messages`.

use serde::{Deserialize, Serialize};

/// Group loot distribution mode.
///
/// Ref: AzerothCore `LootMethod` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LootMode {
    /// Free for All — anyone can loot anything.
    FreeForAll,
    /// Round Robin — common items rotate between group members.
    RoundRobin,
    /// Need Before Greed — rare+ items trigger a roll window.
    NeedBeforeGreed,
    /// Personal Loot — each player gets their own independent roll (retail default).
    PersonalLoot,
}
