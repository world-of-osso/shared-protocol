use shared::protocol::ItemStack;

#[test]
fn crafting_origin_item_wire_preserves_ability_and_recipe_identity() {
    let origin = serde_json::json!({
        "skill_line_ability_id": 1355,
        "recipe_spell_id": 2153,
        "crafting_quality_id": null
    });
    let item = serde_json::json!({
        "item_guid": 9001,
        "item_id": 2303,
        "definition_source": "Retail",
        "count": 1,
        "durability": null,
        "soulbound": false,
        "crafting_origin": origin
    });
    let decoded: ItemStack = serde_json::from_value(item).unwrap();
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&decoded, config).unwrap();
    let (wire, read): (ItemStack, usize) =
        bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(read, bytes.len());
    assert_eq!(wire, decoded);
    let encoded = serde_json::to_value(decoded).unwrap();
    assert_eq!(
        encoded.get("crafting_origin"),
        Some(&origin),
        "owner item wire must preserve actual craft origin, not confuse ability1355 with spell2153"
    );
    assert!(encoded["crafting_origin"]["crafting_quality_id"].is_null());
}
