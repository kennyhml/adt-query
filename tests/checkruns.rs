use sapi::api::StatelessQuery;

mod common;

#[tokio::test]
async fn the_available_checkrun_reporters_are_retrieved() {
    let client = common::setup_test_system_client();

    let endpoint = sapi::adt::checkruns::Reporters {};
    endpoint.query(&client).await.unwrap();
}

#[tokio::test]
async fn checkrun_reports_warnings() {
    let client = common::setup_test_system_client();

    // make a get request for csrf token first
    let endpoint = sapi::adt::checkruns::Reporters {};
    endpoint.query(&client).await.unwrap();

    let endpoint = sapi::adt::checkruns::RunCheckBuilder::default()
        .object(
            "/sap/bc/adt/functions/groups/http_runtime/fmodules/http_get_handler_list",
            "active",
        )
        .reporter("abapCheckRun")
        .build()
        .unwrap();

    let response = endpoint.query(&client).await.unwrap();
    println!("{:?}", response);
}
