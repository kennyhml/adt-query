use std::sync::Arc;

use sapi::{
    Contextualize, Cookie, Session,
    api::{StatefulQuery, StatelessQuery},
    error::QueryError,
};

mod common;

#[tokio::test]
async fn initial_system_logon() {
    let client = common::setup_test_system_client();

    let endpoint = sapi::adt::core::discovery::CoreDiscovery {};

    endpoint.query(&client).await.unwrap();
    assert!(client.is_logged_on().await, "Client is not logged on.");
}

#[tokio::test]
async fn unauthorized_system_logon() {
    let client = common::setup_unauthorized_client();

    let endpoint = sapi::adt::core::discovery::CoreDiscovery {};

    let result = endpoint.query(&client).await;
    assert!(matches!(result, Err(QueryError::Unauthorized)));
}

#[tokio::test]
async fn same_session_reused_in_subsequent_requests() {
    let client = common::setup_test_system_client();
    let endpoint = sapi::adt::core::discovery::CoreDiscovery {};

    // First request
    endpoint.query(&client).await.unwrap();
    let first_session_id = client
        .cookies()
        .lock()
        .await
        .find(Cookie::SAP_SESSIONID)
        .expect("Missing SAP_SESSIONID after first request")
        .value()
        .to_string();

    endpoint.query(&client).await.unwrap();
    let second_session_id = client
        .cookies()
        .lock()
        .await
        .find(Cookie::SAP_SESSIONID)
        .expect("Missing SAP_SESSIONID after second request")
        .value()
        .to_string();

    assert_eq!(
        first_session_id, second_session_id,
        "Session ID changed across requests!"
    );
}

#[tokio::test]
async fn concurrent_requests_only_create_one_session() {
    let client = Arc::new(common::setup_test_system_client());
    let endpoint = Arc::new(sapi::adt::core::discovery::CoreDiscovery {});

    let task1 = {
        let client = client.clone();
        let endpoint = endpoint.clone();
        tokio::spawn(async move {
            endpoint.query(&*client).await.unwrap();
            client
                .cookies()
                .lock()
                .await
                .find(Cookie::SAP_SESSIONID)
                .expect("Missing SAP_SESSIONID after first request")
                .value()
                .to_string()
        })
    };

    let task2 = {
        let client = client.clone();
        let endpoint = endpoint.clone();
        tokio::spawn(async move {
            endpoint.query(&*client).await.unwrap();
            client
                .cookies()
                .lock()
                .await
                .find(Cookie::SAP_SESSIONID)
                .expect("Missing SAP_SESSIONID after first request")
                .value()
                .to_string()
        })
    };

    match tokio::try_join!(task1, task2) {
        Ok((result1, result2)) => {
            assert_eq!(result1, result2, "Different sessions were created.");
        }
        Err(_) => panic!("Failed to join the tasks"),
    }
}

#[tokio::test]
async fn request_context_gets_injected() {
    let client = common::setup_test_system_client();

    let endpoint = sapi::adt::core::discovery::CoreDiscoveryStateful {};

    let context = client.reserve_context();
    let response = endpoint.query(&client, context).await.unwrap();

    let set_cookies = response.headers().get_all("set-cookie");

    assert!(
        set_cookies
            .iter()
            .find(|h| h.to_str().unwrap().contains("sap-contextid"))
            .is_some(),
        "No header 'set-cookie'  containing 'sap-contextid'"
    );
}
