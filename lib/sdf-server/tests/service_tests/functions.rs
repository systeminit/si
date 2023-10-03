use axum::{http::Method, Router};

use dal::{Func, FuncBackendKind, FuncBackendResponseType, StandardModel};
use dal_test::{sdf_test, AuthTokenRef, DalContextHead};

use sdf_server::service::func::execute::{ExecuteRequest, ExecuteResponse};

use crate::service_tests::api_request_auth_json_body;

#[sdf_test]
async fn test_execution_endpoint_qualification_function(
    DalContextHead(ctx): DalContextHead,
    app: Router,
    AuthTokenRef(auth_token): AuthTokenRef<'_>,
) {
    let mut func = Func::new(
        &ctx,
        "keebiscool",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Qualification,
    )
    .await
    .expect("cannot create new function");
    func.set_code_plaintext(
        &ctx,
        Some(
            "async function qualification(component: Input): Promise < Output > {

        return {
            result: 'success',
            message: component.properties.some
        };
    }",
        ),
    )
    .await
    .expect("unable to set code plaintext");
    func.set_handler(&ctx, Some("qualification".to_string()))
        .await
        .expect("unable to set entrypoint");

    ctx.commit().await.expect("cannot commit");

    let request = ExecuteRequest {
        id: *func.id(),
        args: serde_json::json!({"properties": {"some" : "info"}}),
        execution_key: "somethingfun".to_string(),
        visibility: *ctx.visibility(),
    };

    let response: ExecuteResponse =
        api_request_auth_json_body(app, Method::POST, "/api/func/execute", auth_token, &request)
            .await;

    assert_eq!(
        response.output,
        serde_json::json!({"result": "success", "message": "info"})
    );
}
