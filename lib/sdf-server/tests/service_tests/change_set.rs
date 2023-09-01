use axum::{http::Method, Router};
use dal_test::{
    sdf_test, test_harness::create_change_set as dal_create_change_set, AuthTokenRef,
    DalContextHead,
};
use sdf_server::service::change_set::{
    apply_change_set::{ApplyChangeSetRequest, ApplyChangeSetResponse},
    create_change_set::{CreateChangeSetRequest, CreateChangeSetResponse},
    get_change_set::{GetChangeSetRequest, GetChangeSetResponse},
    list_open_change_sets::ListOpenChangeSetsResponse,
};

use crate::service_tests::{
    api_request_auth_empty, api_request_auth_json_body, api_request_auth_query,
};

#[sdf_test]
async fn list_open_change_sets(
    DalContextHead(ctx): DalContextHead,
    app: Router,
    AuthTokenRef(auth_token): AuthTokenRef<'_>,
) {
    let _a_change_set = dal_create_change_set(&ctx).await;
    let _b_change_set = dal_create_change_set(&ctx).await;
    ctx.commit().await.expect("cannot commit transaction");

    let response: ListOpenChangeSetsResponse = api_request_auth_empty(
        app,
        Method::GET,
        "/api/change_set/list_open_change_sets",
        auth_token,
    )
    .await;
    assert_eq!(response.len(), 2);
}

#[sdf_test]
async fn create_change_set(app: Router, AuthTokenRef(auth_token): AuthTokenRef<'_>) {
    let request: CreateChangeSetRequest = CreateChangeSetRequest {
        change_set_name: "mastodon".to_string(),
    };

    let response: CreateChangeSetResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/change_set/create_change_set",
        auth_token,
        &request,
    )
    .await;
    assert_eq!(&response.change_set.name, "mastodon");
}

#[sdf_test]
async fn get_change_set(
    DalContextHead(ctx): DalContextHead,
    app: Router,
    AuthTokenRef(auth_token): AuthTokenRef<'_>,
) {
    let change_set = dal_create_change_set(&ctx).await;
    ctx.commit().await.expect("cannot commit txn");
    let request = GetChangeSetRequest { pk: change_set.pk };

    let response: GetChangeSetResponse =
        api_request_auth_query(app, "/api/change_set/get_change_set", auth_token, &request).await;
    assert_eq!(&response.change_set, &change_set);
}

#[sdf_test]
async fn apply_change_set(
    DalContextHead(ctx): DalContextHead,
    app: Router,
    AuthTokenRef(auth_token): AuthTokenRef<'_>,
) {
    let change_set = dal_create_change_set(&ctx).await;
    ctx.commit().await.expect("cannot commit txn");
    let request = ApplyChangeSetRequest {
        change_set_pk: change_set.pk,
    };

    let _response: ApplyChangeSetResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/change_set/apply_change_set",
        auth_token,
        &request,
    )
    .await;
}
