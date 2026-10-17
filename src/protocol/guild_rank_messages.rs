//! Player-owned guild controls, on `GuildChannel` (ordered reliable).
//! Retail: Blizzard_GuildControlUI rank order, flags, money/day and bank tab controls.
use super::ProtocolRegistrationExt;
use bevy::prelude::*;
use lightyear::prelude::NetworkDirection;
use serde::{Deserialize, Serialize};

pub const GUILD_MIN_RANKS: usize = 2;
pub const GUILD_MAX_RANKS: usize = 10;
/// Mainline GameDialogDefs.lua SET_GUILD_COMMUNITIY_NOTE maxLetters.
pub const GUILD_MEMBER_NOTE_MAX_LETTERS: usize = 31;

pub fn valid_guild_member_note(note: &str) -> bool {
    note.chars().count() <= GUILD_MEMBER_NOTE_MAX_LETTERS && !note.chars().any(char::is_control)
}
pub const GUILD_RIGHT_CHAT_LISTEN: u32 = 1 << 0;
pub const GUILD_RIGHT_CHAT_SPEAK: u32 = 1 << 1;
pub const GUILD_RIGHT_OFFICER_LISTEN: u32 = 1 << 2;
pub const GUILD_RIGHT_OFFICER_SPEAK: u32 = 1 << 3;
pub const GUILD_RIGHT_INVITE: u32 = 1 << 4;
pub const GUILD_RIGHT_REMOVE: u32 = 1 << 5;
// TrinityCore a352b1fa, Guild.h GuildRankRights. Checkbox IDs are NOT bit offsets.
pub const GUILD_RIGHT_PROMOTE: u32 = 0x0000_0080;
pub const GUILD_RIGHT_DEMOTE: u32 = 0x0000_0100;
pub const GUILD_RIGHT_EDIT_MOTD: u32 = 0x0000_1000;
pub const GUILD_RIGHT_EDIT_INFO: u32 = 0x0001_0000;
pub const GUILD_RIGHT_EDIT_PUBLIC_NOTE: u32 = 0x0000_2000;
pub const GUILD_RIGHT_EDIT_OFFICER_NOTE: u32 = 0x0000_8000;
pub const GUILD_RIGHT_VIEW_OFFICER_NOTE: u32 = 0x0000_4000;
pub const GUILD_RIGHT_WITHDRAW_GOLD: u32 = 0x0008_0000;
pub const GUILD_RIGHT_WITHDRAW_REPAIR: u32 = 0x0004_0000;
/// Implemented rights only; TC GR_RIGHT_ALL also contains unsupported/unnamed bits.
pub const GUILD_RIGHT_ALL: u32 = GUILD_RIGHT_CHAT_LISTEN
    | GUILD_RIGHT_CHAT_SPEAK
    | GUILD_RIGHT_OFFICER_LISTEN
    | GUILD_RIGHT_OFFICER_SPEAK
    | GUILD_RIGHT_INVITE
    | GUILD_RIGHT_REMOVE
    | GUILD_RIGHT_PROMOTE
    | GUILD_RIGHT_DEMOTE
    | GUILD_RIGHT_EDIT_MOTD
    | GUILD_RIGHT_EDIT_INFO
    | GUILD_RIGHT_EDIT_OFFICER_NOTE
    | GUILD_RIGHT_EDIT_PUBLIC_NOTE
    | GUILD_RIGHT_VIEW_OFFICER_NOTE
    | GUILD_RIGHT_WITHDRAW_GOLD
    | GUILD_RIGHT_WITHDRAW_REPAIR;

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
    /// Mainline GuildRoster note dialog, with authenticated actor and same-guild target.
    SetMemberNote {
        character_name: String,
        note: String,
        officer: bool,
    },
    /// Leave one's guild; Guild Master must transfer leadership first.
    Leave,
    /// Remove a lower-ranked member, subject to GUILD_RIGHT_REMOVE.
    RemoveMember {
        character_name: String,
    },
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
    /// Authenticated actor's current guild only; replies with GuildEventLog.
    QueryEventLog,
}

