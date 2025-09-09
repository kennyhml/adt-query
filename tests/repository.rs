use sapi::api::StatelessQuery;

use sapi::adt::api;

mod common;

#[tokio::test]
async fn available_facets_are_retrieved() {
    let client = common::setup_test_system_client();

    let endpoint = api::repository::AvailableFacets::default();
    let result = endpoint.query(&client).await.unwrap();
    assert!(
        result.body().facets.len() > 5,
        "At least 5 Facets should be retrieved"
    )
}
