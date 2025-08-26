use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
    response::IntoResponse,
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
    change_set_mvs::create_index_for_new_change_set_and_watch,
    tracking::track,
};
use sdf_extract::{
    EddaClient as EddaClientExtractor,
    FriggStore,
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
use crate::StatusCode;

#[allow(clippy::too_many_arguments)]
pub async fn create_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    FriggStore(frigg): FriggStore,
    EddaClientExtractor(edda_client): EddaClientExtractor,
    Json(request): Json<CreateChangeSetRequest>,
) -> ChangeSetResult<impl IntoResponse> {
    let ctx = builder.build_head(access_builder).await?;

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

    let change_set = change_set.into_frontend_type(&ctx).await?;
    ctx.commit_no_rebase().await?;
    if create_index_for_new_change_set_and_watch(
        &frigg,
        &edda_client,
        ctx.workspace_pk()?,
        change_set.id,
        ctx.change_set_id(), // note DalCtx is built for Head here to create a change set!
        ctx.workspace_snapshot()?.address().await,
    )
    .await?
    {
        // Return 200 if the build succeeded in time
        Ok((StatusCode::OK, Json(CreateChangeSetResponse { change_set })))
    } else {
        // Return 202 Accepted with the same response body if the build didn't succeed in time
        // to let the caller know the create succeeded, we're just waiting on downstream work
        Ok((
            StatusCode::ACCEPTED,
            Json(CreateChangeSetResponse { change_set }),
        ))
    }
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
