use super::{FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    func::argument::{FuncArgument, FuncArgumentId, FuncArgumentKind},
    StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveArgumentRequest {
    pub id: FuncArgumentId,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
    pub name: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveArgumentResponse {
    pub success: bool,
}

pub async fn save_argument(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SaveArgumentRequest>,
) -> FuncResult<Json<SaveArgumentResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut arg = FuncArgument::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncArgNotFound)?;

    arg.set_name(&ctx, request.name).await?;
    arg.set_kind(&ctx, request.kind).await?;
    arg.set_element_kind(&ctx, request.element_kind).await?;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;
    ctx.commit().await?;

    Ok(Json(SaveArgumentResponse { success: true }))
}
