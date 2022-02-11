use std::collections::HashSet;

use crate::service_tests::api_request_auth_query;
use crate::test_setup;
use dal::test_harness::{create_schema, create_schema_variant};
use dal::{Component, HistoryActor, SchemaKind, StandardModel, Tenancy, Visibility};
use sdf::service::component::get_component_metadata::{
    GetComponentMetadataRequest, GetComponentMetadataResponse,
};
use sdf::service::component::list_components_names_only::{
    ListComponentNamesOnlyRequest, ListComponentNamesOnlyResponse,
};

#[tokio::test]
async fn list_components_names_only() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
        app,
        nba,
        auth_token
    );
    let visibility = Visibility::new_head(false);
    let tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    let history_actor = HistoryActor::SystemInit;

    let component_name1 = "poop";
    let component_name2 = "ilikemybutt";
    for name in &[component_name1, component_name2] {
        let _component = Component::new(&txn, &nats, &tenancy, &visibility, &history_actor, &name)
            .await
            .expect("cannot create new component");
    }
    txn.commit().await.expect("cannot commit transaction");

    let request = ListComponentNamesOnlyRequest {
        visibility,
        workspace_id: *nba.workspace.id(),
    };
    let response: ListComponentNamesOnlyResponse = api_request_auth_query(
        app,
        "/api/component/list_components_names_only",
        &auth_token,
        &request,
    )
    .await;

    let filtered_components_names_only: HashSet<String> = response
        .list
        .iter()
        .filter_map(|list_item| match &list_item.label {
            component_name
                if component_name == component_name1 || component_name == component_name2 =>
            {
                Some(component_name.to_string())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        filtered_components_names_only,
        vec![component_name1.to_string(), component_name2.to_string()]
            .into_iter()
            .collect()
    );
}

#[tokio::test]
async fn get_component_metadata() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        app,
        nba,
        auth_token
    );
    let visibility = Visibility::new_head(false);
    let tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    let history_actor = HistoryActor::SystemInit;

    let mut schema_tenancy = tenancy.clone();
    schema_tenancy.universal = true;
    let schema = create_schema(
        &txn,
        &nats,
        &schema_tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concept,
    )
    .await;

    let schema_variant =
        create_schema_variant(&txn, &nats, &schema_tenancy, &visibility, &history_actor).await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema variant");

    let component = Component::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "hello friend",
    )
    .await
    .expect("cannot create new component");
    component
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("Cannot attach component to schema");
    component
        .set_schema_variant(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            schema_variant.id(),
        )
        .await
        .expect("Cannot attach component to schema variant");
    txn.commit().await.expect("cannot commit transaction");

    let request = GetComponentMetadataRequest {
        visibility,
        component_id: *component.id(),
        workspace_id: *nba.workspace.id(),
        system_id: None,
    };
    let response: GetComponentMetadataResponse = api_request_auth_query(
        app,
        "/api/component/get_component_metadata",
        &auth_token,
        &request,
    )
    .await;

    assert_eq!(response.schema_name, schema.name());
}
