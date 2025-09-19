use adt_query::{
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

    let result = endpoint.query(&client, ctx).await.unwrap();
    assert_eq!(result.status(), 200)
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
    let lock_handle = match endpoint.query(&client, ctx).await.unwrap() {
        LockResult::AlreadyLocked(_) => panic!("Could not obtain the inital lock."),
        LockResult::ObjectLocked(data) => data.body().lock_handle.to_owned(),
    };

    // Query again, this should cause an `AlreadyLocked` Error.
    let result = endpoint.query(&client, ctx).await.unwrap();
    assert!(
        matches!(result, LockResult::AlreadyLocked(_)),
        "Expected the resource to be locked already."
    );

    // Unlock
    let endpoint = object::UnlockBuilder::default()
        .object_uri("/sap/bc/adt/programs/programs/zwegwerf1")
        .lock_handle(lock_handle)
        .build()
        .unwrap();

    let result = endpoint.query(&client, ctx).await.unwrap();
    assert_eq!(result.status(), 200)
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

    assert_eq!(client.drop_context(ctx).await.unwrap(), true);

    // let ctx = client.create_context();
    // let result = endpoint.query(&client, ctx).await.unwrap();
    // assert!(matches!(result, LockResult::ObjectLocked(_)));
}
