use super::{FuncArgumentView, FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    func::argument::{FuncArgument, FuncArgumentKind},
    Func, FuncId, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateArgumentRequest {
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
    pub name: String,
    pub func_id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type CreateArgumentResponse = FuncArgumentView;

pub async fn create_argument(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateArgumentRequest>,
) -> FuncResult<Json<CreateArgumentResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // Check existence of func id
    let _func = Func::get_by_id(&ctx, &request.func_id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    let existing =
        FuncArgument::find_by_name_for_func(&ctx, &request.name, request.func_id).await?;

    if let Some(existing) = existing {
        if existing.visibility().in_change_set() {
            return Err(FuncError::FuncArgumentAlreadyExists);
        }
    }

    let func_arg = FuncArgument::new(
        &ctx,
        &request.name,
        request.kind,
        request.element_kind,
        request.func_id,
    )
    .await?;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;
    ctx.commit().await?;

    Ok(Json(CreateArgumentResponse {
        id: func_arg.id().to_owned(),
        name: func_arg.name().to_owned(),
        kind: *func_arg.kind(),
        element_kind: func_arg.element_kind().cloned(),
    }))
}
