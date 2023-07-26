use std::collections::HashSet;

use axum::{http::Method, Router};
use dal::{DalContext, StandardModel};
use dal_test::{sdf_test, test_harness::create_schema as dal_create_schema, AuthTokenRef};
use sdf_server::service::schema::{
    create_schema::{CreateSchemaRequest, CreateSchemaResponse},
    get_schema::{GetSchemaRequest, GetSchemaResponse},
    list_schemas::{ListSchemaRequest, ListSchemaResponse},
};

use crate::service_tests::{api_request_auth_json_body, api_request_auth_query};

#[sdf_test]
async fn create_schema(ctx: DalContext, app: Router, AuthTokenRef(auth_token): AuthTokenRef<'_>) {
    let visibility = *ctx.visibility();
    let request = CreateSchemaRequest {
        name: "fancyPants".to_string(),
        visibility,
    };

    let response: CreateSchemaResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/schema/create_schema",
        auth_token,
        &request,
    )
    .await;
    assert_eq!(response.schema.name(), "fancyPants");
}

#[sdf_test]
async fn list_schemas(ctx: DalContext, app: Router, AuthTokenRef(auth_token): AuthTokenRef<'_>) {
    let rand_schema1 = dal_create_schema(&ctx).await;
    let rand_schema1_name = rand_schema1.name();
    let rand_schema2 = dal_create_schema(&ctx).await;
    let rand_schema2_name = rand_schema2.name();

    let visibility = *ctx.visibility();
    ctx.blocking_commit().await.expect("cannot commit txn");

    let request = ListSchemaRequest { visibility };

    let response: ListSchemaResponse =
        api_request_auth_query(app, "/api/schema/list_schemas", auth_token, &request).await;
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

#[sdf_test]
async fn get_schemas(ctx: DalContext, app: Router, AuthTokenRef(auth_token): AuthTokenRef<'_>) {
    let schema_one = dal_create_schema(&ctx).await;

    let visibility = *ctx.visibility();
    ctx.blocking_commit().await.expect("cannot commit txn");

    let request = GetSchemaRequest {
        visibility,
        schema_id: *schema_one.id(),
    };

    let response: GetSchemaResponse =
        api_request_auth_query(app, "/api/schema/get_schema", auth_token, &request).await;
    assert_eq!(response, schema_one);
}
