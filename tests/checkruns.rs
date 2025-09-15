mod common;

// #[tokio::test]
// async fn the_available_checkrun_reporters_are_retrieved() {
//     let client = common::setup_test_system_client();

//     let endpoint = sapi::adt::api::checkruns::Reporters {};
//     endpoint.query(&client).await.unwrap();
// }

// #[tokio::test]
// async fn checkrun_reports_warnings() {
//     let client = common::setup_test_system_client();

//     // make a get request for csrf token first
//     let endpoint = sapi::adt::api::checkruns::Reporters {};
//     endpoint.query(&client).await.unwrap();

//     let object = ObjectBuilder::default()
//         .object_uri("/sap/bc/adt/functions/groups/http_runtime/fmodules/http_get_handler_list")
//         .version("active")
//         .build()
//         .unwrap();

//     let endpoint = sapi::adt::api::checkruns::RunCheckBuilder::default()
//         .objects(ObjectListBuilder::default().object(object).build().unwrap())
//         .reporter("abapCheckRun")
//         .build()
//         .unwrap();

//     endpoint.query(&client).await.unwrap();
// }
