use adt_query::{api::object, dispatch::StatefulDispatch};
mod common;

#[tokio::test()]
async fn lock_is_retained_in_stateful_session() {
    let client = common::setup_test_system_client();

    let op = object::LockBuilder::default()
        .object_uri("programs/programs/zwegwerf1")
        .access_mode(object::AccessMode::Modify)
        .build()
        .unwrap();

    let ctx = client.create_user_session();
    let result = op.dispatch(&client, ctx).await.unwrap();

    let handle = &result.body().lock_handle;
    let op = object::UnlockBuilder::default()
        .object_uri("programs/programs/zwegwerf1")
        .lock_handle(handle)
        .build()
        .unwrap();

    let result = op.dispatch(&client, ctx).await.unwrap();
    assert_eq!(result.status(), 200)
}

#[tokio::test]
async fn object_is_already_locked() {
    let client = common::setup_test_system_client();

    let op = object::LockBuilder::default()
        .object_uri("programs/programs/zdemo1")
        .access_mode(object::AccessMode::Modify)
        .build()
        .unwrap();

    let ctx = client.create_user_session();
    let result = op.dispatch(&client, ctx).await.unwrap();
    let handle = &result.body().lock_handle;

    // Query again, this should cause an `AlreadyLocked` Error.
    let result = op.dispatch(&client, ctx).await;
    assert!(
        matches!(result, Err(_)),
        "Expected the resource to be locked already."
    );

    // Unlock
    let op = object::UnlockBuilder::default()
        .object_uri("programs/programs/zdemo1")
        .lock_handle(handle)
        .build()
        .unwrap();

    let result = op.dispatch(&client, ctx).await.unwrap();
    assert_eq!(result.status(), 200)
}

#[tokio::test]
async fn dropping_context_unlocks_objects() {
    let client = common::setup_test_system_client();

    let op = object::LockBuilder::default()
        .object_uri("programs/programs/zabapgit_standalone")
        .access_mode(object::AccessMode::Modify)
        .build()
        .unwrap();

    let ctx = client.create_user_session();
    op.dispatch(&client, ctx).await.unwrap();

    assert_eq!(client.destroy_user_session(ctx).await.unwrap(), true);
}
