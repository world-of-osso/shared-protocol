//! Authored concentration currency metadata. No initial grant or recharge epoch is inferred.
use crate::currency::CurrencyWallet;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcentrationOriginGap;

impl std::fmt::Display for ConcentrationOriginGap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            "CRAFTING_CONCENTRATION_ORIGIN: initial grant and recharge origin are unsourced",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcentrationCurrency {
    pub id: u32,
    pub name: String,
    pub icon: u32,
    pub cap: u32,
    pub recharge_amount: u32,
    pub recharge_cycle_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcentrationSnapshot {
    pub skill_line: u32,
    pub currency: ConcentrationCurrency,
    /// Absent wallet entries are unknown, not a zero/full initial grant.
    pub quantity: Option<u32>,
    pub recharge_origin: Result<u64, ConcentrationOriginGap>,
}

#[derive(Debug, Default, bevy::prelude::Resource)]
pub struct ConcentrationCatalog {
    by_skill: BTreeMap<u32, ConcentrationCurrency>,
}

impl ConcentrationCatalog {
    pub fn load(dir: &Path) -> Result<Self, String> {
        let currencies: BTreeMap<_, _> = read_csv::<CurrencyRow>(dir, "CurrencyTypes")?
            .into_iter()
            .map(|row| (row.id, row.into_currency()))
            .collect();
        let mut by_skill = BTreeMap::new();
        for row in read_csv::<ExpansionRow>(dir, "ProfessionExpansion")? {
            if row.currency_id == 0 {
                continue;
            }
            let currency = currencies.get(&row.currency_id).ok_or_else(|| {
                format!(
                    "SkillLine {}: missing concentration currency {}",
                    row.skill_line, row.currency_id
                )
            })?;
            by_skill.insert(row.skill_line, currency.clone());
        }
        Ok(Self { by_skill })
    }

    pub fn for_skill(&self, skill: u32) -> Option<&ConcentrationCurrency> {
        self.by_skill.get(&skill)
    }

    pub fn snapshot(&self, skill: u32, wallet: &CurrencyWallet) -> Option<ConcentrationSnapshot> {
        let currency = self.for_skill(skill)?.clone();
        let quantity = wallet
            .entries
            .iter()
            .find(|entry| entry.id == currency.id)
            .map(|entry| entry.amount);
        Some(ConcentrationSnapshot {
            skill_line: skill,
            currency,
            quantity,
            recharge_origin: Err(ConcentrationOriginGap),
        })
    }
}

fn read_csv<T: DeserializeOwned>(dir: &Path, table: &str) -> Result<Vec<T>, String> {
    let path = dir.join(format!("{table}.csv"));
    let mut reader =
        csv::Reader::from_path(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    reader
        .deserialize()
        .collect::<Result<Vec<T>, _>>()
        .map_err(|error| format!("{}: {error}", path.display()))
}

#[derive(Deserialize)]
struct ExpansionRow {
    #[serde(rename = "SkillLineID")]
    skill_line: u32,
    #[serde(rename = "ConcentrationCurrencyID")]
    currency_id: u32,
}

#[derive(Deserialize)]
struct CurrencyRow {
    #[serde(rename = "ID")]
    id: u32,
    #[serde(rename = "Name_lang")]
    name: String,
    #[serde(rename = "InventoryIconFileID")]
    icon: u32,
    #[serde(rename = "MaxQty")]
    cap: u32,
    #[serde(rename = "RechargingAmountPerCycle")]
    recharge_amount: u32,
    #[serde(rename = "RechargingCycleDurationMS")]
    recharge_cycle_ms: u32,
}

impl CurrencyRow {
    fn into_currency(self) -> ConcentrationCurrency {
        ConcentrationCurrency {
            id: self.id,
            name: self.name,
            icon: self.icon,
            cap: self.cap,
            recharge_amount: self.recharge_amount,
            recharge_cycle_ms: self.recharge_cycle_ms,
        }
    }
}
