use shared::protocol::{
    MountJournalEntry, MountJournalUpdate, SetMountFavourite, SummonRandomFavouriteMount,
};

fn round_trip<T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug>(
    message: T,
) {
    let bytes = bitcode::serialize(&message).unwrap();
    let decoded: T = bitcode::deserialize(&bytes).unwrap();
    assert_eq!(decoded, message);
}

#[test]
fn mount_journal_preserves_authentic_metadata_and_account_state_on_wire() {
    round_trip(MountJournalUpdate {
        mounts: vec![MountJournalEntry {
            mount_id: 18,
            spell_id: 6648,
            teaching_items: vec![5655],
            name: "Chestnut Mare".into(),
            description: "Sturdy and steady; \"the horse that built Stormwind.\"".into(),
            source_text: r"|cFFFFD200Vendor: |rKatie Stokx|n|cFFFFD200Zone: |rStormwind City|n|cFFFFD200Cost: |r1|TINTERFACE\\MONEYFRAME\\UI-GOLDICON.BLP:0|t".into(),
            source_type: 2,
            icon_file_data_id: 132261,
            display_id: 2405,
            capability_flags: 29,
            journal_types: shared::protocol::mount_journal_types::GROUND,
            flags: 0,
            collected: true,
            favourite: true,
            usable: true,
            active: false,
        }],
        error: None,
    });
    round_trip(SetMountFavourite {
        mount_id: 18,
        favourite: true,
    });
    round_trip(SummonRandomFavouriteMount);
}
