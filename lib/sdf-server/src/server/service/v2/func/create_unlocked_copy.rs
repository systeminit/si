use axum::{
    extract::{OriginalUri, Path},
    response::IntoResponse,
    Json,
};
use dal::{
    func::authoring::FuncAuthoringClient, ChangeSet, ChangeSetId, FuncId, SchemaVariantId,
    WorkspacePk, WsEvent,
};

use serde::{Deserialize, Serialize};
use si_frontend_types::{FuncCode, FuncSummary};

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    tracking::track,
};

use super::{get_code_response, get_types, FuncAPIResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UnlockFuncRequest {
    pub schema_variant_id: Option<SchemaVariantId>,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncResponse {
    summary: FuncSummary,
    code: FuncCode,
}

pub async fn create_unlocked_copy(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<UnlockFuncRequest>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let new_func =
        FuncAuthoringClient::create_unlocked_func_copy(&ctx, func_id, request.schema_variant_id)
            .await?;
    let types = get_types(&ctx, new_func.id).await?;
    let code = get_code_response(&ctx, new_func.id).await?;
    let summary = new_func.into_frontend_type(&ctx).await?;

    WsEvent::func_created(&ctx, summary.clone(), types)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "unlocked_func",
        serde_json::json!({
            "how": "/func/unlocked_func",
            "func_id": summary.func_id,
            "func_name": summary.name.to_owned(),
            "func_kind": summary.kind,
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }

    Ok(response.body(serde_json::to_string(&CreateFuncResponse {
        summary,
        code,
    })?)?)
}
