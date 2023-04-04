use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::{http, Router};
use serde::de::DeserializeOwned;
use serde::Serialize;
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

#[macro_export]
macro_rules! test_setup {
    (
        $ctx:ident,
        $jwt_secret_key:ident,
        $pg:ident,
        $pgconn:ident,
        $pgtxn:ident,
        $nats_conn:ident,
        $nats:ident,
        $veritech:ident,
        $encr_key:ident,
        $app:ident,
        $nw:ident,
        $auth_token:ident,
        $dal_ctx:ident,
        $job_processor:ident,
        $council_subject_prefix:ident $(,)?
    ,
    ) => {
        ::dal_test::test_harness::one_time_setup()
            .await
            .expect("one time setup failed");
        let $ctx = ::dal_test::test_harness::TestContext::init().await;
        let (
            $pg,
            $nats_conn,
            $job_processor,
            $veritech,
            $encr_key,
            $jwt_secret_key,
            $council_subject_prefix,
        ) = $ctx.entries();
        let $nats = $nats_conn.transaction();
        let mut $pgconn = $pg.get().await.expect("cannot connect to pg");
        let $pgtxn = $pgconn.transaction().await.expect("cannot create txn");
        let (posthog_client, posthog_sender) = si_posthog_rs::new()
            .api_endpoint("http://localhost:9999")
            .api_key("poop")
            .enabled(false)
            .build()
            .expect("cannot create posthog client and sender");
        tokio::spawn(posthog_sender.run());

        let ($app, _, _) = sdf_server::build_service(
            $pg.clone(),
            $nats_conn.clone(),
            $job_processor.clone(),
            $veritech.clone(),
            $encr_key.clone(),
            $jwt_secret_key.clone(),
            "myunusedsignupsecret".into(),
            $council_subject_prefix.to_owned(),
            posthog_client,
            None,
        )
        .expect("cannot build new server");
        let $app: ::axum::Router = $app.into();

        let ($nw, $auth_token) = {
            let services_context = ::dal::ServicesContext::new(
                $pg.clone(),
                $nats_conn.clone(),
                $job_processor.clone(),
                $veritech.clone(),
                std::sync::Arc::new($encr_key.clone()),
                $council_subject_prefix.to_owned(),
                None,
            );
            let builder = services_context.into_builder();
            let mut ctx = builder
                .build(::dal::RequestContext::default())
                .await
                .expect("cannot start transactions");

            let ($nw, $auth_token) =
                ::dal_test::test_harness::workspace_signup(&mut ctx, &$jwt_secret_key).await;

            ctx.commit().await.expect("cannot finish setup");

            ($nw, $auth_token)
        };

        let services_context = dal::ServicesContext::new(
            $pg.clone(),
            $nats_conn.clone(),
            $job_processor.clone(),
            $veritech.clone(),
            std::sync::Arc::new($encr_key.clone()),
            $council_subject_prefix.to_owned(),
            None,
        );
        let builder = services_context.into_builder();

        // This macro expands into funcitons that might not need to mutate,
        // triggering a lint warning
        #[allow(unused_mut)]
        let mut $dal_ctx = {
            let mut ctx = builder
                .build_default()
                .await
                .expect("cannot start transactions");

            let visibility = ::dal::Visibility::new_head(false);

            ctx.update_tenancy(::dal::Tenancy::new(*$nw.workspace.pk()));
            ctx.update_visibility(visibility);
            ctx.update_history_actor(::dal::HistoryActor::SystemInit);
            ctx
        };
    };
}
