//! Mainline GuildNews.lua/ GuildUtil.lua, separate from guild lifecycle history.
#[cfg(feature = "transport")]
use super::ProtocolRegistrationExt;
#[cfg(feature = "transport")]
use bevy::prelude::*;
#[cfg(feature = "transport")]
use lightyear::prelude::NetworkDirection;
use serde::{Deserialize, Serialize};

/// TrinityCore a352b1fa SharedDefines.h GUILD_NEWSLOG_MAX_RECORDS.
pub const GUILD_NEWS_MAX_RECORDS: usize = 250;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GuildNewsRequest {
    Query,
    /// Immutable news identity, not a filtered/sorted client row index.
    SetSticky {
        id: u64,
        sticky: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GuildNewsKind {
    PlayerAchievement {
        member: String,
        achievement_id: u32,
        name: String,
        description: String,
    },
    ItemLooted {
        member: String,
        item_id: u32,
    },
    ItemCrafted {
        member: String,
        item_id: u32,
    },
    ItemPurchased {
        member: String,
        item_id: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildNewsEntry {
    pub id: u64,
    pub timestamp: u64,
    pub sticky: bool,
    pub kind: GuildNewsKind,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuildNewsError {
    NotInGuild,
    Permissions,
    EntryNotFound,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildNewsState {
    /// Durable insertion order, independent of client filters and sticky sorting.
    pub entries: Vec<GuildNewsEntry>,
    pub server_time: u64,
    pub error: Option<GuildNewsError>,
}

#[cfg(feature = "transport")]
pub(super) fn register_guild_news_protocol(app: &mut App) {
    app.register_protocol_message::<GuildNewsRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GuildNewsState>()
        .add_direction(NetworkDirection::ServerToClient);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guild_news_wire_keeps_stable_identity_real_subject_and_refusal() {
        let request = GuildNewsRequest::SetSticky {
            id: 251,
            sticky: true,
        };
        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"SetSticky":{"id":251,"sticky":true}}"#
        );
        let reply = GuildNewsState {
            entries: vec![GuildNewsEntry {
                id: 251,
                timestamp: 1_792_118_000,
                sticky: true,
                kind: GuildNewsKind::PlayerAchievement {
                    member: "Ada".into(),
                    achievement_id: 14782,
                    name: "Level 50".into(),
                    description: "Reach level 50.".into(),
                },
            }],
            server_time: 1_792_118_010,
            error: None,
        };
        let bytes = bitcode::serialize(&reply).unwrap();
        assert_eq!(
            bitcode::deserialize::<GuildNewsState>(&bytes).unwrap(),
            reply
        );
        let refused = GuildNewsState {
            entries: vec![],
            server_time: reply.server_time,
            error: Some(GuildNewsError::NotInGuild),
        };
        let bytes = bitcode::serialize(&refused).unwrap();
        assert_eq!(
            bitcode::deserialize::<GuildNewsState>(&bytes).unwrap(),
            refused
        );
    }
}
