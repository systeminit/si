use std::collections::HashSet;

use crate::dal::test;
use crate::service_tests::api_request_auth_query;
use crate::test_setup;
use dal::test_harness::{
    create_component_for_schema_variant, create_schema, create_schema_variant,
};
use dal::{Component, SchemaKind, StandardModel, Visibility};
use sdf::service::component::get_components_metadata::{
    GetComponentsMetadataRequest, GetComponentsMetadataResponse,
};
use sdf::service::component::list_components_identification::{
    ListComponentsIdentificationRequest, ListComponentsIdentificationResponse,
};

#[test]
async fn list_components_identification() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        _conn,
        _txn,
        _nats_conn,
        _nats,
        veritech,
        encr_key,
        app,
        nba,
        auth_token,
        dal_ctx,
        dal_txns,
    );
    let visibility = Visibility::new_head(false);
    let schema = create_schema(&dal_ctx, &SchemaKind::Concept).await;
    let schema_variant = create_schema_variant(&dal_ctx, *schema.id()).await;
    schema_variant
        .set_schema(&dal_ctx, schema.id())
        .await
        .expect("Unable to set schema");

    let component_name1 = "poop";
    let component_name2 = "ilikemybutt";
    for name in &[component_name1, component_name2] {
        let _component =
            Component::new_for_schema_variant_with_node(&dal_ctx, &name, schema_variant.id())
                .await
                .expect("cannot create new component");
    }
    dal_txns.commit().await.expect("cannot commit transaction");

    let request = ListComponentsIdentificationRequest {
        visibility,
        workspace_id: *nba.workspace.id(),
    };
    let response: ListComponentsIdentificationResponse = api_request_auth_query(
        app,
        "/api/component/list_components_identification",
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

#[test]
async fn get_components_metadata() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        _conn,
        _txn,
        _nats_conn,
        _nats,
        veritech,
        encr_key,
        app,
        nba,
        auth_token,
        dal_ctx,
        dal_txns,
    );
    let visibility = Visibility::new_head(false);

    let schema = create_schema(&dal_ctx, &SchemaKind::Concept).await;

    let schema_variant = create_schema_variant(&dal_ctx, *schema.id()).await;
    schema_variant
        .set_schema(&dal_ctx, schema.id())
        .await
        .expect("cannot set schema variant");

    let component = create_component_for_schema_variant(&dal_ctx, schema_variant.id()).await;
    dal_txns.commit().await.expect("cannot commit transaction");

    let request = GetComponentsMetadataRequest {
        visibility,
        workspace_id: *nba.workspace.id(),
        system_id: None,
    };
    let response: GetComponentsMetadataResponse = api_request_auth_query(
        app,
        "/api/component/get_components_metadata",
        &auth_token,
        &request,
    )
    .await;

    assert_eq!(response.data[0].schema_name, schema.name());
}
