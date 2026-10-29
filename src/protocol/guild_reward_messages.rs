//! Mainline GuildRewards.lua `RequestGuildRewards` / `GetGuildRewardInfo`, served from
//! TrinityCore `guild_rewards` (`HandleRequestGuildRewardsList`, GuildHandler.cpp:545).
//! Guild standing itself travels in the reputation snapshot (faction 1168).
use super::ProtocolRegistrationExt;
use crate::reputation::Standing;
use bevy::prelude::*;
use lightyear::prelude::NetworkDirection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuildRewardsRequest;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildRewardAchievement {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildRewardItem {
    pub item_id: u32,
    /// `MinGuildRep`: lowest guild standing allowed to buy the item.
    pub min_standing: Standing,
    /// Copper.
    pub cost: u64,
    /// Guild achievements of the requester's faction the guild must have earned;
    /// `GetGuildRewardInfo` shows the first.
    pub achievements: Vec<GuildRewardAchievement>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuildRewardsError {
    NotInGuild,
}

/// Rewards whose `RaceMask` admits the requester, in `guild_rewards` order.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildRewardsState {
    pub rewards: Vec<GuildRewardItem>,
    pub error: Option<GuildRewardsError>,
}

pub(super) fn register_guild_reward_protocol(app: &mut App) {
    app.register_protocol_message::<GuildRewardsRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GuildRewardsState>()
        .add_direction(NetworkDirection::ServerToClient);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guild_rewards_wire_keeps_standing_cost_and_achievement() {
        let state = GuildRewardsState {
            rewards: vec![
                GuildRewardItem {
                    item_id: 63352,
                    min_standing: Standing::Honored,
                    cost: 1_500_000,
                    achievements: vec![GuildRewardAchievement {
                        id: 4989,
                        name: "A Class Act".into(),
                    }],
                },
                GuildRewardItem {
                    item_id: 61931,
                    min_standing: Standing::Friendly,
                    cost: 15_000_000,
                    achievements: vec![],
                },
            ],
            error: None,
        };
        let bytes = bitcode::serialize(&state).unwrap();
        assert_eq!(
            bitcode::deserialize::<GuildRewardsState>(&bytes).unwrap(),
            state
        );
        let refused = GuildRewardsState {
            rewards: vec![],
            error: Some(GuildRewardsError::NotInGuild),
        };
        let bytes = bitcode::serialize(&refused).unwrap();
        assert_eq!(
            bitcode::deserialize::<GuildRewardsState>(&bytes).unwrap(),
            refused
        );
    }
}
