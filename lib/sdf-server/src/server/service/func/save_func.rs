use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::func::authoring::FuncAuthoringClient;
use dal::func::view::FuncView;
use dal::func::FuncAssociations;
use dal::{ChangeSet, Func, FuncId, Visibility};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::func::FuncResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFuncRequest {
    pub id: FuncId,
    pub display_name: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub associations: Option<FuncAssociations>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SaveFuncResponse {
    is_revertible: bool,
    types: String,
    associations: Option<FuncAssociations>,
}

pub async fn save_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<SaveFuncRequest>,
) -> FuncResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let request_id = request.id;

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

    let func = Func::get_by_id_or_error(&ctx, request_id).await?;
    let func_view = FuncView::assemble(&ctx, &func).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "save_func",
        serde_json::json!({
                    "func_id": func.id,
                    "func_name": func.name.as_str(),
                    "func_variant": func.backend_response_type,
                    "func_is_builtin": func.builtin,
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&SaveFuncResponse {
        is_revertible: func_view.is_revertible,
        types: func_view.types,
        associations: func_view.associations,
    })?)?)
}
