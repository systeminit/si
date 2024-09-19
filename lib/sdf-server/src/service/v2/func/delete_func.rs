use axum::{
    extract::{Host, OriginalUri, Path},
    response::IntoResponse,
};
use dal::{func::binding::FuncBinding, ChangeSet, ChangeSetId, Func, FuncId, WorkspacePk, WsEvent};
use serde::{Deserialize, Serialize};

use super::{FuncAPIError, FuncAPIResult};
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFuncResponse {
    pub success: bool,
    pub name: String,
}

pub async fn delete_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let func = Func::get_by_id_or_error(&ctx, func_id).await?;
    if func.is_locked {
        return Err(FuncAPIError::CannotDeleteLockedFunc(func_id));
    }
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // first detach func from everywhere
    FuncBinding::delete_all_bindings_for_func_id(&ctx, func_id).await?;

    // then delete func
    let func_name = Func::delete_by_id(&ctx, func_id).await?;

    WsEvent::func_deleted(&ctx, func_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "deleted_func",
        serde_json::json!({
            "how": "/func/deleted_func",
            "func_id": func_id,
            "func_name": func.name.clone(),
            "func_kind": func.kind.clone(),
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&DeleteFuncResponse {
        success: true,
        name: func_name,
    })?)?)
}
