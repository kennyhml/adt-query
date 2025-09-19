use adt_query::{
    Contextualize,
    adt::api::object::{self, LockResult},
    query::StatefulQuery,
};
mod common;

#[tokio::test]
async fn lock_is_retained_in_stateful_session() {
    let client = common::setup_test_system_client();

    let endpoint = object::LockBuilder::default()
        .object_uri("/sap/bc/adt/programs/programs/zwegwerf1")
        .access_mode(object::AccessMode::Modify)
        .build()
        .unwrap();

    let ctx = client.create_context();
    let result = endpoint.query(&client, ctx).await.unwrap();

    let handle = match &result {
        object::LockResult::AlreadyLocked(_) => panic!("Object already locked"),
        object::LockResult::ObjectLocked(data) => &data.body().lock_handle,
    };

    let endpoint = object::UnlockBuilder::default()
        .object_uri("/sap/bc/adt/programs/programs/zwegwerf1")
        .lock_handle(handle)
        .build()
        .unwrap();

    endpoint.query(&client, ctx).await.unwrap();
}

#[tokio::test]
async fn object_is_already_locked() {
    let client = common::setup_test_system_client();

    let endpoint = object::LockBuilder::default()
        .object_uri("/sap/bc/adt/programs/programs/zwegwerf1")
        .access_mode(object::AccessMode::Modify)
        .build()
        .unwrap();

    let ctx = client.create_context();
    let result = endpoint.query(&client, ctx).await.unwrap();
    assert!(
        matches!(result, LockResult::ObjectLocked(_)),
        "Could not obtain the initial lock on the resource."
    );

    // Query again, this should cause an `AlreadyLocked` Error.
    let result = endpoint.query(&client, ctx).await.unwrap();
    assert!(
        matches!(result, LockResult::AlreadyLocked(_)),
        "Expected the resource to be locked already."
    );
}

#[tokio::test]
async fn dropping_context_unlocks_objects() {
    let client = common::setup_test_system_client();

    let endpoint = object::LockBuilder::default()
        .object_uri("/sap/bc/adt/programs/programs/zwegwerf1")
        .access_mode(object::AccessMode::Modify)
        .build()
        .unwrap();

    let ctx = client.create_context();
    let result = endpoint.query(&client, ctx).await.unwrap();
    assert!(
        matches!(result, LockResult::ObjectLocked(_)),
        "Could not obtain the initial lock on the resource."
    );

    assert!(client.drop_context(ctx).is_some());

    let ctx = client.create_context();
    let result = endpoint.query(&client, ctx).await.unwrap();
    assert!(matches!(result, LockResult::ObjectLocked(_)));
}
