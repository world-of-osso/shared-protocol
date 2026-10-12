use shared::protocol::{
    SaveTransmogOutfit, TransmogOutfitError, TransmogOutfitFailed, TransmogState,
};

#[test]
fn transmog_state_roundtrips_authored_sets_outfit_and_illusion_data() {
    // Local Retail12.1 DB2 identities: Gladius821, TransmogIllusion3 (SpellItemEnchantment
    // 1898 Lifestealing, icon136130), OutfitEntry2. These namespaces never substitute.
    let value = serde_json::json!({
        "npc":54442,"slots":[],
        "sets":[{"set_id":13,"parent_set_id":80,"name":"Wild Combatant's Chain Armor","source_ids":[70594,70595],
            "primary_appearances":[
                {"source":{"source_id":70594,"appearance_id":25939,"item_id":124923,"display_id":142240,
                "icon_fdid":1092121,"name":"Wild Combatant's Chain Armor","inventory_type":5},
                "collection":"Collected","slots":["Chest"]},
                {"source":{"source_id":70595,"appearance_id":25940,"item_id":124924,"display_id":142246,
                "icon_fdid":1092122,"name":"Wild Combatant's Chain Gauntlets","inventory_type":10},
                "collection":"Uncollected","slots":["Hands"]}]}],
        "outfit_entries":[{"id":2,"name":"Outfit 1","order_index":2,"cost":0,"source":1,"flags":35,"set_type":1}],
        "outfits":[{"id":2,"name":"Gladius","choices":[{"slot":"MainHand","choice":{"Source":821}}],"illusions":[]}],
        "illusions":[{"id":3,"enchantment_id":1898,"name":"Lifestealing","icon_fdid":136130,
            "cost":10000,"unlock_condition_id":0,"flags":0,"collected":false}],
        "outfit_slots":[{"id":13,"slot":"MainHand","flags":3,"collection_type":0}],
        "outfit_options":[{"id":1,"slot_info_id":13,"name":"One Handed Weapon","option":1,"flags":0}],
        "custom_sets":[{"id":1,"name":"Gladius","choices":[{"slot":"MainHand","choice":{"Source":821}}]}]
    });
    let decoded: TransmogState = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&decoded).unwrap(), value);
    let bytes = bincode::serde::encode_to_vec(&decoded, bincode::config::standard()).unwrap();
    let (wire, consumed): (TransmogState, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(wire, decoded);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn transmog_outfit_save_roundtrips_without_paid_apply_guids() {
    let value = serde_json::json!({"npc":54442,"outfit":{"id":2,"name":"Gladius",
        "choices":[{"slot":"MainHand","choice":{"Source":821}}],
        "illusions":[{"slot":"MainHand","illusion_id":3}]}});
    let save: SaveTransmogOutfit = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&save).unwrap(), value);
    let bytes = bincode::serde::encode_to_vec(&save, bincode::config::standard()).unwrap();
    let (decoded, _): (SaveTransmogOutfit, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(decoded, save);
    let failure = TransmogOutfitFailed {
        npc: 54442,
        error: TransmogOutfitError::EntryUnavailable,
    };
    let bytes = bincode::serde::encode_to_vec(&failure, bincode::config::standard()).unwrap();
    let (decoded, _): (TransmogOutfitFailed, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(decoded, failure);
}

#[test]
fn transmog_set_pieces_distinguish_all_three_collection_states() {
    for state in ["Collected", "Uncollected", "NonCollectable"] {
        let value = serde_json::json!({"npc":54442,"slots":[],"sets":[{"set_id":13,
            "parent_set_id":80,"name":"Wild Combatant's Chain Armor","source_ids":[70594],
            "primary_appearances":[{"source":{"source_id":70594,"appearance_id":25939,
            "item_id":124923,"display_id":142240,"icon_fdid":1092121,
            "name":"Wild Combatant's Chain Armor","inventory_type":5},"collection":state,
            "slots":["Chest"]}]}],"outfit_entries":[],"outfits":[],"illusions":[],"outfit_slots":[],"outfit_options":[],"custom_sets":[]});
        let decoded: TransmogState = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), value);
    }
}
