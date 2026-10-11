//! Currency system: honor, conquest, justice/valor points, and other tokens.
//!
//! Static definitions live in `ALL_CURRENCIES` / `currency_by_id`.
//! Per-character state is tracked in `CurrencyWallet`.
//!
//! PvP identities/caps: local Retail 12.1.0.69933 CurrencyTypes1095531.
//! Conquest's seasonal cap uses WorldState31270; no static weekly cap is authored.

pub const HONOR_CURRENCY_ID: u32 = 1792;
pub const CONQUEST_CURRENCY_ID: u32 = 1602;

use serde::{Deserialize, Serialize};

// --- Static definitions ---

/// A currency definition (static data, like an item template).
pub struct CurrencyDef {
    pub id: u32,
    pub name: &'static str,
    /// Max amount a character can hold. 0 = no cap.
    pub cap: u32,
    /// Weekly earn cap. 0 = no weekly cap.
    pub weekly_cap: u32,
    /// Whether this currency is account-wide (shared across characters).
    pub account_wide: bool,
}

/// All known currency definitions.
pub const ALL_CURRENCIES: &[CurrencyDef] = &[
    CurrencyDef {
        id: HONOR_CURRENCY_ID,
        name: "Honor",
        cap: 15_000,
        weekly_cap: 0,
        account_wide: false,
    },
    CurrencyDef {
        id: CONQUEST_CURRENCY_ID,
        name: "Conquest",
        cap: 0,
        weekly_cap: 0,
        account_wide: false,
    },
    CurrencyDef {
        id: 3,
        name: "Justice Points",
        cap: 4_000,
        weekly_cap: 0,
        account_wide: false,
    },
    CurrencyDef {
        id: 4,
        name: "Valor Points",
        cap: 4_000,
        weekly_cap: 1_000,
        account_wide: false,
    },
    CurrencyDef {
        id: 5,
        name: "Champion's Seal",
        cap: 0,
        weekly_cap: 0,
        account_wide: false,
    },
    CurrencyDef {
        id: 6,
        name: "Epicurean's Award",
        cap: 0,
        weekly_cap: 0,
        account_wide: false,
    },
    CurrencyDef {
        id: 7,
        name: "Timewarped Badge",
        cap: 0,
        weekly_cap: 0,
        account_wide: true,
    },
];

/// Look up a currency definition by ID.
pub fn currency_by_id(id: u32) -> Option<&'static CurrencyDef> {
    ALL_CURRENCIES.iter().find(|c| c.id == id)
}

// --- Error type ---

/// Why a currency operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrencyError {
    /// Unknown currency ID.
    UnknownCurrency,
    /// Not enough of this currency.
    Insufficient,
}

// --- Per-character wallet ---

/// A single currency entry for a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyEntry {
    pub id: u32,
    pub amount: u32,
    /// Amount earned this week (for weekly-capped currencies).
    pub week_earned: u32,
}

/// A player's currency wallet.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CurrencyWallet {
    pub entries: Vec<CurrencyEntry>,
}

impl CurrencyWallet {
    /// Current amount of a currency (0 if not present).
    pub fn get(&self, id: u32) -> u32 {
        self.entries
            .iter()
            .find(|e| e.id == id)
            .map_or(0, |e| e.amount)
    }

    /// Amount earned this week for a currency (0 if not present).
    pub fn week_earned(&self, id: u32) -> u32 {
        self.entries
            .iter()
            .find(|e| e.id == id)
            .map_or(0, |e| e.week_earned)
    }

    /// Add currency, respecting cap and weekly cap.
    ///
    /// Returns the amount actually added (may be less if capped).
    /// Returns `Err(CurrencyError::UnknownCurrency)` for unknown IDs.
    pub fn add(&mut self, id: u32, amount: u32, def: &CurrencyDef) -> Result<u32, CurrencyError> {
        if def.id != id {
            return Err(CurrencyError::UnknownCurrency);
        }

        let entry = self.entry_mut(id);
        let mut actual = amount;

        // Clamp to weekly cap first.
        if def.weekly_cap > 0 {
            let remaining_weekly = def.weekly_cap.saturating_sub(entry.week_earned);
            actual = actual.min(remaining_weekly);
        }

        // Clamp to total cap.
        if def.cap > 0 {
            let remaining_cap = def.cap.saturating_sub(entry.amount);
            actual = actual.min(remaining_cap);
        }

        entry.amount += actual;
        entry.week_earned += actual;
        Ok(actual)
    }

    /// Remove currency.
    ///
    /// Returns `Err(CurrencyError::Insufficient)` if amount > current.
    pub fn remove(&mut self, id: u32, amount: u32) -> Result<(), CurrencyError> {
        if amount == 0 {
            return Ok(());
        }
        let entry = self.entry_mut(id);
        if entry.amount < amount {
            return Err(CurrencyError::Insufficient);
        }
        entry.amount -= amount;
        Ok(())
    }

    /// Check if the player has at least `amount` of a currency.
    pub fn has(&self, id: u32, amount: u32) -> bool {
        self.get(id) >= amount
    }

