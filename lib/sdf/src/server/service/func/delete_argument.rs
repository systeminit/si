use super::{FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    func::argument::{FuncArgument, FuncArgumentId},
    StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteArgumentRequest {
    pub id: FuncArgumentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteArgumentResponse {
    pub success: bool,
}

pub async fn delete_argument(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<DeleteArgumentRequest>,
) -> FuncResult<Json<DeleteArgumentResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let arg = FuncArgument::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncArgNotFound)?;
    arg.delete(&ctx).await?;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;
    ctx.commit().await?;

    Ok(Json(DeleteArgumentResponse { success: true }))
}
