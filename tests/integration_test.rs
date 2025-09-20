use adt_query::query::StatelessQuery;
use std::sync::Arc;
mod common;

#[tokio::test]
async fn create_and_destroy_security_session() {
    let client = common::setup_test_system_client();

    let endpoint = adt_query::api::core::CoreDiscovery {};

    endpoint.query(&client).await.unwrap();
    assert!(
        client.session_id().await.is_some(),
        "Could not establish a security session."
    );

    client.destroy_session().await.unwrap();
    assert!(
        client.session_id().await.is_none(),
        "Could not destroy the security session."
    );
}

#[tokio::test]
async fn same_session_reused_in_subsequent_requests() {
    let client = common::setup_test_system_client();
    let endpoint = adt_query::api::core::CoreDiscovery {};

    // First request
    endpoint.query(&client).await.unwrap();
    let first_session_id = client.session_id().await;

    endpoint.query(&client).await.unwrap();
    let second_session_id = client.session_id().await;

    assert_eq!(
        first_session_id, second_session_id,
        "Session ID changed across requests!"
    );
    client.destroy_session().await.unwrap();
}

#[tokio::test]
async fn concurrent_logins_create_only_one_session() {
    let client = Arc::new(common::setup_test_system_client());
    let endpoint = Arc::new(adt_query::api::core::CoreDiscovery {});

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
    client.destroy_session().await.unwrap();
}

#[tokio::test]
async fn new_session_created_automatically() {
    let client = common::setup_test_system_client();
    let endpoint = adt_query::api::core::CoreDiscovery {};

    // First request
    endpoint.query(&client).await.unwrap();
    let first_session_id = client.session_id().await;

    client.destroy_session().await.unwrap();

    endpoint.query(&client).await.unwrap();
    let second_session_id = client.session_id().await;

    assert_ne!(
        first_session_id, second_session_id,
        "Session ID did not change across session destruction!"
    );
    client.destroy_session().await.unwrap();
}
