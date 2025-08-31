use sapi::{
    adt::models::checkrun::{ObjectBuilder, ObjectListBuilder},
    api::{CacheControlled, StatelessQuery},
};

use sapi::adt::api;

mod common;

#[tokio::test]
async fn program_data_is_fetched_without_cache() {
    let client = common::setup_test_system_client();

    let endpoint = api::programs::ProgramBuilder::default()
        .name("ZDEMO1")
        .version("active")
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
        .version("active")
        .etag("202412141946310018")
        .build()
        .unwrap();

    let result = endpoint.query(&client).await.unwrap();
    assert!(matches!(result, CacheControlled::NotModified(_)))
}

#[tokio::test]
async fn checkrun_reports_warnings() {
    let client = common::setup_test_system_client();

    // make a get request for csrf token first
    let endpoint = sapi::adt::api::checkruns::Reporters {};
    endpoint.query(&client).await.unwrap();

    let object = ObjectBuilder::default()
        .object_uri("/sap/bc/adt/functions/groups/http_runtime/fmodules/http_get_handler_list")
        .version("active")
        .build()
        .unwrap();

    let endpoint = sapi::adt::api::checkruns::RunCheckBuilder::default()
        .objects(ObjectListBuilder::default().object(object).build().unwrap())
        .reporter("abapCheckRun")
        .build()
        .unwrap();

    endpoint.query(&client).await.unwrap();
}
