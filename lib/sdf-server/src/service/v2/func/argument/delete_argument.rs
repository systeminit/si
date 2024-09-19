use axum::{
    extract::{Host, OriginalUri, Path},
    response::IntoResponse,
};
use dal::{
    func::{argument::FuncArgumentId, authoring::FuncAuthoringClient},
    ChangeSet, ChangeSetId, Func, FuncId, WorkspacePk, WsEvent,
};

use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::v2::func::FuncAPIResult,
    track,
};

pub async fn delete_func_argument(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, func_id, func_argument_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        FuncId,
        FuncArgumentId,
    )>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    FuncAuthoringClient::delete_func_argument(&ctx, func_argument_id).await?;

    let func_summary = Func::get_by_id_or_error(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?;
    WsEvent::func_updated(&ctx, func_summary.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "delete_func_argument",
        serde_json::json!({
            "how": "/func/delete_func_argument",
            "func_id": func_id,
            "func_name": func_summary.name.clone(),
            "func_kind": func_summary.kind.clone(),
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&func_summary)?)?)
}
