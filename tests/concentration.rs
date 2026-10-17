use shared::concentration::{ConcentrationCatalog, ConcentrationOriginGap};
use shared::currency::CurrencyWallet;
use std::path::Path;

#[test]
fn authored_leatherworking_currency_preserves_cap_and_millisecond_recharge() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/concentration");
    let catalog = ConcentrationCatalog::load(&dir).unwrap();
    let currency = catalog.for_skill(2830).unwrap();
    assert_eq!(currency.id, 3049);
    assert_eq!(currency.name, "Leatherworking Concentration");
    assert_eq!(currency.icon, 5747318);
    assert_eq!(currency.cap, 1000);
    assert_eq!(currency.recharge_amount, 1);
    assert_eq!(currency.recharge_cycle_ms, 345600);
    assert!(catalog.for_skill(2540).is_none());
}

#[test]
fn missing_wallet_entry_is_unknown_not_an_initial_zero_or_full_grant() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/concentration");
    let catalog = ConcentrationCatalog::load(&dir).unwrap();
    let wallet = CurrencyWallet::default();
    let snapshot = catalog.snapshot(2830, &wallet).unwrap();
    assert_eq!(snapshot.quantity, None);
    assert_eq!(snapshot.recharge_origin, Err(ConcentrationOriginGap));
    assert!(
        snapshot
            .recharge_origin
            .unwrap_err()
            .to_string()
            .starts_with("CRAFTING_CONCENTRATION_ORIGIN:")
    );
    assert!(wallet.entries.is_empty());
}

#[test]
fn existing_quantity_is_read_only_and_does_not_establish_recharge_origin() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/concentration");
    let catalog = ConcentrationCatalog::load(&dir).unwrap();
    let wallet = CurrencyWallet {
        entries: vec![shared::currency::CurrencyEntry {
            id: 3049,
            amount: 137,
            week_earned: 0,
        }],
    };
    let snapshot = catalog.snapshot(2830, &wallet).unwrap();
    assert_eq!(snapshot.quantity, Some(137));
    assert_eq!(snapshot.recharge_origin, Err(ConcentrationOriginGap));
    assert_eq!(wallet.get(3049), 137);
}
