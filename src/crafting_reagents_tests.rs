use super::*;
use std::path::PathBuf;

fn catalog() -> CraftingReagentCatalog {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/crafting-reagents");
    CraftingReagentCatalog::load(&root).unwrap()
}

#[test]
fn leatherworking_hat_slots_keep_authored_tier_choices_and_quantities() {
    let catalog = catalog();
    let slots = &catalog.recipes[&375188];
    assert_eq!(slots.len(), 3);
    assert_eq!(
        (slots[0].slot_index, slots[0].slot_id, slots[0].quantity),
        (0, 149, 2)
    );
    assert_eq!(
        (slots[1].slot_index, slots[1].slot_id, slots[1].quantity),
        (1, 146, 10)
    );
    let leather: Vec<_> = slots[1]
        .choices
        .iter()
        .map(|choice| (choice.item_id, choice.tier))
        .collect();
    assert_eq!(
        leather,
        [(193208, Some(1)), (193210, Some(2)), (193211, Some(3))]
    );
}

#[test]
fn mixed_quality_allocation_requires_owned_allowed_items_and_exact_slot_totals() {
    let catalog = catalog();
    let owned = [(193222, 2), (193208, 6), (193210, 4)].into();
    let selected = vec![
        ReagentAllocation {
            slot_index: 0,
            item_id: 193222,
            quantity: 2,
        },
        ReagentAllocation {
            slot_index: 1,
            item_id: 193208,
            quantity: 6,
        },
        ReagentAllocation {
            slot_index: 1,
            item_id: 193210,
            quantity: 4,
        },
    ];
    assert_eq!(catalog.validate(375188, &selected, &owned), Ok(()));
    let mut invalid = selected.clone();
    invalid[2].quantity = 5;
    assert!(catalog.validate(375188, &invalid, &owned).is_err());
    invalid = selected.clone();
    invalid[2].item_id = 244634; // Authentic Midnight hide, not this recipe's leather.
    assert!(catalog.validate(375188, &invalid, &owned).is_err());
    invalid = selected.clone();
    invalid[1].quantity = 5;
    assert!(catalog.validate(375188, &invalid, &owned).is_err());
    assert!(catalog.validate(375188, &selected[..1], &owned).is_err());
    invalid = selected.clone();
    invalid.push(selected[0].clone());
    assert!(catalog.validate(375188, &invalid, &owned).is_err());
    assert_eq!(
        crate::profession::evaluate_crafted_quality(),
        Err(crate::profession::CraftingOperationGap::CraftingOperationComposition)
    );
}
