use shared::protocol::AuctionSearchQuery;

#[test]
fn auction_query_preserves_exact_item_and_category_filters_on_wire() {
    let input = serde_json::json!({
        "text": "leather", "page": 1, "page_size": 50,
        "min_level": null, "max_level": null, "quality": null,
        "usable_only": false, "sort_field": "Buyout", "sort_dir": "Asc",
        "faction": 0, "item_id": 2318, "class_id": 7
    });
    let query: AuctionSearchQuery = serde_json::from_value(input.clone()).unwrap();
    assert_eq!(serde_json::to_value(&query).unwrap(), input);
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&query, config).unwrap();
    let (decoded, used): (AuctionSearchQuery, usize) =
        bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(decoded, query);
    assert_eq!(used, bytes.len());
}
