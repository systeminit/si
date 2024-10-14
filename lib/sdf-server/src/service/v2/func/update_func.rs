use super::FuncAPIResult;
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};
use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::{
    func::authoring::FuncAuthoringClient, ChangeSet, ChangeSetId, FuncId, WorkspacePk, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_frontend_types::FuncSummary;
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFuncRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    client_ulid: Ulid,
}

pub async fn update_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<UpdateFuncRequest>,
) -> FuncAPIResult<ForceChangeSetResponse<FuncSummary>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let updated_func =
        FuncAuthoringClient::update_func(&ctx, func_id, request.display_name, request.description)
            .await?
            .into_frontend_type(&ctx)
            .await?;

    WsEvent::func_updated(&ctx, updated_func.clone(), Some(request.client_ulid))
        .await?
        .publish_on_commit(&ctx)
        .await?;
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "update_func",
        serde_json::json!({
            "how": "/func/update_binding",
            "func_id": func_id,
            "func_name": updated_func.name.clone(),
            "func_kind": updated_func.kind.clone(),
        }),
    );
    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        updated_func,
    ))
}
