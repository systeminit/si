use std::collections::HashSet;

use axum::http::Method;
use dal::StandardModel;
use dal_test::{test, test_harness::create_schema as dal_create_schema};
use sdf::service::schema::{
    create_schema::{CreateSchemaRequest, CreateSchemaResponse},
    get_schema::{GetSchemaRequest, GetSchemaResponse},
    list_schemas::{ListSchemaRequest, ListSchemaResponse},
};

use crate::{
    service_tests::{api_request_auth_json_body, api_request_auth_query},
    test_setup,
};

#[test]
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
        _encr_key,
        app,
        _nba,
        auth_token,
        dal_ctx,
        _job_processor,
        _council_subject_prefix,
    );
    let visibility = *dal_ctx.visibility();
    let request = CreateSchemaRequest {
        name: "fancyPants".to_string(),
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
}

#[test]
async fn list_schemas() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        _conn,
        _txn,
        _nats_conn,
        _nats,
        _veritech,
        _encr_key,
        app,
        nba,
        auth_token,
        dal_ctx,
        _job_processor,
        _council_subject_prefix,
    );
    let rand_schema1 = dal_create_schema(&dal_ctx).await;
    let rand_schema1_name = rand_schema1.name();
    let rand_schema2 = dal_create_schema(&dal_ctx).await;
    let rand_schema2_name = rand_schema2.name();

    let visibility = *dal_ctx.visibility();
    dal_ctx.commit().await.expect("cannot commit txn");

    let request = ListSchemaRequest {
        visibility,
        workspace_pk: *nba.workspace.pk(),
    };
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

#[test]
async fn get_schemas() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        _conn,
        _txn,
        _nats_conn,
        _nats,
        _veritech,
        _encr_key,
        app,
        nba,
        auth_token,
        dal_ctx,
        _job_processor,
        _council_subject_prefix,
    );
    let schema_one = dal_create_schema(&dal_ctx).await;

    let visibility = *dal_ctx.visibility();
    dal_ctx.commit().await.expect("cannot commit txn");

    let request = GetSchemaRequest {
        visibility,
        schema_id: *schema_one.id(),
        workspace_pk: *nba.workspace.pk(),
    };
    let response: GetSchemaResponse =
        api_request_auth_query(app, "/api/schema/get_schema", &auth_token, &request).await;
    assert_eq!(response, schema_one);
}
