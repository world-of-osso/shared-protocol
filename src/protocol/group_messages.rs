//! Party/raid management messages.
//!
//! Channel: everything here uses `GroupChannel`. The older `GroupInviteIntent` /
//! `GroupUninviteIntent` intents (core messages) stay valid; the server accepts them
//! from any channel. The roster itself is `GroupRosterSnapshot` and results are
//! `GroupCommandResponse` (protocol snapshots).

use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::loot::LootMode;
use crate::protocol_snapshots::GroupRoleSnapshot;

/// Reliable ordered channel for party/raid management, bidirectional.
pub struct GroupChannel;

/// Seconds the invitee has to answer a party invite (Retail `PARTY_INVITE` popup).
pub const GROUP_INVITE_TIMEOUT_SECS: f32 = 60.0;
/// Seconds members have to answer a ready check.
pub const READY_CHECK_DURATION_SECS: f32 = 30.0;

/// Client answers the pending party invite (`AcceptGroup` / `DeclineGroup`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RespondGroupInvite {
    pub accept: bool,
}

/// Client leaves its current group (`LeaveParty`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LeaveGroup;

/// Leader hands leadership to another member (`PromoteToLeader`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PromoteGroupLeader {
    pub name: String,
}

/// Leader converts the party into a raid (`C_PartyInfo.ConvertToRaid`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ConvertGroupToRaid;

/// Leader moves a raid member to subgroup 1–8 (`SetRaidSubgroup`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SetRaidSubgroup {
    pub name: String,
    pub subgroup: u8,
}

/// Sets a member's assigned role. Members may set their own; the leader may set anyone's.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SetGroupRole {
    pub name: String,
    pub role: GroupRoleSnapshot,
}

/// Leader sets the group loot method (stored and reported in the roster only).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SetGroupLootMethod {
    pub method: LootMode,
}

/// Leader starts a ready check (`DoReadyCheck`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct StartReadyCheck;

/// Member answers the active ready check (`ConfirmReadyCheck`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RespondReadyCheck {
    pub ready: bool,
}

/// Server asks the invitee to accept a party invite: the `PARTY_INVITE` popup data.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GroupInvitePrompt {
    pub inviter_name: String,
    pub timeout_secs: f32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadyCheckAnswer {
    Pending,
    Ready,
    NotReady,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ReadyCheckMemberSnapshot {
    pub name: String,
    pub answer: ReadyCheckAnswer,
}

/// Ready check state, broadcast to every member on start, on each answer and on finish.
/// On finish, members still `Pending` did not answer in time (Retail: Away).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReadyCheckUpdate {
    pub initiator_name: String,
    pub time_remaining_secs: f32,
    pub members: Vec<ReadyCheckMemberSnapshot>,
    pub finished: bool,
}

/// Which Retail `GlobalStrings` entry a `GroupCommandResponse` carries.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupMessageCode {
    BadPlayerName,
    AlreadyInGroup,
    GroupFull,
    NotLeader,
    NotInGroup,
    NotInRaid,
    InviteSelf,
    InvitePartyBusy,
    TargetNotInGroup,
    InvitePlayer,
    DeclineGroup,
    JoinedGroup,
    LeftGroup,
    LeftGroupYou,
    UninviteYou,
    RaidYouJoined,
    RaidYouLeft,
    RaidMemberAdded,
    RaidMemberRemoved,
    NewLeader,
    NewLeaderYou,
    GroupDisbanded,
    PartyConvertedToRaid,
    SetLootFreeForAll,
    SetLootRoundRobin,
    SetLootNeedBeforeGreed,
    SetLootPersonal,
    ReadyCheckInProgress,
    ReadyCheckStarted,
    ReadyCheckAllReady,
    ReadyCheckFinished,
}

