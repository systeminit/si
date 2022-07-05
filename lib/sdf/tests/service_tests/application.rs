use axum::http::Method;

use crate::dal::test;
use crate::service_tests::{api_request_auth_json_body, api_request_auth_query};
use crate::test_setup;
use dal::{Component, StandardModel, Visibility};
use sdf::service::application::create_application::{
    CreateApplicationRequest, CreateApplicationResponse,
};
use sdf::service::application::list_applications::{
    ListApplicationRequest, ListApplicationResponse,
};

#[test]
async fn create_application() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        conn,
        _txn,
        _nats_conn,
        _nats,
        _veritech,
        _encr_key,
        app,
        nba,
        auth_token,
        _dal_ctx,
        dal_txns,
        _faktory,
    );
    let _visibility = Visibility::new_head(false);
    let request = CreateApplicationRequest {
        name: "fancyPants".to_string(),
        workspace_id: *nba.workspace.id(),
    };

    let _response: CreateApplicationResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/application/create_application",
        &auth_token,
        &request,
    )
    .await;
}

#[test]
async fn list_applications() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        conn,
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
        _faktory,
    );

    let visibility = Visibility::new_head(false);

    let (_component, _node) = Component::new_application_with_node(&dal_ctx, "fancyPants")
        .await
        .expect("cannot create new application");

    // TODO This commit is important to the test. We should probably figure out a way of doing this without requiring committing to the test DB.
    dal_txns.commit().await.expect("cannot commit transaction");

    let request = ListApplicationRequest {
        visibility,
        workspace_id: *nba.workspace.id(),
    };
    let response: ListApplicationResponse = api_request_auth_query(
        app,
        "/api/application/list_applications",
        &auth_token,
        &request,
    )
    .await;
    assert_eq!(response.list.len(), 1);
}
