use axum::http::Method;

use crate::dal::test;
use crate::service_tests::{api_request_auth_json_body, api_request_auth_query};
use crate::test_setup;
use dal::{Component, HistoryActor, StandardModel, Tenancy, Visibility};
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
        txn,
        _nats_conn,
        _nats,
        _veritech,
        _encr_key,
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

    let tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    assert_eq!(
        response
            .application
            .find_prop_value_by_json_pointer::<String>(&txn, &tenancy, &visibility, "/root/si/name")
            .await
            .expect("No name prop in application"),
        Some("fancyPants".to_owned())
    );
}

#[test]
async fn list_applications() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
        app,
        nba,
        auth_token
    );
    let visibility = Visibility::new_head(false);
    let tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    let history_actor = HistoryActor::SystemInit;

    let (_component, _node) = Component::new_application_with_node(
        &txn,
        &nats,
        veritech,
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "fancyPants",
    )
    .await
    .expect("cannot create new application");
    // TODO This commit is important to the test. We should probably figure out a way of doing this without requiring committing to the test DB.
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

    let txn = conn
        .transaction()
        .await
        .expect("Unable to create transaction");
    assert_eq!(
        response.list[0]
            .application
            .find_prop_value_by_json_pointer::<String>(&txn, &tenancy, &visibility, "/root/si/name")
            .await
            .expect("Unable to find name"),
        Some("fancyPants".to_owned())
    );
}
