use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::func::authoring::CreateFuncOptions;
use dal::func::{authoring, FuncKind};
use dal::{ChangeSet, Visibility};
use serde::{Deserialize, Serialize};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::func::FuncError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncRequest {
    kind: FuncKind,
    name: Option<String>,
    options: Option<CreateFuncOptions>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn create_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateFuncRequest>,
) -> FuncResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    if let Some(name) = request.name.as_deref() {
        if dal::func::is_intrinsic(name)
            || ["si:resourcePayloadToValue", "si:normalizeToArray"].contains(&name)
        {
            return Err(FuncError::FuncNameReserved(name.into()));
        }
    }

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let created_func =
        authoring::create_func(&ctx, request.kind, request.name, request.options).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "created_func",
        serde_json::json!({
                    "func_id": created_func.id,
                    "func_handler": created_func.handler.as_ref().map(|h| h.to_owned()),
                    "func_name": created_func.name.to_owned(),
                    "func_kind": created_func.kind,
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&created_func)?)?)
}
