use axum::{
    extract::{OriginalUri, Path},
    response::IntoResponse,
    Json,
};
use dal::{
    func::authoring::FuncAuthoringClient, ChangeSet, ChangeSetId, FuncId, WorkspacePk, WsEvent,
};

use serde::{Deserialize, Serialize};

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    tracking::track,
};

use super::FuncAPIResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFuncRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
}

pub async fn update_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<UpdateFuncRequest>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let updated_func =
        FuncAuthoringClient::update_func(&ctx, func_id, request.display_name, request.description)
            .await?
            .into_frontend_type(&ctx)
            .await?;

    WsEvent::func_updated(&ctx, updated_func.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "update_func",
        serde_json::json!({
            "how": "/func/update_binding",
            "func_id": func_id,
            "func_name": updated_func.name.clone(),
            "func_kind": updated_func.kind.clone(),
        }),
    );
    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&updated_func)?)?)
}