/// TrinityCore a352b1fa SharedDefines.h GUILD_EVENTLOG_MAX_RECORDS.
pub const GUILD_EVENT_LOG_MAX_RECORDS: usize = 100;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GuildEventKind {
    Invite {
        actor: String,
        member: String,
    },
    Join {
        member: String,
    },
    Promote {
        actor: String,
        member: String,
        rank: String,
    },
    Demote {
        actor: String,
        member: String,
        rank: String,
    },
    Remove {
        actor: String,
        member: String,
    },
    Quit {
        member: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildEvent {
    /// Unix seconds at the real mutation, not query time.
    pub timestamp: u64,
    /// Names/rank are captured at event time, not reconstructed from today's roster.
    pub kind: GuildEventKind,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildEventLog {
    /// Oldest to newest insertion order; Mainline GuildInfo.lua renders in reverse.
    pub events: Vec<GuildEvent>,
    pub server_time: u64,
    pub error: Option<GuildRankError>,
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
    LeaderCannotLeave,
    NoteTooLong,
}

impl GuildRankError {
    /// Retail GlobalStrings. The client displays the same refusal as the server.
    pub fn message(self) -> &'static str {
        match self {
            Self::NoteTooLong => "Guild notes must be a single line of at most 31 characters.",
            Self::LeaderCannotLeave => {
                "You must promote a new Guild Master using /gleader before leaving the guild."
            } // ERR_GUILD_LEADER_LEAVE
            Self::NotInGuild => "You are not in a guild.", // ERR_GUILD_PLAYER_NOT_IN_GUILD
            Self::Permissions => "You don't have permission to do that.", // ERR_GUILD_PERMISSIONS
            Self::RankTooHigh => "That player's rank is too high", // ERR_GUILD_RANK_TOO_HIGH_S (%s = That player)
            Self::RankTooLow => "That player is already at the lowest rank", // ERR_GUILD_RANK_TOO_LOW_S
            Self::RankInUse => "That guild rank is currently in use.", // ERR_GUILD_RANK_IN_USE
            Self::TooManyRanks | Self::TooFewRanks => "You don't have permission to do that.", // ERR_GUILD_PERMISSIONS (Retail disables the buttons)
            Self::InvalidName => "Invalid guild name.", // ERR_GUILD_NAME_INVALID
            Self::MemberNotFound => "That player is not in your guild.", // ERR_GUILD_PLAYER_NOT_IN_GUILD_S (%s = That player)
            Self::WrongTab => "Incorrect bank tab",                      // ERR_GUILD_BANK_WRONG_TAB
        }
    }
}

pub(super) fn register_guild_rank_protocol(app: &mut App) {
    app.register_protocol_message::<GuildRankRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GuildRanksState>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_protocol_message::<GuildEventLog>()
        .add_direction(NetworkDirection::ServerToClient);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guild_lifecycle_wire_preserves_member_and_invitation() {
        for request in [
            GuildRankRequest::Leave,
            GuildRankRequest::SetMemberNote {
                character_name: "Cara".into(),
                note: "Alchemy".into(),
                officer: false,
            },
            GuildRankRequest::SetMemberNote {
                character_name: "Cara".into(),
                note: "Raid healer".into(),
                officer: true,
            },
            GuildRankRequest::RemoveMember {
                character_name: "Cara".into(),
            },
        ] {
            let bytes =
                bincode::serde::encode_to_vec(&request, bincode::config::standard()).unwrap();
            let (decoded, used): (GuildRankRequest, _) =
                bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
            assert_eq!(decoded, request);
            assert_eq!(used, bytes.len());
        }
        let invite = crate::protocol::GuildInvitation {
            guild_id: 7,
            guild_name: "Bank Testers".into(),
            inviter_name: "Alice".into(),
            tabard: None,
        };
        let bytes = bincode::serde::encode_to_vec(&invite, bincode::config::standard()).unwrap();
        let (decoded, used): (crate::protocol::GuildInvitation, _) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
        assert_eq!(decoded, invite);
        assert_eq!(used, bytes.len());
    }

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
