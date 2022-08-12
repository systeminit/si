use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    generate_name, Func, FuncBackendKind, FuncBackendResponseType, FuncId, StandardModel,
    Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncResponse {
    pub id: FuncId,
    pub handler: Option<String>,
    pub kind: FuncBackendKind,
    pub name: String,
    pub code: Option<String>,
}

static DEFAULT_QUALIFICATION_CODE: &str = "/*
* Your qualification function
* The signature should never be changed
*
* The input type is `Component`
* The return type is `Qualification`
*
* interface System {
*   name: string;
* }
*
* // The properties are derived from the fields in the Attributes panel
* interface Properties {
*   si: unknown;
*   domain: unknown
* }
*
* enum Kind {
*   Standard,
*   Credential
* }
*
* interface Data {
*   system: System | null;
*   kind: Kind;
*   properties: Properties;
* }
*
* interface Code {
*   format: string;
*   code: string | null;
* }
*
* interface Component {
*   data: Data;
*   parents: Component[]; // The parent's parents won't be available
*   codes: Code[];
* }
*
* interface Qualification {
*   qualified: boolean;
*   message: string;
* }
*/
async function qualification(component) {
  return {
    qualified: true,
    message: 'Component qualified'
  };
}";

pub async fn create_func(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateFuncRequest>,
) -> FuncResult<Json<CreateFuncResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let mut func = Func::new(
        &ctx,
        generate_name(None),
        FuncBackendKind::JsQualification,
        FuncBackendResponseType::Qualification,
    )
    .await?;

    func.set_code_plaintext(&ctx, Some(DEFAULT_QUALIFICATION_CODE))
        .await?;
    func.set_handler(&ctx, Some("qualification".to_owned()))
        .await?;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    txns.commit().await?;

    Ok(Json(CreateFuncResponse {
        id: func.id().to_owned(),
        handler: func.handler().map(|h| h.to_owned()),
        kind: func.backend_kind().to_owned(),
        name: func.name().to_owned(),
        code: func.code_plaintext()?,
    }))
}
