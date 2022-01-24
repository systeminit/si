use axum::http::Method;

use crate::service_tests::{api_request_auth_json_body, api_request_auth_query};
use crate::test_setup;
use dal::{Component, HistoryActor, StandardModel, Tenancy, Visibility};
use sdf::service::application::create_application::{
    CreateApplicationRequest, CreateApplicationResponse,
};
use sdf::service::application::list_applications::{
    ListApplicationRequest, ListApplicationResponse,
};

#[tokio::test]
async fn create_application() {
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
        nba,
        auth_token
    );
    let visibility = Visibility::new_head(false);
    let request = CreateApplicationRequest {
        name: "fancyPants".to_string(),
        visibility,
        workspace_id: *nba.workspace.id(),
    };
    let response: CreateApplicationResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/application/create_application",
        &auth_token,
        &request,
    )
    .await;
    assert_eq!(response.application.name(), "fancyPants");
}

#[tokio::test]
async fn list_applications() {
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

    let (component, _node) = Component::new_application_with_node(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "poop",
    )
    .await
    .expect("cannot create new application");
    txn.commit().await.expect("cannot commit transaction");

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
    assert_eq!(response.list[0].application.name(), component.name());
}
