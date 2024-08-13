use axum::{
    extract::{Host, OriginalUri, Path},
    response::IntoResponse,
};

use dal::{
    func::authoring::FuncAuthoringClient, ChangeSet, ChangeSetId, Func, FuncId, WorkspacePk,
};

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    tracking::track,
};

use super::FuncAPIResult;

pub async fn execute_func(
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
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncAuthoringClient::execute_func(&ctx, func_id).await?;
    let func = Func::get_by_id_or_error(&ctx, func_id).await?;

    // ws event?

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "execute_func",
        serde_json::json!({
            "how": "/func/execute_func",
            "func_id": func_id,
            "func_name": func.name.clone(),
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
