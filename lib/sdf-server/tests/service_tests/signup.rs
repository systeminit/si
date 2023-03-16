use axum::{
    body::Body,
    http::{self, Method, Request, StatusCode},
    Router,
};
use dal_test::{
    test,
    test_harness::{generate_fake_name, one_time_setup, TestContext},
};
use sdf_server::service::signup::{self, create_account::CreateAccountResponse};
use tower::ServiceExt;

#[test]
async fn create_account() {
    one_time_setup().await.expect("cannot setup tests");
    let ctx = TestContext::init().await;
    let (pg, nats, job_processor, _veritech, encr_key, jwt_secret_key, council_subject_prefix) =
        ctx.entries();
    let veritech = veritech_client::Client::new(nats.clone());
    let telemetry = ctx.telemetry();
    let (app, _, _) = sdf_server::build_service(
        telemetry,
        pg.clone(),
        nats.clone(),
        job_processor,
        veritech,
        *encr_key,
        jwt_secret_key.clone(),
        "my-signup-secret".into(),
        council_subject_prefix.to_owned(),
        None,
    )
    .expect("cannot build new server");
    let app: Router = app;

    let fake_name = generate_fake_name();
    let request = signup::create_account::CreateAccountRequest {
        workspace_name: fake_name,
        user_name: "bobo".to_string(),
        user_email: "bobo@tclown.org".to_string(),
        user_password: "bobor7les".to_string(),
        signup_secret: "my-signup-secret".to_string(),
    };
    let api_request = Request::builder()
        .method(Method::POST)
        .uri("/api/signup/create_account")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!(&request)).expect("cannot turn request to json"),
        ))
        .expect("cannot create api request");
    let response = app.oneshot(api_request).await.expect("cannot send request");
    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_json: serde_json::Value =
        serde_json::from_slice(&body).expect("response is not valid json");
    let response: CreateAccountResponse =
        serde_json::from_value(body_json).expect("response is not a valid rust struct");
    assert!(response.success);
}

#[test]
async fn create_account_invalid_signup_secret() {
    one_time_setup().await.expect("cannot setup tests");
    let ctx = TestContext::init().await;
    let (pg, nats, job_processor, _veritech, encr_key, jwt_secret_key, council_subject_prefix) =
        ctx.entries();
    let veritech = veritech_client::Client::new(nats.clone());
    let telemetry = ctx.telemetry();
    let (app, _, _) = sdf_server::build_service(
        telemetry,
        pg.clone(),
        nats.clone(),
        job_processor,
        veritech,
        *encr_key,
        jwt_secret_key.clone(),
        "nope-nope-nope".into(),
        council_subject_prefix.to_owned(),
        None,
    )
    .expect("cannot build new server");
    let app: Router = app;

    let fake_name = generate_fake_name();
    let request = signup::create_account::CreateAccountRequest {
        workspace_name: fake_name,
        user_name: "bobo".to_string(),
        user_email: "bobo@tclown.org".to_string(),
        user_password: "bobor7les".to_string(),
        signup_secret: "i-was-wrong".to_string(),
    };
    let api_request = Request::builder()
        .method(Method::POST)
        .uri("/api/signup/create_account")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!(&request)).expect("cannot turn request to json"),
        ))
        .expect("cannot create api request");
    let response = app.oneshot(api_request).await.expect("cannot send request");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_json: serde_json::Value =
        serde_json::from_slice(&body).expect("response is not valid json");
    let error = body_json
        .get("error")
        .expect("failed to get error field in response");
    assert_eq!(
        &400,
        error
            .get("statusCode")
            .expect("failed to get code field in error response")
    );
    assert_eq!(
        "signup failed",
        error
            .get("message")
            .expect("failed to get code field in error response")
    );
}
