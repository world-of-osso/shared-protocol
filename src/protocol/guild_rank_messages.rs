//! Player-owned guild controls, on `GuildChannel` (ordered reliable).
//! Retail: Blizzard_GuildControlUI rank order, flags, money/day and bank tab controls.
use super::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::NetworkDirection;
use serde::{Deserialize, Serialize};

pub const GUILD_MIN_RANKS: usize = 2;
pub const GUILD_MAX_RANKS: usize = 10;
pub const GUILD_RIGHT_CHAT_LISTEN: u32 = 1 << 0;
pub const GUILD_RIGHT_CHAT_SPEAK: u32 = 1 << 1;
pub const GUILD_RIGHT_OFFICER_LISTEN: u32 = 1 << 2;
pub const GUILD_RIGHT_OFFICER_SPEAK: u32 = 1 << 3;
pub const GUILD_RIGHT_INVITE: u32 = 1 << 4;
pub const GUILD_RIGHT_REMOVE: u32 = 1 << 5;
pub const GUILD_RIGHT_PROMOTE: u32 = 1 << 6;
pub const GUILD_RIGHT_DEMOTE: u32 = 1 << 7;
pub const GUILD_RIGHT_EDIT_MOTD: u32 = 1 << 8;
pub const GUILD_RIGHT_EDIT_INFO: u32 = 1 << 9;
pub const GUILD_RIGHT_EDIT_OFFICER_NOTE: u32 = 1 << 10;
pub const GUILD_RIGHT_VIEW_OFFICER_NOTE: u32 = 1 << 11;
pub const GUILD_RIGHT_WITHDRAW_GOLD: u32 = 1 << 12;
pub const GUILD_RIGHT_WITHDRAW_REPAIR: u32 = 1 << 13;
pub const GUILD_RIGHT_ALL: u32 = (1 << 14) - 1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildRankTab {
    pub view: bool,
    pub deposit: bool,
    /// Stacks/day; zero denies withdrawals. Only rank zero is unlimited.
    pub withdrawals_per_day: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildRankSettings {
    pub name: String,
    pub rights: u32,
    /// Copper/day shared by withdrawals and repairs. Zero is no allowance.
    pub gold_per_day: u64,
    pub tabs: Vec<GuildRankTab>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildRankMember {
    pub character_name: String,
    /// Zero-based order; zero is the immutable Guild Master.
    pub rank: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildRanksState {
    pub ranks: Vec<GuildRankSettings>,
    pub members: Vec<GuildRankMember>,
    pub own_rank: u8,
    /// Purchased tab names, in bank order.
    pub tab_names: Vec<String>,
    pub error: Option<GuildRankError>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GuildRankRequest {
    Query,
    Add {
        name: String,
    },
    Remove {
        rank: u8,
    },
    Rename {
        rank: u8,
        name: String,
    },
    /// Move one position, never across rank zero. Member identities follow the rank.
    Move {
        rank: u8,
        up: bool,
    },
    SetPermissions {
        rank: u8,
        rights: u32,
        gold_per_day: u64,
    },
    SetTab {
        rank: u8,
        tab: u8,
        view: bool,
        deposit: bool,
        withdrawals_per_day: u32,
    },
    Promote {
        character_name: String,
    },
    Demote {
        character_name: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuildRankError {
    NotInGuild,
    Permissions,
    RankTooHigh,
    RankTooLow,
    RankInUse,
    TooManyRanks,
    TooFewRanks,
    InvalidName,
    MemberNotFound,
    WrongTab,
}

impl GuildRankError {
    /// Retail GlobalStrings. The client displays the same refusal as the server.
    pub fn message(self) -> &'static str {
        match self {
            Self::NotInGuild => "You are not in a guild.", // ERR_GUILD_PLAYER_NOT_IN_GUILD
            Self::Permissions => "You don't have permission to do that.", // ERR_GUILD_PERMISSIONS
            Self::RankTooHigh => "That player's rank is too high.", // ERR_GUILD_RANK_TOO_HIGH
            Self::RankTooLow => "That player is already at the lowest rank.", // ERR_GUILD_RANK_TOO_LOW
            Self::RankInUse => "That guild rank is currently in use.", // ERR_GUILD_RANK_IN_USE
            Self::TooManyRanks => "You already have too many guild ranks.", // ERR_GUILD_RANKS_TOO_MANY
            Self::TooFewRanks => "You must have at least 2 guild ranks.", // ERR_GUILD_RANKS_TOO_FEW
            Self::InvalidName => "Invalid guild rank name.", // ERR_GUILD_RANK_NAME_INVALID
            Self::MemberNotFound => "That player is not in your guild.", // ERR_GUILD_PLAYER_NOT_IN_GUILD_S (name omitted)
            Self::WrongTab => "Incorrect bank tab",                      // ERR_GUILD_BANK_WRONG_TAB
        }
    }
}

pub(super) fn register_guild_rank_protocol(app: &mut App) {
    app.register_protocol_message::<GuildRankRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GuildRanksState>()
        .add_direction(NetworkDirection::ServerToClient);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guild_ranks_wire_preserves_zero_limits_and_member_order() {
        let state = GuildRanksState {
            ranks: vec![GuildRankSettings {
                name: "Reader".into(),
                rights: GUILD_RIGHT_CHAT_LISTEN,
                gold_per_day: 0,
                tabs: vec![GuildRankTab {
                    view: true,
                    deposit: false,
                    withdrawals_per_day: 0,
                }],
            }],
            members: vec![GuildRankMember {
                character_name: "Alice".into(),
                rank: 1,
            }],
            own_rank: 1,
            tab_names: vec!["Materials".into()],
            error: None,
        };
        let bytes = bincode::serde::encode_to_vec(&state, bincode::config::standard()).unwrap();
        let (decoded, used): (GuildRanksState, _) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
        assert_eq!(decoded, state);
        assert_eq!(used, bytes.len());
        let request = GuildRankRequest::SetTab {
            rank: 1,
            tab: 0,
            view: true,
            deposit: false,
            withdrawals_per_day: 0,
        };
        assert_eq!(
            serde_json::from_str::<GuildRankRequest>(&serde_json::to_string(&request).unwrap())
                .unwrap(),
            request
        );
    }
}
