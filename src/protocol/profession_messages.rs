//! Retail profession specialization and fixed-output crafting order operations.
//! Numeric quality/concentration evaluation is deliberately not part of this contract.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionTraitPurchase {
    pub node_id: u32,
    pub entry_id: u32,
    pub ranks: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ProfessionSpecializationRequest {
    Query {
        skill_line: u32,
    },
    /// Absolute purchased ranks. Profession knowledge spending is irreversible.
    Commit {
        skill_line: u32,
        tree_id: u32,
        purchases: Vec<ProfessionTraitPurchase>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionSpecCost {
    pub trait_currency_id: u32,
    pub amount: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionSpecPath {
    pub left_group: u32,
    pub right_group: u32,
    pub trait_currency_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionSpecPerk {
    /// Unique ProfTrait path referenced by source Granted conditions; zero if unresolved.
    pub path_node_id: u32,
    pub required_points: u32,
    pub unlocks_subpath: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionSpecNodeSnapshot {
    pub node_id: u32,
    pub entry_id: u32,
    pub name: String,
    pub description: String,
    pub icon: u32,
    pub spell_id: u32,
    pub rank: u32,
    pub granted_rank: u32,
    pub max_rank: u32,
    pub pos_x: i32,
    pub pos_y: i32,
    /// Raw TraitNode.Type, not a class/spec/hero interpretation.
    pub kind: u8,
    pub available: bool,
    pub unavailable_reason: Option<String>,
    pub costs: Vec<ProfessionSpecCost>,
    pub path: Option<ProfessionSpecPath>,
    pub perk: Option<ProfessionSpecPerk>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionSpecEdgeSnapshot {
    pub left_node_id: u32,
    pub right_node_id: u32,
    pub kind: u8,
    pub visual_style: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionSpecCurrencySnapshot {
    pub trait_currency_id: u32,
    /// CurrencyTypes ID for wallet-backed knowledge, zero for derived unlock tokens.
    pub currency_id: u32,
    pub available: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionSpecTreeSnapshot {
    pub tree_id: u32,
    pub name: String,
    pub root_node_id: u32,
    pub unlocked: bool,
    pub nodes: Vec<ProfessionSpecNodeSnapshot>,
    pub edges: Vec<ProfessionSpecEdgeSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionSpecializationsSnapshot {
    pub skill_line: u32,
    /// CurrencyTypes ID, not TraitCurrency ID.
    pub knowledge_currency_id: u32,
    pub knowledge_available: u32,
    pub currencies: Vec<ProfessionSpecCurrencySnapshot>,
    pub trees: Vec<ProfessionSpecTreeSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProfessionSpecializationStateUpdate {
    pub snapshot: Option<ProfessionSpecializationsSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CraftingOrderKind {
    Public,
    Guild,
    Personal,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CraftingOrderStatus {
    Open,
    Claimed,
    Completed,
    Cancelled,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CraftingOrderReagent {
    pub item_id: u32,
    /// Authored requirement for one craft; the server validates this value.
    pub quantity: u32,
    /// Customer contribution; the crafter supplies the remaining quantity.
    pub supplied: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum CraftingOrderRequest {
    Query {
        skill_line: u32,
        kind: CraftingOrderKind,
        mine: bool,
    },
    Place {
        spell_id: u32,
        kind: CraftingOrderKind,
        recipient: Option<String>,
        commission: u64,
        reagents: Vec<CraftingOrderReagent>,
        duration_hours: u8,
        note: String,
    },
    Claim {
        order_id: u64,
    },
    Release {
        order_id: u64,
    },
    /// Starts the authoritative recipe cast; completion delivers through mail.
    Fulfill {
        order_id: u64,
    },
    Cancel {
        order_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CraftingOrderSnapshot {
    pub id: u64,
    pub spell_id: u32,
    pub item_id: u32,
    pub count: u32,
    pub kind: CraftingOrderKind,
    pub customer: String,
    pub recipient: Option<String>,
    pub crafter: Option<String>,
    pub commission: u64,
    pub reagents: Vec<CraftingOrderReagent>,
    pub expires_at: u64,
    pub note: String,
    pub status: CraftingOrderStatus,
    pub can_claim: bool,
    pub can_fulfill: bool,
    pub can_release: bool,
    pub can_cancel: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CraftingOrderRecipeSnapshot {
    pub spell_id: u32,
    pub skill_line: u32,
    pub item_id: u32,
    pub count: u32,
    pub name: String,
    pub reagents: Vec<CraftingOrderReagent>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CraftingOrdersSnapshot {
    pub skill_line: u32,
    pub kind: CraftingOrderKind,
    pub mine: bool,
    pub recipes: Vec<CraftingOrderRecipeSnapshot>,
    pub orders: Vec<CraftingOrderSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CraftingOrderStateUpdate {
    pub snapshot: Option<CraftingOrdersSnapshot>,
    pub message: Option<String>,
    pub error: Option<String>,
}
