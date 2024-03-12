use axum::{
    body::Body,
    http::{self, Method, Request, StatusCode},
    Router,
};
use serde::{de::DeserializeOwned, Serialize};
use tower::ServiceExt;

mod crdt;
mod secret;
mod session;

// TODO(nick): bring these back as they make sense. Make sure to refactor, redo, drop, etc. as we go.
// mod change_set;
// mod component;
// mod scenario;
// mod functions;

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
        .header(http::header::AUTHORIZATION, format!("Bearer {auth_token}"));

    let api_request = api_request
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

#[allow(dead_code)]
pub async fn api_request_auth_no_response<Req: Serialize>(
    app: Router,
    method: Method,
    uri: impl AsRef<str>,
    auth_token: impl AsRef<str>,
    request: &Req,
) {
    let auth_token = auth_token.as_ref();
    let uri = uri.as_ref();
    let api_request = Request::builder()
        .method(method)
        .uri(uri)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {auth_token}"));

    let api_request = api_request
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!(&request)).expect("cannot turn request to json"),
        ))
        .expect("cannot create api request");
    let response = app.oneshot(api_request).await.expect("cannot send request");
    let status = response.status();
    let body = hyper::body::to_bytes(response.into_body())
        .await
        .expect("cannot read body");
    if status != StatusCode::OK {
        dbg!(&body);
        assert_eq!(
            StatusCode::OK, // expected,
            status,         // actual
        );
    }

    assert_eq!(body, "", "response is not empty");
}
