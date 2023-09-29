use axum::{
    Router, http::Method 
};

use dal::{StandardModel, Func, FuncBackendKind, FuncBackendResponseType, };
use dal_test::{
    sdf_test,
    AuthTokenRef, DalContextHead,
};

use dal::func::backend::js_attribute::FuncBackendJsAttributeArgs;

use veritech_client::{
    ResolverFunctionComponent, ResolverFunctionResponseType, ComponentView, ComponentKind
};

use sdf_server::service::func::execute::{
    ExecuteRequest, ExecuteResponse 
}; 

use crate::service_tests::api_request_auth_json_body;

#[sdf_test]
async fn test_execution_endpoint_qualification_function(
    DalContextHead(ctx): DalContextHead,
    app: Router,
    AuthTokenRef(auth_token): AuthTokenRef<'_>,
) {
    let mut func = Func::new(&ctx, "keebiscool", FuncBackendKind::JsAttribute, FuncBackendResponseType::Qualification).await.expect("cannot create new function");
    func.set_code_plaintext(&ctx, Some("async function qualification(component: Input): Promise < Output > {

        return {
            result: 'success',
            message: component.properties.some
        };
    }")).await.expect("unable to set code plaintext");
    func.set_handler(&ctx, Some("qualification".to_string())).await.expect("unable to set entrypoint");
    
    ctx.commit().await.expect("cannot commit");

    let request = ExecuteRequest {
        id: *func.id(), 
        args: serde_json::to_value(FuncBackendJsAttributeArgs{
            component: ResolverFunctionComponent {
                data: ComponentView {
                    kind: ComponentKind::Standard,
                    properties: serde_json::json!({"some" : "info"})
                },
                parents: Vec::new()
            },
            response_type: ResolverFunctionResponseType::Qualification
        }).expect("unable to serialize the arguments"),
        execution_key: "somethingfun".to_string(),
        visibility: *ctx.visibility()
    };

    let response: ExecuteResponse = api_request_auth_json_body(
        app,
        Method::POST,
        "/api/func/execute",
        auth_token,
        &request,
    )
    .await;

    assert_eq!(response.output, serde_json::json!({"result": "success", "message": "info"}));


}
