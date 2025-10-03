use adt_query::{
    api::object::{self, SourceCodeObject},
    dispatch::StatefulDispatch,
};
mod common;

#[tokio::test()]
async fn lock_is_retained_in_stateful_session() {
    let client = common::setup_test_system_client();

    // Lock the object to allow modifictions
    let op = object::LockBuilder::default()
        .object_uri("programs/programs/zwegwerf1")
        .access_mode(object::AccessMode::Modify)
        .build()
        .unwrap();

    let ctx = client.create_user_session();
    let result = op.dispatch(&client, ctx).await.unwrap();
    let handle = &result.body().lock_handle;

    let op = object::UpdateSourceCodeBuilder::default()
        .object(SourceCodeObject::Program("ZWEGWERF1".into()))
        .content(
            "*&---------------------------------------------------------------------*\n\
            *& Report ZWEGWERF1\n\
            *&---------------------------------------------------------------------*\n\
            *&\n\
            *&---------------------------------------------------------------------*\n\
            REPORT ZWEGWERF1.\n\
            types gtyt_test type i.\n",
        )
        .lock_handle(handle)
        .build()
        .unwrap();

    let _result = op.dispatch(&client, ctx).await.unwrap();

    // Unlock
    let op = object::UnlockBuilder::default()
        .object_uri("programs/programs/zwegwerf1")
        .lock_handle(handle)
        .build()
        .unwrap();

    let result = op.dispatch(&client, ctx).await.unwrap();
    assert_eq!(result.status(), 200)
}
