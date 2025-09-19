use adt_query::query::StatelessQuery;
use std::sync::Arc;
mod common;

#[tokio::test]
async fn initial_system_logon() {
    let client = common::setup_test_system_client();

    let endpoint = adt_query::adt::api::core::CoreDiscovery {};

    endpoint.query(&client).await.unwrap();
    assert!(
        client.session_id().await.is_some(),
        "Client is not logged on."
    );
}

#[tokio::test]
async fn same_session_reused_in_subsequent_requests() {
    let client = common::setup_test_system_client();
    let endpoint = adt_query::adt::api::core::CoreDiscovery {};

    // First request
    endpoint.query(&client).await.unwrap();
    let first_session_id = client.session_id().await;

    endpoint.query(&client).await.unwrap();
    let second_session_id = client.session_id().await;

    assert_eq!(
        first_session_id, second_session_id,
        "Session ID changed across requests!"
    );
}

#[tokio::test]
async fn concurrent_requests_only_create_one_session() {
    let client = Arc::new(common::setup_test_system_client());
    let endpoint = Arc::new(adt_query::adt::api::core::CoreDiscovery {});

    let task1 = {
        let client = client.clone();
        let endpoint = endpoint.clone();
        tokio::spawn(async move {
            endpoint.query(&*client).await.unwrap();
            client.session_id().await
        })
    };

    let task2 = {
        let client = client.clone();
        let endpoint = endpoint.clone();
        tokio::spawn(async move {
            endpoint.query(&*client).await.unwrap();
            client.session_id().await
        })
    };

    match tokio::try_join!(task1, task2) {
        Ok((result1, result2)) => {
            assert_eq!(result1, result2, "Different sessions were created.");
        }
        Err(_) => panic!("Failed to join the tasks"),
    }
}
