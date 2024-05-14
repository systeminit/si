use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::func::authoring::FuncAuthoringClient;
use dal::{ChangeSet, WsEvent};

use super::{save_func::SaveFuncRequest, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

pub async fn save_and_exec(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<SaveFuncRequest>,
) -> FuncResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // Cache for posthog tracking.
    let func_id = request.id;
    let func_name = request.name.clone();

    FuncAuthoringClient::save_func(
        &ctx,
        func_id,
        request.display_name,
        request.name,
        request.description,
        request.code,
        request.associations,
    )
    .await?;

    FuncAuthoringClient::execute_func(&ctx, func_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "save_and_exec",
        serde_json::json!({
            "how": "/func/save_and_exec",
            "func_id": func_id,
            "func_name": func_name.as_str(),
        }),
    );

    WsEvent::func_saved(&ctx, func_id)
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
