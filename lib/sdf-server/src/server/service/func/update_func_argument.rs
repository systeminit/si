use axum::{response::IntoResponse, Json};
use dal::func::argument::{FuncArgument, FuncArgumentId, FuncArgumentKind};
use dal::{ChangeSet, FuncId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFuncArgumentRequest {
    func_id: FuncId,
    func_argument_id: FuncArgumentId,
    name: String,
    kind: FuncArgumentKind,
    element_kind: Option<FuncArgumentKind>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn update_func_argument(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<UpdateFuncArgumentRequest>,
) -> FuncResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncArgument::modify_by_id(&ctx, request.func_argument_id, |existing_arg| {
        existing_arg.name = request.name;
        existing_arg.kind = request.kind;
        existing_arg.element_kind = request.element_kind;
        Ok(())
    })
    .await?;

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
