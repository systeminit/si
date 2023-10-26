use axum::{http::Method, Router};
use dal::WorkspaceSignup;
use dal_test::{sdf_test, AuthTokenRef, DalContextHead};
use sdf_server::service::session::{
    load_workspace::LoadWorkspaceResponse, restore_authentication::RestoreAuthenticationResponse,
};

use crate::service_tests::api_request_auth_empty;

#[sdf_test]
async fn restore_authentication(
    DalContextHead(ctx): DalContextHead,
    app: Router,
    AuthTokenRef(auth_token): AuthTokenRef<'_>,
    nw: WorkspaceSignup,
) {
    // TODO(fnichol): the `dal_test::helpers::workspace_signup` prep logic in `#[sdf_test]` is too
    // naive and assumes that a `DalContext` will be requested in the test. Otherwise the workspace
    // creation is hanging out on a txn which gets rolled back if we don't consume-and-use the txn
    // in a DalContext for this test. I don't like this, because it makes you wonder why we
    // immeditatly commit here, and I don't want to promote this pattern. For the moment though,
    // I'm going to let this play as-is.
    ctx.commit().await.expect("failed to commit");

    let response: RestoreAuthenticationResponse = api_request_auth_empty(
        app,
        Method::GET,
        "/api/session/restore_authentication",
        auth_token,
    )
    .await;
    assert_eq!(nw.workspace, response.workspace);
    assert_eq!(nw.user, response.user);
}

#[sdf_test]
async fn load_workspace(
    DalContextHead(ctx): DalContextHead,
    app: Router,
    AuthTokenRef(auth_token): AuthTokenRef<'_>,
    nw: WorkspaceSignup,
) {
    // TODO(fnichol): the `dal_test::helpers::workspace_signup` prep logic in `#[sdf_test]` is too
    // naive and assumes that a `DalContext` will be requested in the test. Otherwise the workspace
    // creation is hanging out on a txn which gets rolled back if we don't consume-and-use the txn
    // in a DalContext for this test. I don't like this, because it makes you wonder why we
    // immeditatly commit here, and I don't want to promote this pattern. For the moment though,
    // I'm going to let this play as-is.
    ctx.commit().await.expect("failed to commit");

    let response: LoadWorkspaceResponse =
        api_request_auth_empty(app, Method::GET, "/api/session/load_workspace", auth_token).await;
    assert_eq!(nw.workspace, response.workspace);
}
