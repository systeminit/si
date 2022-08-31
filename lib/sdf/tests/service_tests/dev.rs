use dal::{StandardModel, SystemId, Visibility};
use sdf::service::dev::test::{TestRequest, TestResponse};

use crate::dal::test;
use crate::service_tests::api_request_auth_query;
use crate::test_setup;

#[test]
async fn test() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        _conn,
        _txn,
        _nats_conn,
        _nats,
        veritech,
        encr_key,
        app,
        nba,
        auth_token,
        _dal_ctx,
        dal_txns,
        _faktory,
    );

    let request = TestRequest {
        visibility: Visibility::new_head(false),
        workspace_id: *nba.workspace.id(),
        system_id: Some(SystemId::NONE),
    };
    let response: TestResponse =
        api_request_auth_query(app, "/api/dev/test", &auth_token, &request).await;
    assert!(response.success);
}
