//! Housing authority, plots, account storage and basic decor edits.
//! Static DB2 membership is never ownership. References: Retail C_Housing,
//! C_HousingNeighborhood, C_HousingCatalog and C_HousingBasicMode documentation.
use bevy::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

use super::ProtocolRegistrationExt;

pub struct HousingChannel;

/// House-local position in client/server Bevy axes, yaw in radians.
/// Basic mode keeps the authored scale and other authored rotation components.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct DecorTransform {
    pub position: [f32; 3],
    pub yaw: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum HousingCommand {
    ListPlots {
        neighborhood_id: u32,
    },
    EnterNeighborhood {
        neighborhood_id: u32,
    },
    /// Requires authoritative presence at this plot's authored cornerstone.
    PurchasePlot {
        plot_id: u32,
    },
    EnterHouse {
        house_id: u64,
    },
    LeaveHouse,
    /// HouseSettingFlags.HouseAccessAnyone; unset is not an inferred Retail default.
    SetHouseAccessAnyone {
        house_id: u64,
        allowed: bool,
    },
    /// Consumes one owned Retail vendor/collect-effect item into account storage.
    RedeemDecor {
        item_guid: u64,
    },
    PlaceDecor {
        house_id: u64,
        decor_id: u32,
        transform: DecorTransform,
    },
    MoveDecor {
        house_id: u64,
        instance_id: u64,
        position: [f32; 3],
    },
    RotateDecor {
        house_id: u64,
        instance_id: u64,
        yaw: f32,
    },
    RemoveDecor {
        house_id: u64,
        instance_id: u64,
    },
    /// Edits are durably committed as accepted; refresh proves saved state.
    ReloadLayout {
        house_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HousingRequest {
    pub request_id: u64,
    pub command: HousingCommand,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HousingPlot {
    pub plot_id: u32,
    pub neighborhood_id: u32,
    /// Literal NeighborhoodPlot.PlotIndex, not an invented display number.
    pub plot_index: u32,
    pub name: String,
    pub cornerstone: [f32; 3],
    pub teleport: [f32; 3],
    /// DB2 Cost, in the copper contract of Retail cornerstone MoneyFrame/GetMoney.
    pub cost: u64,
    pub house_id: Option<u64>,
    pub owner_character_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HousingHouse {
    pub house_id: u64,
    pub neighborhood_id: u32,
    pub plot_id: u32,
    pub owner_character_id: u64,
    pub can_edit: bool,
    pub house_access_anyone: Option<bool>,
    /// No verified first-house component/room grant: do not synthesize geometry.
    pub exterior_grant_available: bool,
    pub interior_grant_available: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct StoredDecor {
    pub decor_id: u32,
    pub quantity: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlacedDecor {
    pub instance_id: u64,
    pub decor_id: u32,
    pub transform: DecorTransform,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HouseLayout {
    pub house_id: u64,
    pub can_edit: bool,
    pub decor: Vec<PlacedDecor>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct HousingState {
    pub build_key: String,
    pub neighborhood_id: Option<u32>,
    pub plots: Vec<HousingPlot>,
    pub owned_houses: Vec<HousingHouse>,
    pub stored: Vec<StoredDecor>,
    pub layout: Option<HouseLayout>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HousingReply {
    pub request_id: u64,
    pub state: HousingState,
    pub error: Option<String>,
}

pub(super) fn register_housing_protocol(app: &mut App) {
    app.register_protocol_message::<HousingRequest>()
        .add_direction(NetworkDirection::ClientToServer);
    app.register_protocol_message::<HousingReply>()
        .add_direction(NetworkDirection::ServerToClient);
    app.add_channel::<HousingChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(Default::default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
}