    /// Reset all `week_earned` counters to 0 (called on weekly reset).
    pub fn reset_weekly(&mut self) {
        for entry in &mut self.entries {
            entry.week_earned = 0;
        }
    }

    /// Get or create the entry for a currency ID.
    fn entry_mut(&mut self, id: u32) -> &mut CurrencyEntry {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            return &mut self.entries[pos];
        }
        self.entries.push(CurrencyEntry {
            id,
            amount: 0,
            week_earned: 0,
        });
        self.entries.last_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn honor() -> &'static CurrencyDef {
        currency_by_id(HONOR_CURRENCY_ID).unwrap()
    }
    fn conquest() -> &'static CurrencyDef {
        currency_by_id(CONQUEST_CURRENCY_ID).unwrap()
    }
    fn justice() -> &'static CurrencyDef {
        currency_by_id(3).unwrap()
    }
    fn valor() -> &'static CurrencyDef {
        currency_by_id(4).unwrap()
    }
    fn champion_seal() -> &'static CurrencyDef {
        currency_by_id(5).unwrap()
    }

    #[test]
    fn retail_pvp_currency_ids_and_source_caps() {
        let honor = currency_by_id(1792).expect("local CurrencyTypes Honor1792");
        let conquest = currency_by_id(1602).expect("local CurrencyTypes Conquest1602");
        assert_eq!(honor.cap, 15_000);
        assert_eq!(conquest.weekly_cap, 0);
        assert!(currency_by_id(1).is_none());
        assert!(currency_by_id(2).is_none());
    }

    #[test]
    fn get_unknown_returns_zero() {
        let wallet = CurrencyWallet::default();
        assert_eq!(wallet.get(999), 0);
    }

    #[test]
    fn add_basic() {
        let mut wallet = CurrencyWallet::default();
        let added = wallet.add(HONOR_CURRENCY_ID, 500, honor()).unwrap();
        assert_eq!(added, 500);
        assert_eq!(wallet.get(HONOR_CURRENCY_ID), 500);
    }

    #[test]
    fn add_respects_cap() {
        let mut wallet = CurrencyWallet::default();
        // Add 14_000, then try to add 2_000 more (cap is 15_000).
        wallet.add(HONOR_CURRENCY_ID, 14_000, honor()).unwrap();
        let added = wallet.add(HONOR_CURRENCY_ID, 2_000, honor()).unwrap();
        assert_eq!(added, 1_000);
        assert_eq!(wallet.get(HONOR_CURRENCY_ID), 15_000);
    }

    #[test]
    fn add_respects_weekly_cap() {
        let mut wallet = CurrencyWallet::default();
        // Valor has weekly_cap = 1000.
        let added = wallet.add(4, 1700, valor()).unwrap();
        assert_eq!(added, 1000);
        assert_eq!(wallet.get(4), 1000);
        assert_eq!(wallet.week_earned(4), 1000);
    }

    #[test]
    fn add_both_caps_takes_minimum() {
        // Valor: cap 4000, weekly_cap 1000. Weekly wins when both apply
        // and we try to add more than both would allow.
        let mut wallet = CurrencyWallet::default();
        let added = wallet.add(4, 2_000, valor()).unwrap();
        assert_eq!(added, 1_000); // weekly cap of 1000 is the binding constraint
        assert_eq!(wallet.get(4), 1_000);
        assert_eq!(wallet.week_earned(4), 1_000);
    }

    #[test]
    fn add_weekly_cap_partially_spent() {
        let mut wallet = CurrencyWallet::default();
        wallet.add(4, 750, valor()).unwrap();
        let added = wallet.add(4, 400, valor()).unwrap();
        assert_eq!(added, 250); // 1000 - 750 = 250 remaining weekly
        assert_eq!(wallet.get(4), 1000);
        assert_eq!(wallet.week_earned(4), 1000);
    }

    #[test]
    fn add_no_cap_currency() {
        let mut wallet = CurrencyWallet::default();
        let added = wallet.add(5, 9_999_999, champion_seal()).unwrap();
        assert_eq!(added, 9_999_999);
        assert_eq!(wallet.get(5), 9_999_999);
    }

    #[test]
    fn add_accumulates() {
        let mut wallet = CurrencyWallet::default();
        wallet.add(3, 500, justice()).unwrap();
        wallet.add(3, 300, justice()).unwrap();
        assert_eq!(wallet.get(3), 800);
    }

    #[test]
    fn remove_basic() {
        let mut wallet = CurrencyWallet::default();
        wallet.add(HONOR_CURRENCY_ID, 1_000, honor()).unwrap();
        wallet.remove(HONOR_CURRENCY_ID, 400).unwrap();
        assert_eq!(wallet.get(HONOR_CURRENCY_ID), 600);
    }

    #[test]
    fn remove_insufficient() {
        let mut wallet = CurrencyWallet::default();
        wallet.add(HONOR_CURRENCY_ID, 100, honor()).unwrap();
        let err = wallet.remove(HONOR_CURRENCY_ID, 200);
        assert_eq!(err, Err(CurrencyError::Insufficient));
    }

    #[test]
    fn remove_zero_ok() {
        let mut wallet = CurrencyWallet::default();
        assert!(wallet.remove(HONOR_CURRENCY_ID, 0).is_ok());
        assert!(wallet.remove(999, 0).is_ok());
    }

    #[test]
    fn has_check() {
        let mut wallet = CurrencyWallet::default();
        wallet.add(HONOR_CURRENCY_ID, 500, honor()).unwrap();
        assert!(wallet.has(HONOR_CURRENCY_ID, 500));
        assert!(wallet.has(HONOR_CURRENCY_ID, 499));
        assert!(!wallet.has(HONOR_CURRENCY_ID, 501));
    }

    #[test]
    fn has_zero_always_true() {
        let wallet = CurrencyWallet::default();
        assert!(wallet.has(999, 0));
        assert!(wallet.has(HONOR_CURRENCY_ID, 0));
    }

    #[test]
    fn reset_weekly() {
        let mut wallet = CurrencyWallet::default();
        wallet.add(CONQUEST_CURRENCY_ID, 400, conquest()).unwrap();
        assert_eq!(wallet.week_earned(CONQUEST_CURRENCY_ID), 400);
        wallet.reset_weekly();
        assert_eq!(wallet.week_earned(CONQUEST_CURRENCY_ID), 0);
        assert_eq!(wallet.get(CONQUEST_CURRENCY_ID), 400); // amount preserved
    }

    #[test]
    fn reset_weekly_empty_wallet() {
        let mut wallet = CurrencyWallet::default();
        wallet.reset_weekly(); // must not panic
        assert_eq!(wallet.entries.len(), 0);
    }

    #[test]
    fn currency_by_id_found() {
        let def = currency_by_id(HONOR_CURRENCY_ID).unwrap();
        assert_eq!(def.id, HONOR_CURRENCY_ID);
        assert_eq!(def.name, "Honor");
        assert_eq!(def.cap, 15_000);
    }

    #[test]
    fn currency_by_id_not_found() {
        assert!(currency_by_id(999).is_none());
    }

    #[test]
    fn add_unknown_currency_errors() {
        let mut wallet = CurrencyWallet::default();
        // Pass def for id=1 but request id=999 — mismatch triggers UnknownCurrency.
        let err = wallet.add(999, 100, honor());
        assert_eq!(err, Err(CurrencyError::UnknownCurrency));
    }

    #[test]
    fn remove_unknown_currency_with_zero() {
        let mut wallet = CurrencyWallet::default();
        // Removing 0 of an unknown currency is a no-op, not an error.
        assert!(wallet.remove(999, 0).is_ok());
    }

    #[test]
    fn add_to_cap_then_add_more_returns_zero() {
        let mut wallet = CurrencyWallet::default();
        wallet.add(HONOR_CURRENCY_ID, 15_000, honor()).unwrap();
        let added = wallet.add(HONOR_CURRENCY_ID, 1, honor()).unwrap();
        assert_eq!(added, 0);
        assert_eq!(wallet.get(HONOR_CURRENCY_ID), 15_000);
    }

    #[test]
    fn weekly_cap_independent_of_total_cap() {
        // Valor's total cap4000 is separate from its weekly cap1000.
        let mut wallet = CurrencyWallet::default();
        wallet.add(4, 1000, valor()).unwrap();
        wallet.reset_weekly();
        let added = wallet.add(4, 1000, valor()).unwrap();
        assert_eq!(added, 1000);
        assert_eq!(wallet.get(4), 2000);
    }

    #[test]
    fn serde_round_trip() {
        let mut wallet = CurrencyWallet::default();
        wallet.add(HONOR_CURRENCY_ID, 1_000, honor()).unwrap();
        wallet.add(4, 750, valor()).unwrap();
        let json = serde_json::to_string(&wallet).unwrap();
        let decoded: CurrencyWallet = serde_json::from_str(&json).unwrap();
        assert_eq!(wallet, decoded);
    }

    #[test]
    fn account_wide_flag() {
        let def = currency_by_id(7).unwrap();
        assert_eq!(def.name, "Timewarped Badge");
        assert!(def.account_wide);
        // All others should not be account-wide.
        for id in 1..=6 {
            assert!(
                !currency_by_id(id).unwrap().account_wide,
                "id {id} should not be account_wide"
            );
        }
    }

    #[test]
    fn multiple_currencies_independent() {
        let mut wallet = CurrencyWallet::default();
        wallet.add(HONOR_CURRENCY_ID, 1_000, honor()).unwrap();
        wallet.add(CONQUEST_CURRENCY_ID, 200, conquest()).unwrap();
        assert_eq!(wallet.get(HONOR_CURRENCY_ID), 1_000);
        assert_eq!(wallet.get(CONQUEST_CURRENCY_ID), 200);
        // Adding honor doesn't touch conquest.
        wallet.add(HONOR_CURRENCY_ID, 500, honor()).unwrap();
        assert_eq!(wallet.get(CONQUEST_CURRENCY_ID), 200);
    }
}
