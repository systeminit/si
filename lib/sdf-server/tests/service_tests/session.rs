use axum::http::Method;
use dal_test::test;
use sdf_server::service::session::{
    get_defaults::GetDefaultsResponse, restore_authentication::RestoreAuthenticationResponse,
};

use crate::{service_tests::api_request_auth_empty, test_setup};

#[test]
async fn restore_authentication() {
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
        nw,
        auth_token,
        _dal_ctx,
        _job_processor,
        _council_subject_prefix,
    );

    let response: RestoreAuthenticationResponse = api_request_auth_empty(
        app.clone(),
        Method::GET,
        "/api/session/restore_authentication",
        &auth_token,
    )
    .await;
    assert_eq!(nw.workspace, response.workspace);
    assert_eq!(nw.user, response.user);
}

#[test]
async fn get_defaults() {
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
        nw,
        auth_token,
        dal_ctx,
        _job_processor,
        _council_subject_prefix,
    );
    dal_ctx.commit().await.expect("cannot commit txns");

    let response: GetDefaultsResponse = api_request_auth_empty(
        app.clone(),
        Method::GET,
        "/api/session/get_defaults",
        &auth_token,
    )
    .await;
    assert_eq!(nw.workspace, response.workspace);
}
