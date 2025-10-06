use axum::{
    extract::{
        Host,
        OriginalUri,
        Path,
    },
    response::Json,
};
use dal::{
    Workspace,
    WorkspacePk,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Tenancy;
use telemetry::prelude::*;

use crate::{
    extract::PosthogClient,
    service::v2::admin::{
        AdminAPIResult,
        AdminUserContext,
    },
    track_no_ctx_workspace,
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
        si.workspace.id = %workspace_id,
        si.workspace.concurrency_limit = Empty,
    ),
)]
pub async fn set_concurrency_limit(
    AdminUserContext(mut ctx): AdminUserContext,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path(workspace_id): Path<WorkspacePk>,
    Json(request): Json<SetComponentConcurrencyLimitRequest>,
) -> AdminAPIResult<Json<SetComponentConcurrencyLimitResponse>> {
    ctx.update_tenancy(Tenancy::new(workspace_id));

    let span = current_span_for_instrument_at!("info");

    span.record(
        "si.workspace.concurrency_limit",
        request
            .concurrency_limit
            .map(|limit| limit.to_string())
            .unwrap_or("default".to_string()),
    );

    let mut workspace = Workspace::get_by_pk(&ctx, workspace_id).await?;

    workspace
        .set_component_concurrency_limit(&ctx, request.concurrency_limit)
        .await?;

    ctx.commit_no_rebase().await?;

    track_no_ctx_workspace(
        &posthog_client,
        &original_uri,
        &host_name,
        ctx.history_actor().distinct_id(),
        workspace_id,
        "admin.set_concurrency_limit",
        serde_json::json!({
            "concurrency_limit": workspace.raw_component_concurrency_limit(),
        }),
    );

    Ok(Json(SetComponentConcurrencyLimitResponse {
        concurrency_limit: workspace.raw_component_concurrency_limit(),
    }))
}
