use crate::dal::test;
use crate::service_tests::{
    api_request_auth_empty, api_request_auth_json_body, api_request_auth_query,
};
use crate::test_setup;
use axum::http::Method;
use dal::test_harness::create_change_set as dal_create_change_set;
use dal::test_harness::create_edit_session as dal_create_edit_session;
use dal::{HistoryActor, StandardModel};
use sdf::service::change_set::apply_change_set::{ApplyChangeSetRequest, ApplyChangeSetResponse};
use sdf::service::change_set::cancel_edit_session::{
    CancelEditSessionRequest, CancelEditSessionResponse,
};
use sdf::service::change_set::create_change_set::{
    CreateChangeSetRequest, CreateChangeSetResponse,
};
use sdf::service::change_set::get_change_set::{GetChangeSetRequest, GetChangeSetResponse};
use sdf::service::change_set::list_open_change_sets::ListOpenChangeSetsResponse;
use sdf::service::change_set::save_edit_session::{
    SaveEditSessionRequest, SaveEditSessionResponse,
};
use sdf::service::change_set::start_edit_session::{
    StartEditSessionRequest, StartEditSessionResponse,
};

#[test]
async fn list_open_change_sets() {
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
        dal_txns,
        _faktory,
    );
    let dal_ctx = dal_ctx.clone_with_new_billing_account_tenancies(*nba.billing_account.id());
    let _a_change_set = dal_create_change_set(&dal_ctx).await;
    let _b_change_set = dal_create_change_set(&dal_ctx).await;
    dal_txns.commit().await.expect("cannot commit transaction");
    let response: ListOpenChangeSetsResponse = api_request_auth_empty(
        app,
        Method::GET,
        "/api/change_set/list_open_change_sets",
        &auth_token,
    )
    .await;
    assert_eq!(response.list.len(), 2);
}

#[test]
async fn create_change_set() {
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
        _dal_ctx,
        dal_txns,
        _faktory,
    );
    let request: CreateChangeSetRequest = CreateChangeSetRequest {
        change_set_name: "mastodon".to_string(),
        current_edit_session_pk: None,
    };
    let response: CreateChangeSetResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/change_set/create_change_set",
        &auth_token,
        &request,
    )
    .await;
    assert_eq!(&response.change_set.name, "mastodon");
    assert_eq!(
        &response.edit_session.change_set_pk,
        &response.change_set.pk
    );
}

#[test]
async fn get_change_set() {
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
        dal_txns,
        _faktory,
    );
    let dal_ctx = dal_ctx.clone_with_new_billing_account_tenancies(*nba.billing_account.id());
    let change_set = dal_create_change_set(&dal_ctx).await;
    dal_txns.commit().await.expect("cannot commit txn");

    let request = GetChangeSetRequest { pk: change_set.pk };
    let response: GetChangeSetResponse =
        api_request_auth_query(app, "/api/change_set/get_change_set", &auth_token, &request).await;
    assert_eq!(&response.change_set, &change_set);
}

#[test]
async fn start_edit_session() {
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
        dal_txns,
        _faktory,
    );
    let _history_actor = HistoryActor::SystemInit;
    let change_set = dal_create_change_set(&dal_ctx).await;
    dal_txns.commit().await.expect("cannot commit txn");

    let request = StartEditSessionRequest {
        change_set_pk: change_set.pk,
    };
    let _response: StartEditSessionResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/change_set/start_edit_session",
        &auth_token,
        &request,
    )
    .await;
}

#[test]
async fn cancel_edit_session() {
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
        dal_txns,
        _faktory,
    );
    let dal_ctx = dal_ctx.clone_with_new_billing_account_tenancies(*nba.billing_account.id());
    let change_set = dal_create_change_set(&dal_ctx).await;
    let edit_session = dal_create_edit_session(&dal_ctx, &change_set).await;
    dal_txns.commit().await.expect("cannot commit txn");

    let request = CancelEditSessionRequest {
        edit_session_pk: edit_session.pk,
    };
    let _response: CancelEditSessionResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/change_set/cancel_edit_session",
        &auth_token,
        &request,
    )
    .await;
}

#[test]
async fn save_edit_session() {
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
        dal_txns,
        _faktory,
    );
    let dal_ctx = dal_ctx.clone_with_new_billing_account_tenancies(*nba.billing_account.id());
    let change_set = dal_create_change_set(&dal_ctx).await;
    let edit_session = dal_create_edit_session(&dal_ctx, &change_set).await;
    dal_txns.commit().await.expect("cannot commit txn");

    let request = SaveEditSessionRequest {
        edit_session_pk: edit_session.pk,
    };
    let _response: SaveEditSessionResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/change_set/save_edit_session",
        &auth_token,
        &request,
    )
    .await;
}

#[test]
async fn apply_change_set() {
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
        dal_txns,
        _faktory,
    );
    let dal_ctx = dal_ctx.clone_with_new_billing_account_tenancies(*nba.billing_account.id());
    let change_set = dal_create_change_set(&dal_ctx).await;
    dal_txns.commit().await.expect("cannot commit txn");

    let request = ApplyChangeSetRequest {
        change_set_pk: change_set.pk,
    };
    let _response: ApplyChangeSetResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/change_set/apply_change_set",
        &auth_token,
        &request,
    )
    .await;
}
