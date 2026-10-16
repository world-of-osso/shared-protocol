//! Authored reagent slots, not a crafting outcome calculator.
//! Retail 12.1.5.70391: ModifiedCraftingSpellSlot → MCRSlotXMCRCategory →
//! CraftingReagentQuality. Basic=1 is ProfessionConstantsDocumentation.lua203.
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReagentAllocation {
    pub slot_index: u32,
    pub item_id: u32,
    pub quantity: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReagentChoice {
    pub item_id: u32,
    pub currency_id: u32,
    /// None means the Item/CraftingQuality join assigns no tier. Never infer tier from order.
    pub tier: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReagentSlot {
    pub slot_index: u32,
    pub slot_id: u32,
    pub name: String,
    pub reagent_type: u32,
    pub player_condition: u32,
    pub quantity: u32,
    pub recraft_quantity: u32,
    pub choices: Vec<ReagentChoice>,
}

#[derive(bevy::prelude::Resource, Debug, Default)]
pub struct CraftingReagentCatalog {
    pub recipes: BTreeMap<u32, Vec<ReagentSlot>>,
}

impl CraftingReagentCatalog {
    pub fn load(dir: &Path) -> Result<Self, String> {
        let tiers = load_item_tiers(dir)?;
        let choices = load_category_choices(dir, &tiers)?;
        let slots = load_slot_choices(dir, &choices)?;
        let mut catalog = Self::default();
        for row in read_csv::<SpellSlotRow>(dir, "ModifiedCraftingSpellSlot")? {
            let source = slots.get(&row.slot_id).ok_or_else(|| {
                format!("Spell {}: missing reagent slot {}", row.spell, row.slot_id)
            })?;
            let mut slot = source.clone();
            slot.slot_index = row.slot_index;
            slot.quantity = row.quantity;
            slot.recraft_quantity = row.recraft_quantity;
            catalog.recipes.entry(row.spell).or_default().push(slot);
        }
        for slots in catalog.recipes.values_mut() {
            slots.sort_by_key(|slot| slot.slot_index);
        }
        Ok(catalog)
    }

    /// Validate only allocation membership, exact quantities and bag ownership.
    /// No item is consumed; no successful outcome or skill contribution is implied.
    pub fn validate(
        &self,
        spell: u32,
        selected: &[ReagentAllocation],
        owned: &BTreeMap<u32, u32>,
    ) -> Result<(), String> {
        let slots = self
            .recipes
            .get(&spell)
            .ok_or_else(|| format!("No authored reagent slots for spell {spell}"))?;
        let mut totals = BTreeMap::<u32, u64>::new();
        let mut used_items = BTreeMap::<u32, u64>::new();
        let mut seen = BTreeSet::new();
        for allocation in selected {
            let slot = slots
                .iter()
                .find(|slot| slot.slot_index == allocation.slot_index)
                .ok_or("Unknown reagent slot")?;
            validate_choice(slot, allocation)?;
            if !seen.insert((allocation.slot_index, allocation.item_id)) {
                return Err("Duplicate reagent allocation".into());
            }
            *totals.entry(allocation.slot_index).or_default() += u64::from(allocation.quantity);
            *used_items.entry(allocation.item_id).or_default() += u64::from(allocation.quantity);
        }
        validate_totals(slots, &totals)?;
        for (item, needed) in used_items {
            if u64::from(owned.get(&item).copied().unwrap_or(0)) < needed {
                return Err(format!("Insufficient owned reagent {item}"));
            }
        }
        Ok(())
    }
}

fn validate_choice(slot: &ReagentSlot, allocation: &ReagentAllocation) -> Result<(), String> {
    if slot.player_condition != 0 {
        return Err(format!(
            "REAGENT_SLOT_CONDITION: condition {} is not evaluated",
            slot.player_condition
        ));
    }
    if slot.reagent_type != 1 {
        return Err(
            "REAGENT_NON_BASIC_REQUIREMENTS: modifying/finishing eligibility is not evaluated"
                .into(),
        );
    }
    let permitted = slot.choices.iter().any(|choice| {
        choice.item_id == allocation.item_id && choice.currency_id == 0 && choice.item_id != 0
    });
    if !permitted || allocation.quantity == 0 {
        return Err("Reagent does not match slot or quantity is zero".into());
    }
    Ok(())
}

fn validate_totals(slots: &[ReagentSlot], totals: &BTreeMap<u32, u64>) -> Result<(), String> {
    for slot in slots {
        let total = totals.get(&slot.slot_index).copied().unwrap_or(0);
        if slot.reagent_type == 1 && total != u64::from(slot.quantity) {
            return Err(format!(
                "Slot {} requires exactly {} reagents",
                slot.slot_index, slot.quantity
            ));
        }
    }
    Ok(())
}

fn load_item_tiers(dir: &Path) -> Result<BTreeMap<u32, u32>, String> {
    let quality: BTreeMap<_, _> = read_csv::<QualityRow>(dir, "CraftingQuality")?
        .into_iter()
        .map(|row| (row.id, row.tier))
        .collect();
    Ok(read_csv::<ItemRow>(dir, "Item")?
        .into_iter()
        .filter_map(|row| quality.get(&row.quality).map(|&tier| (row.id, tier)))
        .collect())
}

fn load_category_choices(
    dir: &Path,
    tiers: &BTreeMap<u32, u32>,
) -> Result<BTreeMap<u32, Vec<ReagentChoice>>, String> {
    let mut rows = read_csv::<ChoiceRow>(dir, "CraftingReagentQuality")?;
    rows.sort_by_key(|row| (row.category, row.order, row.item, row.currency));
    let mut choices = BTreeMap::<u32, Vec<ReagentChoice>>::new();
    for row in rows {
        choices
            .entry(row.category)
            .or_default()
            .push(ReagentChoice {
                item_id: row.item,
                currency_id: row.currency,
                tier: tiers.get(&row.item).copied(),
            });
    }
    Ok(choices)
}

fn load_slot_choices(
    dir: &Path,
    choices: &BTreeMap<u32, Vec<ReagentChoice>>,
) -> Result<BTreeMap<u32, ReagentSlot>, String> {
    let mut links = read_csv::<SlotCategoryRow>(dir, "MCRSlotXMCRCategory")?;
    links.sort_by_key(|row| (row.slot, row.order));
    let mut by_slot = BTreeMap::<u32, Vec<ReagentChoice>>::new();
    for link in links {
        if let Some(alternatives) = choices.get(&link.category) {
            by_slot
                .entry(link.slot)
                .or_default()
                .extend(alternatives.iter().cloned());
        }
    }
    let mut slots = BTreeMap::new();
    for row in read_csv::<SlotRow>(dir, "ModifiedCraftingReagentSlot")? {
        let alternatives = by_slot.remove(&row.id).unwrap_or_default();
        slots.insert(
            row.id,
            ReagentSlot {
                slot_index: 0,
                slot_id: row.id,
                name: row.name,
                reagent_type: row.kind,
                player_condition: row.condition,
                quantity: 0,
                recraft_quantity: 0,
                choices: alternatives,
            },
        );
    }
    Ok(slots)
}

fn read_csv<T: DeserializeOwned>(dir: &Path, table: &str) -> Result<Vec<T>, String> {
    let path = dir.join(format!("{table}.csv"));
    let mut reader =
        csv::Reader::from_path(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    reader
        .deserialize()
        .collect::<Result<_, _>>()
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[derive(Deserialize)]
struct SpellSlotRow {
    #[serde(rename = "SpellID")]
    spell: u32,
    #[serde(rename = "Slot")]
    slot_index: u32,
    #[serde(rename = "ModifiedCraftingReagentSlotID")]
    slot_id: u32,
    #[serde(rename = "ReagentCount")]
    quantity: u32,
    #[serde(rename = "ReagentReCraftCount")]
    recraft_quantity: u32,
}
#[derive(Deserialize)]
struct SlotRow {
    #[serde(rename = "ID")]
    id: u32,
    #[serde(rename = "Name_lang")]
    name: String,
    #[serde(rename = "ReagentType")]
    kind: u32,
    #[serde(rename = "PlayerConditionID")]
    condition: u32,
}
#[derive(Deserialize)]
struct SlotCategoryRow {
    #[serde(rename = "ModifiedCraftingReagentSlotID")]
    slot: u32,
    #[serde(rename = "ModifiedCraftingCategoryID")]
    category: u32,
    #[serde(rename = "Order")]
    order: u32,
}
#[derive(Deserialize)]
struct ChoiceRow {
    #[serde(rename = "ModifiedCraftingCategoryID")]
    category: u32,
    #[serde(rename = "ItemID")]
    item: u32,
    #[serde(rename = "CurrencyTypesID")]
    currency: u32,
    #[serde(rename = "OrderIndex")]
    order: u32,
}
#[derive(Deserialize)]
struct ItemRow {
    #[serde(rename = "ID")]
    id: u32,
    #[serde(rename = "CraftingQualityID")]
    quality: u32,
}
#[derive(Deserialize)]
struct QualityRow {
    #[serde(rename = "ID")]
    id: u32,
    #[serde(rename = "QualityTier")]
    tier: u32,
}

#[cfg(test)]
#[path = "crafting_reagents_tests.rs"]
mod tests;
