use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
};
use dal::{
    ChangeSet,
    ChangeSetId,
    Func,
    FuncId,
    WorkspacePk,
    WsEvent,
    func::authoring::FuncAuthoringClient,
};
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    FuncAPIResult,
    get_code_response,
};
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::AccessBuilder,
    },
    track,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCodeRequest {
    pub code: String,
}

pub async fn save_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<SaveCodeRequest>,
) -> FuncAPIResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncAuthoringClient::save_code(&ctx, func_id, request.code).await?;
    let func_code = get_code_response(&ctx, func_id).await?;
    let func = Func::get_by_id(&ctx, func_id).await?;
    WsEvent::func_code_saved(&ctx, func_code, false)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "save_func_code",
        serde_json::json!({
            "how": "/func/save_code",
            "func_id": func_id,
            "func_name": func.name.clone(),
            "func_kind": func.kind.clone(),
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
