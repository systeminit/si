use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::{
    ChangeSetId,
    WorkspacePk,
    WorkspaceSnapshotAddress,
    WsEvent,
    change_set::ChangeSet,
};
use sdf_core::{
    EddaClient,
    tracking::track,
};
use sdf_extract::{
    EddaClient as EddaClientExtractor,
    HandlerContext,
    PosthogClient,
    v1::AccessBuilder,
};
use si_events::audit_log::AuditLogKind;
use si_frontend_types::{
    CreateChangeSetRequest,
    CreateChangeSetResponse,
};
use telemetry::prelude::*;

use super::ChangeSetResult;

pub async fn create_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    EddaClientExtractor(edda_client): EddaClientExtractor,
    Json(request): Json<CreateChangeSetRequest>,
) -> ChangeSetResult<Json<CreateChangeSetResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let workspace_pk = ctx.workspace_pk()?;

    let change_set_name = &request.change_set_name;

    let change_set = ChangeSet::fork_head(&ctx, change_set_name).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "create_change_set",
        serde_json::json!({
                    "change_set_name": change_set_name.clone(),
        }),
    );

    ctx.write_audit_log(AuditLogKind::CreateChangeSet, change_set_name.to_string())
        .await?;

    WsEvent::change_set_created(&ctx, change_set.id, change_set.workspace_snapshot_address)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    create_index_for_new_change_set(
        &edda_client,
        workspace_pk,
        change_set.id,
        ctx.change_set_id(),
        change_set.workspace_snapshot_address,
    )
    .await?;

    let change_set = change_set.into_frontend_type(&ctx).await?;
    ctx.commit_no_rebase().await?;

    Ok(Json(CreateChangeSetResponse { change_set }))
}

#[instrument(
    level = "info",
    name = "sdf.change_set.v1.create_index_for_new_change_set",
    skip_all,
    fields(
        si.edda_request.id = Empty
    )
)]
pub async fn create_index_for_new_change_set(
    edda_client: &EddaClient,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
    base_change_set_id: ChangeSetId,
    to_snapshot_address: WorkspaceSnapshotAddress,
) -> ChangeSetResult<()> {
    let span = Span::current();
    let request_id = edda_client
        .new_change_set(
            workspace_pk,
            change_set_id,
            base_change_set_id,
            to_snapshot_address,
        )
        .await?;
    span.record("si.edda_request.id", request_id.to_string());

    Ok(())
}
