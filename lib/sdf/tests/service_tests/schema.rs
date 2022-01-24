use std::collections::HashSet;

use axum::http::Method;

use dal::test_harness::create_schema as dal_create_schema;
use dal::{HistoryActor, SchemaKind, StandardModel, Tenancy, Visibility};
use sdf::service::schema::create_schema::{CreateSchemaRequest, CreateSchemaResponse};
use sdf::service::schema::get_schema::{GetSchemaRequest, GetSchemaResponse};
use sdf::service::schema::list_schemas::{ListSchemaRequest, ListSchemaResponse};

use crate::service_tests::{api_request_auth_json_body, api_request_auth_query};
use crate::test_setup;

#[tokio::test]
async fn create_schema() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        _conn,
        _txn,
        _nats_conn,
        _nats,
        _veritech,
        app,
        _nba,
        auth_token
    );
    let visibility = Visibility::new_head(false);
    let request = CreateSchemaRequest {
        name: "fancyPants".to_string(),
        kind: SchemaKind::Concrete,
        visibility,
    };
    let response: CreateSchemaResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/schema/create_schema",
        &auth_token,
        &request,
    )
    .await;
    assert_eq!(response.schema.name(), "fancyPants");
    assert_eq!(response.schema.kind(), &SchemaKind::Concrete);
}

#[tokio::test]
async fn list_schemas() {
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
    let tenancy = Tenancy::new_billing_account(vec![*nba.billing_account.id()]);
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let rand_schema1 = dal_create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;
    let rand_schema1_name = rand_schema1.name();
    let rand_schema2 = dal_create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;
    let rand_schema2_name = rand_schema2.name();

    txn.commit().await.expect("cannot commit txn");
    let request = ListSchemaRequest { visibility };
    let response: ListSchemaResponse =
        api_request_auth_query(app, "/api/schema/list_schemas", &auth_token, &request).await;

    let filtered_schema_names: HashSet<String> = response
        .list
        .into_iter()
        .filter_map(|schema| match schema.name() {
            schema_name if schema_name == rand_schema1_name || schema_name == rand_schema2_name => {
                Some(schema_name.to_string())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        filtered_schema_names,
        vec![rand_schema1_name.to_string(), rand_schema2_name.to_string()]
            .into_iter()
            .collect()
    );
}

#[tokio::test]
async fn get_schemas() {
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
    let tenancy = Tenancy::new_billing_account(vec![*nba.billing_account.id()]);
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema_one = dal_create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;
    txn.commit().await.expect("cannot commit txn");
    let request = GetSchemaRequest {
        visibility,
        schema_id: *schema_one.id(),
    };
    let response: GetSchemaResponse =
        api_request_auth_query(app, "/api/schema/get_schema", &auth_token, &request).await;
    assert_eq!(response, schema_one);
}
