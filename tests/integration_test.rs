use sapi::endpoint::StatelessQuery;

mod common;

#[tokio::test]
async fn test_system_logon() {
    let client = common::setup();

    let endpoint = sapi::adt::core::discovery::CoreDiscovery {};

    let result = endpoint.query(&client).await;
}
