use axum::{
    extract::{Host, OriginalUri, Path},
    response::Json,
};
use dal::{Workspace, WorkspaceError, WorkspacePk};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use super::AdminAPIResult;
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track_no_ctx,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentConcurrencyLimitRequest {
    pub concurrency_limit: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentConcurrencyLimitResponse {
    pub concurrency_limit: Option<i32>,
}

#[instrument(
    name = "admin.set_component_concurrency_limit",
    level = "info",
    skip_all,
    fields(
        si.workspace.id = %workspace_pk,
        si.workspace.concurrency_limit = Empty,
    ),
)]
pub async fn set_concurrency_limit(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path(workspace_pk): Path<WorkspacePk>,
    Json(request): Json<SetComponentConcurrencyLimitRequest>,
) -> AdminAPIResult<Json<SetComponentConcurrencyLimitResponse>> {
    let span = current_span_for_instrument_at!("info");

    span.record(
        "si.workspace.concurrency_limit",
        request
            .concurrency_limit
            .map(|limit| limit.to_string())
            .unwrap_or("default".to_string()),
    );

    let ctx = builder.build_head(access_builder).await?;

    let mut workspace = Workspace::get_by_pk(&ctx, &workspace_pk)
        .await?
        .ok_or(WorkspaceError::WorkspaceNotFound(workspace_pk))?;

    workspace
        .set_component_concurrency_limit(&ctx, request.concurrency_limit)
        .await?;

    ctx.commit_no_rebase().await?;

    track_no_ctx(
        &posthog_client,
        &original_uri,
        &host_name,
        ctx.history_actor().distinct_id(),
        Some(workspace_pk.to_string()),
        None,
        "admin.set_concurrency_limit",
        serde_json::json!({
            "concurrency_limit": workspace.raw_component_concurrency_limit(),
        }),
    );

    Ok(Json(SetComponentConcurrencyLimitResponse {
        concurrency_limit: workspace.raw_component_concurrency_limit(),
    }))
}
