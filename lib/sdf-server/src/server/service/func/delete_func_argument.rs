use axum::{response::IntoResponse, Json};
use dal::func::argument::FuncArgumentId;
use dal::func::authoring::FuncAuthoringClient;
use dal::{ChangeSet, FuncId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFuncArgumentRequest {
    func_id: FuncId,
    func_argument_id: FuncArgumentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn delete_func_argument(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<DeleteFuncArgumentRequest>,
) -> FuncResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncAuthoringClient::delete_func_argument(&ctx, request.func_argument_id).await?;

    WsEvent::func_arguments_saved(&ctx, request.func_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