impl GroupMessageCode {
    /// The Retail `GlobalStrings` key for this result.
    pub fn global_string_key(self) -> &'static str {
        match self {
            Self::BadPlayerName => "ERR_BAD_PLAYER_NAME_S",
            Self::AlreadyInGroup => "ERR_ALREADY_IN_GROUP_S",
            Self::GroupFull => "ERR_GROUP_FULL",
            Self::NotLeader => "ERR_NOT_LEADER",
            Self::NotInGroup => "ERR_NOT_IN_GROUP",
            Self::NotInRaid => "ERR_NOT_IN_RAID",
            Self::InviteSelf => "ERR_INVITE_SELF",
            Self::InvitePartyBusy => "ERR_INVITE_PARTY_BUSY",
            Self::TargetNotInGroup => "ERR_TARGET_NOT_IN_GROUP_S",
            Self::InvitePlayer => "ERR_INVITE_PLAYER_S",
            Self::DeclineGroup => "ERR_DECLINE_GROUP_S",
            Self::JoinedGroup => "ERR_JOINED_GROUP_S",
            Self::LeftGroup => "ERR_LEFT_GROUP_S",
            Self::LeftGroupYou => "ERR_LEFT_GROUP_YOU",
            Self::UninviteYou => "ERR_UNINVITE_YOU",
            Self::RaidYouJoined => "ERR_RAID_YOU_JOINED",
            Self::RaidYouLeft => "ERR_RAID_YOU_LEFT",
            Self::RaidMemberAdded => "ERR_RAID_MEMBER_ADDED_S",
            Self::RaidMemberRemoved => "ERR_RAID_MEMBER_REMOVED_S",
            Self::NewLeader => "ERR_NEW_LEADER_S",
            Self::NewLeaderYou => "ERR_NEW_LEADER_YOU",
            Self::GroupDisbanded => "ERR_GROUP_DISBANDED",
            Self::PartyConvertedToRaid => "ERR_PARTY_CONVERTED_TO_RAID",
            Self::SetLootFreeForAll => "ERR_SET_LOOT_FREEFORALL",
            Self::SetLootRoundRobin => "ERR_SET_LOOT_ROUNDROBIN",
            Self::SetLootNeedBeforeGreed => "ERR_SET_LOOT_NBG",
            Self::SetLootPersonal => "ERR_SET_LOOT_PERSONAL",
            Self::ReadyCheckInProgress => "ERR_READY_CHECK_IN_PROGRESS",
            Self::ReadyCheckStarted => "READY_CHECK_MESSAGE",
            Self::ReadyCheckAllReady => "READY_CHECK_ALL_READY",
            Self::ReadyCheckFinished => "READY_CHECK_FINISHED",
        }
    }

    /// Retail English text; `name` fills the `%s` of `_S` entries.
    pub fn format(self, name: &str) -> String {
        match self {
            Self::BadPlayerName => format!("Cannot find player '{name}'."),
            Self::AlreadyInGroup => format!("{name} is already in a group."),
            Self::GroupFull => "Your party is full.".into(),
            Self::NotLeader => "You are not the party leader.".into(),
            Self::NotInGroup => "You aren't in a party.".into(),
            Self::NotInRaid => "You are not in a raid group".into(),
            Self::InviteSelf => "You can't invite yourself to a group.".into(),
            Self::InvitePartyBusy => {
                "Cannot invite additional players until the party is formed".into()
            }
            Self::TargetNotInGroup => format!("{name} is not in your party."),
            Self::InvitePlayer => format!("You have invited {name} to join your group."),
            Self::DeclineGroup => format!("{name} declines your group invitation."),
            Self::JoinedGroup => format!("{name} joins the party."),
            Self::LeftGroup => format!("{name} leaves the party."),
            Self::LeftGroupYou => "You leave the group.".into(),
            Self::UninviteYou => "You have been removed from the group.".into(),
            Self::RaidYouJoined => "You have joined a raid group.".into(),
            Self::RaidYouLeft => "You have left the raid group.".into(),
            Self::RaidMemberAdded => format!("{name} has joined the raid group."),
            Self::RaidMemberRemoved => format!("{name} has left the raid group."),
            Self::NewLeader => format!("{name} is now the group leader."),
            Self::NewLeaderYou => "You are now the group leader.".into(),
            Self::GroupDisbanded => "Your group has been disbanded.".into(),
            Self::PartyConvertedToRaid => "Party converted to Raid".into(),
            Self::SetLootFreeForAll => "Looting set to Free for All.".into(),
            Self::SetLootRoundRobin => "Looting set to Round Robin.".into(),
            Self::SetLootNeedBeforeGreed => "Looting set to Need Before Greed.".into(),
            Self::SetLootPersonal => "Looting set to Personal.".into(),
            Self::ReadyCheckInProgress => "You are already running a ready check".into(),
            Self::ReadyCheckStarted => format!("{name} has initiated a ready check."),
            Self::ReadyCheckAllReady => "Everyone is Ready".into(),
            Self::ReadyCheckFinished => "Ready check finished".into(),
        }
    }
}

pub(super) fn register_group_messages(app: &mut App) {
    for_client::<RespondGroupInvite>(app);
    for_client::<LeaveGroup>(app);
    for_client::<PromoteGroupLeader>(app);
    for_client::<ConvertGroupToRaid>(app);
    for_client::<SetRaidSubgroup>(app);
    for_client::<SetGroupRole>(app);
    for_client::<SetGroupLootMethod>(app);
    for_client::<StartReadyCheck>(app);
    for_client::<RespondReadyCheck>(app);
    app.register_message::<GroupInvitePrompt>()
        .add_direction(NetworkDirection::ServerToClient);
    app.register_message::<ReadyCheckUpdate>()
        .add_direction(NetworkDirection::ServerToClient);
    app.add_channel::<GroupChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
}

fn for_client<M: lightyear::prelude::Message + Serialize + serde::de::DeserializeOwned>(
    app: &mut App,
) {
    app.register_message::<M>()
        .add_direction(NetworkDirection::ClientToServer);
}
