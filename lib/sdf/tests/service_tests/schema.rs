use crate::service_tests::api_request_auth_json_body;
use crate::test_setup;
use axum::http::Method;
use dal::SchemaKind;
use sdf::service::schema::create_schema::{CreateSchemaRequest, CreateSchemaResponse};

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
        app,
        _nba,
        auth_token
    );
    let request = CreateSchemaRequest {
        name: "fancyPants".to_string(),
        kind: SchemaKind::Concrete,
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
