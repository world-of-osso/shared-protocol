use shared::protocol::{
    AuctionBrowseItem, AuctionBrowseResults, AuctionItemFilter, AuctionSearchQuery,
    AuctionSortField, QueryAuctionBrowse, QueryAuctions,
};

#[test]
fn grouped_browse_preserves_global_prices_quantities_and_item_pagination() {
    let query = AuctionSearchQuery {
        page: 1,
        page_size: 50,
        class_id: Some(7),
        ..Default::default()
    };
    let request = QueryAuctionBrowse {
        query: query.clone(),
    };
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&request, config).unwrap();
    let (decoded, used): (QueryAuctionBrowse, usize) =
        bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(decoded, request);
    assert_eq!(used, bytes.len());
    let response = AuctionBrowseResults {
        query,
        total_results: 75,
        items: vec![AuctionBrowseItem {
            item_id: 2589,
            definition_source: shared::item_data::ItemDefinitionSource::Retail,
            name: "Linen Cloth".into(),
            quality: 1,
            required_level: 1,
            lowest_unit_price: 12,
            total_quantity: 600_000,
        }],
    };
    let bytes = bincode::serde::encode_to_vec(&response, config).unwrap();
    let (decoded, used): (AuctionBrowseResults, usize) =
        bincode::serde::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(decoded, response);
    assert_eq!(used, bytes.len());
}

#[test]
fn auction_query_preserves_exact_item_and_category_filters_on_wire() {
    let input = serde_json::json!({
        "text": "leather", "page": 1, "page_size": 50,
        "min_level": null, "max_level": null, "quality": null,
        "usable_only": false, "sort_field": "Buyout", "sort_dir": "Asc",
        "faction": 0, "item_id": 2318, "class_id": 7, "subcategory_filters": []
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

#[test]
fn subcategory_and_bid_quantity_sorts_round_trip_in_existing_queries() {
    for sort_field in [AuctionSortField::Bid, AuctionSortField::Quantity] {
        let query = AuctionSearchQuery {
            sort_field,
            class_id: Some(4),
            subcategory_filters: vec![
                AuctionItemFilter {
                    class_id: 4,
                    subclass_id: Some(1),
                    inventory_type: Some(5),
                },
                AuctionItemFilter {
                    class_id: 4,
                    subclass_id: Some(1),
                    inventory_type: Some(20),
                },
            ],
            page: 1,
            ..Default::default()
        };
        let browse = QueryAuctionBrowse {
            query: query.clone(),
        };
        let items = QueryAuctions { query };
        let config = bincode::config::standard();
        let bytes = bincode::serde::encode_to_vec(&browse, config).unwrap();
        let (decoded, used): (QueryAuctionBrowse, usize) =
            bincode::serde::decode_from_slice(&bytes, config).unwrap();
        assert_eq!(decoded, browse);
        assert_eq!(used, bytes.len());
        let bytes = bincode::serde::encode_to_vec(&items, config).unwrap();
        let (decoded, used): (QueryAuctions, usize) =
            bincode::serde::decode_from_slice(&bytes, config).unwrap();
        assert_eq!(decoded, items);
        assert_eq!(used, bytes.len());
        assert_eq!(
            serde_json::from_value::<QueryAuctions>(serde_json::to_value(&items).unwrap()).unwrap(),
            items
        );
    }
}
