use sapi::{Session, common::Cookie, endpoint::StatelessQuery};

mod common;

#[tokio::test]
async fn initial_system_logon() {
    let client = common::setup();

    let endpoint = sapi::adt::core::discovery::CoreDiscovery {};

    endpoint.query(&client).await.unwrap();
    assert!(client.is_logged_on(), "Client is not logged on.");
}

#[tokio::test]
async fn same_session_reused_in_subsequent_requests() {
    let client = common::setup();
    let endpoint = sapi::adt::core::discovery::CoreDiscovery {};

    // First request
    endpoint.query(&client).await.unwrap();
    let first_session_id = client
        .cookies()
        .load()
        .find(Cookie::SAP_SESSIONID)
        .expect("Missing SAP_SESSIONID after first request")
        .value()
        .to_string();

    endpoint.query(&client).await.unwrap();
    let second_session_id = client
        .cookies()
        .load()
        .find(Cookie::SAP_SESSIONID)
        .expect("Missing SAP_SESSIONID after second request")
        .value()
        .to_string();

    assert_eq!(
        first_session_id, second_session_id,
        "Session ID changed across requests!"
    );
}
