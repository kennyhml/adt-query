use adt_query::{models::adtcore, query::StatelessQuery, response::CacheControlled};

use adt_query::api;

mod common;

#[tokio::test]
async fn program_data_is_fetched_without_cache() {
    let client = common::setup_test_system_client();

    let endpoint = api::programs::ProgramBuilder::default()
        .name("ZDEMO1")
        .version(adtcore::Version::Active)
        .build()
        .unwrap();

    let result = endpoint.query(&client).await.unwrap();
    assert!(matches!(result, CacheControlled::Modified(_)))
}

/// Warning: This test is not super robust because it relies on the etag being the last modification..
#[tokio::test]
async fn program_data_is_not_refetched_with_etag() {
    let client = common::setup_test_system_client();

    let endpoint = api::programs::ProgramBuilder::default()
        .name("ZDEMO1")
        .version(adtcore::Version::Active)
        .etag("202412141946310018")
        .build()
        .unwrap();

    let result = endpoint.query(&client).await.unwrap();
    assert!(matches!(result, CacheControlled::NotModified(_)))
}

#[tokio::test]
async fn program_source_is_fetched_without_cache() {
    let client = common::setup_test_system_client();

    let endpoint = api::programs::ProgramSourceBuilder::default()
        .name("ZDEMO1")
        .version(adtcore::Version::Active)
        .build()
        .unwrap();

    let result = endpoint.query(&client).await.unwrap();
    assert!(matches!(result, CacheControlled::Modified(_)))
}

/// Warning: This test is not super robust because it relies on the etag being the last modification..
#[tokio::test]
async fn program_source_is_not_refetched_with_etag() {
    let client = common::setup_test_system_client();

    let endpoint = api::programs::ProgramSourceBuilder::default()
        .name("ZDEMO1")
        .version(adtcore::Version::Active)
        .etag("202412141946310011")
        .build()
        .unwrap();

    let result = endpoint.query(&client).await.unwrap();
    assert!(matches!(result, CacheControlled::NotModified(_)))
}

#[tokio::test]
async fn program_versions_are_fetched() {
    let client = common::setup_test_system_client();

    let endpoint = api::programs::ProgramVersionsBuilder::default()
        .name("ZDEMO1")
        .build()
        .unwrap();

    let result = endpoint.query(&client).await.unwrap();
    assert_eq!(result.body().title, "Version List of ZDEMO1 (REPS)");
}

#[tokio::test]
async fn program_versions_and_transports_are_fetched() {
    let client = common::setup_test_system_client();

    let endpoint = api::programs::ProgramVersionsBuilder::default()
        .name("z_badi_check")
        .build()
        .unwrap();

    let result = endpoint.query(&client).await.unwrap();
    assert_ne!(result.body().entries[0].transport.len(), 0);
}
