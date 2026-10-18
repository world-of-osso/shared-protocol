//! Serialized enum tags are an external contract: both features append, never reorder.
use std::fmt::Debug;

use serde::{Serialize, de::DeserializeOwned};
use shared::protocol::{GuildRankRequest, InventoryErrorReason};

fn assert_variant<T: Serialize + DeserializeOwned + PartialEq + Debug>(tag: u8, value: &T) {
    let bytes = bincode::serde::encode_to_vec(value, bincode::config::standard()).unwrap();
    assert_eq!(bytes[0], tag, "serialized variant tag for {value:?}");
    let (decoded, used): (T, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(&decoded, value);
    assert_eq!(used, bytes.len());
    let json = serde_json::to_value(value).unwrap();
    let decoded: T = serde_json::from_value(json).unwrap();
    assert_eq!(&decoded, value);
}

#[test]
fn max_count_guild_inventory_discriminants_and_codec() {
    use InventoryErrorReason::*;
    let cases = [
        CantEquipLevelI { level: 10 },
        WrongSlot,
        BagFull,
        InvFull,
        CantStack,
        CantDualWield,
        NotEquippable,
        SlotEmpty,
        ItemNotFound,
        TooFewToSplit,
        SplitFailed,
        CantDoRightNow,
        InternalBagError,
        CantEquipEver,
        NotInCombat,
        BagInBag,
        WrongBagType,
        CantSwap,
        DestroyNonemptyBag,
        ItemUniqueEquippable,
        ItemMaxLimitCategoryEquippedExceeded { quantity: 2 },
    ];
    for (tag, value) in cases.iter().enumerate() {
        assert_variant(tag as u8, value);
    }
    assert_variant(21, &ItemMaxCount);
}

#[test]
fn max_count_guild_rank_discriminants_and_codec() {
    use GuildRankRequest::*;
    let cases = [
        Query,
        SetMemberNote {
            character_name: "Cara".into(),
            note: "Officer note".into(),
            officer: true,
        },
        Leave,
        RemoveMember {
            character_name: "Cara".into(),
        },
        Add {
            name: "Veteran".into(),
        },
        Remove { rank: 3 },
        Rename {
            rank: 3,
            name: "Veteran".into(),
        },
        Move { rank: 3, up: true },
        SetPermissions {
            rank: 3,
            rights: 0x1000,
            gold_per_day: 1234,
        },
        SetTab {
            rank: 3,
            tab: 1,
            view: true,
            deposit: false,
            withdrawals_per_day: 2,
        },
        Promote {
            character_name: "Cara".into(),
        },
        Demote {
            character_name: "Cara".into(),
        },
        QueryEventLog,
    ];
    for (tag, value) in cases.iter().enumerate() {
        assert_variant(tag as u8, value);
    }
    assert_variant(
        13,
        &SetMemberRank {
            character_name: "Cara".into(),
            rank: 4,
        },
    );
}
