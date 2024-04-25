use axum::{response::IntoResponse, Json};
use dal::func::authoring::FuncAuthoringClient;
use dal::func::view::FuncView;
use dal::{ChangeSet, Func};

use super::{save_func::SaveFuncRequest, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::func::SaveFuncResponse;

pub async fn save_and_exec(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SaveFuncRequest>,
) -> FuncResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncAuthoringClient::save_func(
        &ctx,
        request.id,
        request.display_name,
        request.name,
        request.description,
        request.code,
        request.associations,
    )
    .await?;

    FuncAuthoringClient::execute_func(&ctx, request.id).await?;

    let func = Func::get_by_id_or_error(&ctx, request.id).await?;
    let func_view = FuncView::assemble(&ctx, &func).await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&SaveFuncResponse {
        types: func_view.types,
        associations: func_view.associations,
    })?)?)
}
