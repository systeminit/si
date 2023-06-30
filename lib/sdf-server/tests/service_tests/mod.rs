use axum::{
    body::Body,
    http::{self, Method, Request, StatusCode},
    Router,
};
use serde::{de::DeserializeOwned, Serialize};
use tower::ServiceExt;

mod change_set;
mod component;
mod scenario;
mod schema;
mod secret;
mod session;

pub async fn api_request_auth_query<Req: Serialize, Res: DeserializeOwned>(
    app: Router,
    uri: impl AsRef<str>,
    auth_token: impl AsRef<str>,
    request: &Req,
) -> Res {
    let auth_token = auth_token.as_ref();
    let uri_str = uri.as_ref();
    let params = serde_url_params::to_string(&request).expect("cannot serialize params");
    let uri = format!("{uri_str}?{params}");
    let api_request = Request::builder()
        .method(Method::GET)
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
    match serde_json::from_value(body_json.clone()) {
        Ok(body) => body,
        Err(e) => {
            dbg!(&body_json);
            panic!("response is not a valid rust struct: {e:?}");
        }
    }
}

pub async fn api_request_auth_json_body<Req: Serialize, Res: DeserializeOwned>(
    app: Router,
    method: Method,
    uri: impl AsRef<str>,
    auth_token: impl AsRef<str>,
    request: &Req,
) -> Res {
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

    // TODO(nick): handle cases where the sdf func returns the unit type. Perhaps the body will
    // be of length zero? Unsure and lot of potential foot guns.
    let body_json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(body_json) => body_json,
        Err(e) => {
            dbg!(&body);
            panic!(
                "response is not valid json (perhaps (de)serialization casing is the cause): {e:?}",
            );
        }
    };
    serde_json::from_value(body_json).expect("response is not a valid rust struct")
}

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
