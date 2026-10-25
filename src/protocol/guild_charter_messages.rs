//! Retail registrar and owned Guild Charter flow on the reliable GuildChannel.
//! NPC, account, inventory, signature and guild authority remain server-owned.
#[cfg(feature = "transport")]
use super::ProtocolRegistrationExt;
use bevy::prelude::*;
#[cfg(feature = "transport")]
use lightyear::prelude::NetworkDirection;
use serde::{Deserialize, Serialize};

pub const GUILD_CHARTER_ITEM: u32 = 5863;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GuildCharterRequest {
    Registrar {
        npc: u64,
    },
    Purchase {
        npc: u64,
        guild_name: String,
    },
    Show {
        charter_guid: u64,
    },
    Offer {
        charter_guid: u64,
        character_name: String,
    },
    Sign {
        charter_guid: u64,
    },
    Rename {
        charter_guid: u64,
        guild_name: String,
    },
    TurnIn {
        npc: u64,
        charter_guid: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildRegistrarSnapshot {
    pub npc: u64,
    pub npc_name: String,
    pub charter_cost: u64,
    pub charter_guid: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GuildPetitionSnapshot {
    pub charter_guid: u64,
    pub guild_name: String,
    pub owner_name: String,
    pub signatures: Vec<String>,
    pub required_signatures: u8,
    pub is_owner: bool,
    pub can_sign: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GuildCharterReply {
    Registrar(GuildRegistrarSnapshot),
    Purchased { charter_guid: u64 },
    Petition(GuildPetitionSnapshot),
    Created { guild_id: u32, guild_name: String },
    Failed { message: String },
}

#[cfg(feature = "transport")]
pub(super) fn register_guild_charter_protocol(app: &mut App) {
    app.register_protocol_message::<GuildCharterRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<GuildCharterReply>()
        .add_direction(NetworkDirection::ServerToClient);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guild_charter_wire_keeps_owned_guids_target_names_and_authoritative_signatures() {
        let request = GuildCharterRequest::Offer {
            charter_guid: 701,
            character_name: "Elowen".into(),
        };
        let bytes = bitcode::serialize(&request).unwrap();
        assert_eq!(
            bitcode::deserialize::<GuildCharterRequest>(&bytes).unwrap(),
            request
        );
        let reply = GuildCharterReply::Petition(GuildPetitionSnapshot {
            charter_guid: 701,
            guild_name: "Raid Team".into(),
            owner_name: "Cara".into(),
            signatures: vec!["Elowen".into(), "Alessio".into()],
            required_signatures: 4,
            is_owner: false,
            can_sign: true,
        });
        let bytes = bitcode::serialize(&reply).unwrap();
        assert_eq!(
            bitcode::deserialize::<GuildCharterReply>(&bytes).unwrap(),
            reply
        );
    }
}
