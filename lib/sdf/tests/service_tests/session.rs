use axum::http::{Method, StatusCode};
use dal_test::test;
use sdf::service::session::{
    get_defaults::GetDefaultsResponse,
    login::{LoginRequest, LoginResponse},
    restore_authentication::RestoreAuthenticationResponse,
};

use crate::{
    service_tests::{api_request, api_request_auth_empty, api_request_raw},
    test_setup,
};

#[test]
async fn login() {
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
        _auth_token,
        _dal_ctx,
        _job_processor,
        _council_subject_prefix,
    );

    let request = LoginRequest {
        billing_account_name: nba.billing_account.name().to_string(),
        user_email: nba.user.email().to_string(),
        user_password: "snakes".to_string(),
    };
    let _response: LoginResponse = api_request(app.clone(), "/api/session/login", &request).await;

    let wrong_ba_request = LoginRequest {
        billing_account_name: "poop tastic".to_string(),
        user_email: nba.user.email().to_string(),
        user_password: "snakes".to_string(),
    };
    let (wrong_ba_status, wrong_ba_response) =
        api_request_raw(app.clone(), "/api/session/login", &wrong_ba_request).await;
    assert_eq!(wrong_ba_status, StatusCode::CONFLICT);
    assert_eq!(wrong_ba_response["error"]["message"], "login failed");

    let wrong_email_request = LoginRequest {
        billing_account_name: nba.billing_account.name().to_string(),
        user_email: "spinklehovfer@example.com".to_string(),
        user_password: "snakes".to_string(),
    };
    let (wrong_email_status, wrong_email_response) =
        api_request_raw(app.clone(), "/api/session/login", &wrong_email_request).await;
    assert_eq!(wrong_email_status, StatusCode::CONFLICT);
    assert_eq!(wrong_email_response["error"]["message"], "login failed");

    let wrong_password_request = LoginRequest {
        billing_account_name: nba.billing_account.name().to_string(),
        user_email: nba.user.email().to_string(),
        user_password: "poop".to_string(),
    };
    let (wrong_password_status, wrong_password_response) =
        api_request_raw(app.clone(), "/api/session/login", &wrong_password_request).await;
    assert_eq!(wrong_password_status, StatusCode::CONFLICT);
    assert_eq!(wrong_password_response["error"]["message"], "login failed");
}

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
        nba,
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
    assert_eq!(nba.billing_account, response.billing_account);
    assert_eq!(nba.user, response.user);
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
        nba,
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
    assert_eq!(nba.workspace, response.workspace);
}
