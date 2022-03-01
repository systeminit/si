use crate::service_tests::{api_request, api_request_auth_empty, api_request_raw};
use crate::test_setup;
use axum::http::{Method, StatusCode};
use sdf::service::session::get_defaults::GetDefaultsResponse;
use sdf::service::session::login::{LoginRequest, LoginResponse};
use sdf::service::session::restore_authentication::RestoreAuthenticationResponse;

#[tokio::test]
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
        _auth_token
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
    assert_eq!(wrong_ba_status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_ba_response["error"]["message"], "login failed");

    let wrong_email_request = LoginRequest {
        billing_account_name: nba.billing_account.name().to_string(),
        user_email: "spinklehovfer@example.com".to_string(),
        user_password: "snakes".to_string(),
    };
    let (wrong_email_status, wrong_email_response) =
        api_request_raw(app.clone(), "/api/session/login", &wrong_email_request).await;
    assert_eq!(wrong_email_status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_email_response["error"]["message"], "login failed");

    let wrong_password_request = LoginRequest {
        billing_account_name: nba.billing_account.name().to_string(),
        user_email: nba.user.email().to_string(),
        user_password: "poop".to_string(),
    };
    let (wrong_password_status, wrong_password_response) =
        api_request_raw(app.clone(), "/api/session/login", &wrong_password_request).await;
    assert_eq!(wrong_password_status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_password_response["error"]["message"], "login failed");
}

#[tokio::test]
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
        auth_token
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

#[tokio::test]
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
        auth_token
    );

    let response: GetDefaultsResponse = api_request_auth_empty(
        app.clone(),
        Method::GET,
        "/api/session/get_defaults",
        &auth_token,
    )
    .await;
    assert_eq!(nba.organization, response.organization);
    assert_eq!(nba.workspace, response.workspace);
}
