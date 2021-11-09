use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::{http, Router};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tower::ServiceExt;

mod session;
mod signup;

pub async fn api_request_auth_empty<Res: DeserializeOwned>(
    app: Router,
    method: Method,
    uri: impl AsRef<str>,
    auth_token: impl AsRef<str>,
) -> Res {
    let auth_token = auth_token.as_ref();
    let uri = uri.as_ref();
    let api_request = Request::builder()
        .method(method)
        .uri(uri)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", auth_token))
        .body(Body::empty())
        .expect("cannot create api request");
    let response = app.oneshot(api_request).await.expect("cannot send request");
    let status = response.status();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_json: serde_json::Value =
        serde_json::from_slice(&body).expect("response is not valid json");
    if status != StatusCode::OK {
        dbg!(&body_json);
        assert_eq!(status, StatusCode::OK);
    }
    serde_json::from_value(body_json).expect("response is not a valid rust struct")
}


pub async fn api_request<Req: Serialize, Res: DeserializeOwned>(
    app: Router,
    uri: impl AsRef<str>,
    request: &Req,
) -> Res {
    let uri = uri.as_ref();
    let api_request = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!(&request)).expect("cannot turn request to json"),
        ))
        .expect("cannot create api request");
    let response = app.oneshot(api_request).await.expect("cannot send request");
    let status = response.status();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_json: serde_json::Value =
        serde_json::from_slice(&body).expect("response is not valid json");
    if status != StatusCode::OK {
        dbg!(&body_json);
        assert_eq!(status, StatusCode::OK);
    }
    serde_json::from_value(body_json).expect("response is not a valid rust struct")
}

pub async fn api_request_raw<Req: Serialize>(
    app: Router,
    uri: impl AsRef<str>,
    request: &Req,
) -> (StatusCode, serde_json::Value) {
    let uri = uri.as_ref();
    let api_request = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!(&request)).expect("cannot turn request to json"),
        ))
        .expect("cannot create api request");
    let response = app.oneshot(api_request).await.expect("cannot send request");
    let status = response.status();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_json: serde_json::Value =
        serde_json::from_slice(&body).expect("response is not valid json");
    (status, body_json)
}

#[macro_export]
macro_rules! test_setup {
    ($ctx:ident,
     $secret_key:ident,
     $pg:ident,
     $pgconn:ident,
     $pgtxn:ident,
     $nats_conn:ident,
     $nats:ident,
     $app:ident,
     $nba:ident,
     $auth_token:ident) => {
        dal::test_harness::one_time_setup()
            .await
            .expect("one time setup failed");
        let $ctx = dal::test_harness::TestContext::init().await;
        let ($pg, $nats_conn, $secret_key) = $ctx.entries();
        let $nats = $nats_conn.transaction();
        let mut $pgconn = $pg.get().await.expect("cannot connect to pg");
        let $pgtxn = $pgconn.transaction().await.expect("cannot create txn");
        let ($app, _) = sdf::build_service(
            $pg.clone(),
            $nats_conn.clone(),
            JwtSigningKey {
                key: $secret_key.clone(),
            },
        )
        .expect("cannot build new server");
        let $app: axum::Router = $app.into();
        let ($nba, $auth_token) =
            dal::test_harness::billing_account_signup(&$pgtxn, &$nats, &$secret_key).await;
        $pgtxn.commit().await.expect("cannot commit txn");
        $nats.commit().await.expect("cannot commit nats");
        let $nats = $nats_conn.transaction();
        let $pgtxn = $pgconn.transaction().await.expect("cannot create txn");
    };
}
