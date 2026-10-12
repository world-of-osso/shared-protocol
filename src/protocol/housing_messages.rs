//! Housing authority, plots, account storage and basic decor edits.
//! Static DB2 membership is never ownership. References: Retail C_Housing,
//! C_HousingNeighborhood, C_HousingCatalog and C_HousingBasicMode documentation.
use bevy::prelude::*;
use lightyear::prelude::{AppChannelExt, ChannelMode, ChannelSettings, NetworkDirection};
use serde::{Deserialize, Serialize};

use super::ProtocolRegistrationExt;

pub struct HousingChannel;

/// House-local Y-up position and Euler rotation in radians, uniform model scale.
/// Basic mode changes position/yaw while preserving pitch/roll/scale.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct DecorTransform {
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub scale: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum HousingCommand {
    ListPlots {
        neighborhood_id: u32,
    },
    EnterNeighborhood {
        neighborhood_id: u32,
    },
    /// Finder ports to the authored plot teleport; admission never purchases it.
    VisitPlot {
        plot_id: u32,
    },
    /// Requires authoritative presence at this plot's authored cornerstone.
    PurchasePlot {
        plot_id: u32,
    },
    EnterHouse {
        house_id: u64,
    },
    LeaveHouse,
    /// Owner saves the HouseAccess flags only; plot/blueprint/owner policy is unchanged.
    SaveHouseAccess {
        house_id: u64,
        access: HouseAccess,
    },
    /// Validate/stage dye selections only; no owned dye or decor is consumed.
    PreviewDecorDyes {
        house_id: u64,
        instance_id: u64,
        changes: Vec<DecorDye>,
    },
    /// Commit this connection's preview against current owned inventory.
    CommitDecorDyes {
        house_id: u64,
        instance_id: u64,
    },
    CancelDecorDyes,
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
    /// Expert mode commits one complete staged transform, not per-frame input.
    TransformDecor {
        house_id: u64,
        instance_id: u64,
        transform: DecorTransform,
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

/// Retail HousingPlotOwnerType: relationship to the requesting character/account.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HousingPlotOwnerType {
    None,
    Stranger,
    Friend,
    SelfOwned,
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
    pub owner_type: HousingPlotOwnerType,
}

/// Literal Retail HouseSettingFlags.HouseAccess* selections. None on a house
/// means not configured, never an inferred default. Relationships are server-owned.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HouseAccess {
    pub anyone: bool,
    pub neighbors: bool,
    pub guild: bool,
    pub friends: bool,
    pub party: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HousingHouse {
    pub house_id: u64,
    pub neighborhood_id: u32,
    pub plot_id: u32,
    pub owner_character_id: u64,
    pub can_edit: bool,
    pub house_access: Option<HouseAccess>,
    /// No verified first-house component/room grant: do not synthesize geometry.
    pub exterior_grant_available: bool,
    pub interior_grant_available: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct StoredDecor {
    pub decor_id: u32,
    pub quantity: u32,
}

/// Literal DecorDyeSlot record identity; None returns this slot to default color.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DecorDye {
    pub slot_id: u32,
    pub color_id: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DecorDyePreview {
    pub house_id: u64,
    pub instance_id: u64,
    pub changes: Vec<DecorDye>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlacedDecor {
    pub instance_id: u64,
    pub decor_id: u32,
    pub transform: DecorTransform,
    /// Committed dyes only. Native recoloring is not yet available.
    pub dyes: Vec<DecorDye>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HouseLayout {
    pub house_id: u64,
    pub can_edit: bool,
    pub decor: Vec<PlacedDecor>,
}

/// Retail NeighborhoodRosterMemberInfo of a public neighborhood, which has no
/// owner/manager ranks or subdivisions.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NeighborhoodResident {
    pub name: String,
    pub online: bool,
    /// Literal NeighborhoodPlot.PlotIndex of the resident's house.
    pub plot_index: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NeighborhoodRoster {
    pub neighborhood_id: u32,
    pub residents: Vec<NeighborhoodResident>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct HousingState {
    pub build_key: String,
    pub neighborhood_id: Option<u32>,
    /// Present only in the reply to an authenticated, in-range UseGameObject.
    /// Plot identity, not a private owner/account ID or an ownership grant.
    pub opened_cornerstone: Option<u32>,
    /// Present only in the reply to an authenticated, in-range use of the
    /// neighborhood's authored bulletin board (PlayerInteractionType 72).
    pub bulletin_board: Option<NeighborhoodRoster>,
    pub plots: Vec<HousingPlot>,
    pub owned_houses: Vec<HousingHouse>,
    pub stored: Vec<StoredDecor>,
    /// The account's placed instances per decor across all its houses; Retail
    /// HousingCatalogEntryInfo.totalNumPlaced. Owned = stored + placed.
    pub placed: Vec<StoredDecor>,
    pub layout: Option<HouseLayout>,
    /// Actor-local transient preview, never saved ownership or an observer update.
    pub dye_preview: Option<DecorDyePreview>,
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
