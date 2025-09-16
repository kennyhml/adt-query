use sapi::{
    adt::{
        api,
        models::vfs::{Facet, FacetOrderBuilder, PreselectionBuilder},
    },
    query::StatelessQuery,
};

mod common;

#[tokio::test]
async fn local_objects_are_retrieved() {
    let client = common::setup_test_system_client();

    let endpoint = api::repository::RepositoryContentBuilder::default()
        .order(
            FacetOrderBuilder::default()
                .push(Facet::Owner)
                .push(Facet::Package)
                .push(Facet::Group)
                .push(Facet::Type)
                .build()
                .unwrap(),
        )
        .push_preselection(
            PreselectionBuilder::default()
                .facet(Facet::Owner)
                .include("DEVELOPER")
                .build()
                .unwrap(),
        )
        .push_preselection(
            PreselectionBuilder::default()
                .facet(Facet::Package)
                .include("$TMP")
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();
    let result = endpoint.query(&client).await.unwrap();
    println!("{:?}", result);
}

#[tokio::test]
async fn available_facets_are_retrieved() {
    let client = common::setup_test_system_client();

    let endpoint = api::repository::AvailableFacets::default();
    let result = endpoint.query(&client).await.unwrap();
    assert!(
        result.body().facets.len() > 5,
        "At least 5 Facets should be retrieved"
    )
}

#[tokio::test]
async fn all_object_properties_are_retrieved() {
    let client = common::setup_test_system_client();

    let endpoint = api::repository::ObjectPropertiesBuilder::default()
        .object_uri("/sap/bc/adt/oo/classes/cl_ris_adt_res_app")
        .build()
        .unwrap();
    let result = endpoint.query(&client).await.unwrap();
    assert_eq!(result.body().object.name, "CL_RIS_ADT_RES_APP");
}

#[tokio::test]
async fn selected_object_properties_are_retrieved() {
    let client = common::setup_test_system_client();

    let endpoint = api::repository::ObjectPropertiesBuilder::default()
        .object_uri("/sap/bc/adt/oo/classes/cl_ris_adt_res_app")
        .include_facet(Facet::Package)
        .include_facet(Facet::ApplicationComponent)
        .build()
        .unwrap();
    let result = endpoint.query(&client).await.unwrap();
    assert!(
        result
            .body()
            .properties
            .iter()
            .all(|v| matches!(v.facet, Facet::Package | Facet::ApplicationComponent))
    );
}

#[tokio::test]
async fn no_transports_are_retrieved() {
    let client = common::setup_test_system_client();

    let endpoint = api::repository::ObjectTransportsBuilder::default()
        .object_uri("/sap/bc/adt/oo/classes/cl_ris_adt_res_app")
        .build()
        .unwrap();
    let result = endpoint.query(&client).await.unwrap();
    assert!(result.body().transports.is_empty())
}
