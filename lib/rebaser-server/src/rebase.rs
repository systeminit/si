use audit_logs_stream::AuditLogsStreamError;
use dal::{
    DalContext,
    TransactionsError,
    Workspace,
    WorkspaceError,
    WorkspacePk,
    WorkspaceSnapshot,
    WsEvent,
    WsEventError,
    billing_publish,
    change_set::{
        ChangeSet,
        ChangeSetError,
        ChangeSetId,
    },
    workspace_snapshot::{
        WorkspaceSnapshotError,
        selector::WorkspaceSnapshotSelectorDiscriminants,
        split_snapshot::SplitSnapshot,
    },
};
use edda_client::EddaClient;
use pending_events::PendingEventsError;
use rebaser_core::api_types::{
    enqueue_updates_request::EnqueueUpdatesRequest,
    enqueue_updates_response::v1::RebaseStatus,
};
use shuttle_server::ShuttleError;
use si_events::{
    RebaseBatchAddressKind,
    WorkspaceSnapshotAddress,
};
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time::Instant;
use tokio_util::task::TaskTracker;

use crate::Features;

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum RebaseError {
    #[error("audit logs stream error: {0}")]
    AuditLogsStream(#[from] AuditLogsStreamError),
    #[error("workspace snapshot error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("edda client error: {0}")]
    EddaClient(#[from] edda_client::ClientError),
    #[error("layerdb error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("missing rebase batch {0}")]
    MissingRebaseBatch(RebaseBatchAddressKind),
    #[error("pending events error: {0}")]
    PendingEvents(#[from] PendingEventsError),
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("shuttle error: {0}")]
    Shuttle(#[from] ShuttleError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("unexpected rebase batch address kind")]
    UnexpectedRebaseBatchAddressKind,
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace pk expected but was none")]
    WorkspacePkExpected,
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

type RebaseResult<T> = Result<T, RebaseError>;

#[instrument(
    name = "rebase.perform_rebase",
    level = "info",
    skip_all,
    fields(
        si.change_set.id = %request.change_set_id,
        si.conflicts = Empty,
        si.conflicts.count = Empty,
        si.updates = Empty,
        si.updates.count = Empty,
        si.corrected_updates.count = Empty,
        si.workspace.id = %request.workspace_id,
        si.edda_request.id = Empty,
        si.rebase.rebase_time = Empty,
        si.rebase.perform_updates_time = Empty,
        si.rebase.correct_transforms_time = Empty,
        si.rebase.snapshot_fetch_parse_time = Empty,
        si.rebase.pointer_updated = Empty
    ))]
pub(crate) async fn perform_rebase(
    ctx: &mut DalContext,
    edda: &EddaClient,
    request: &EnqueueUpdatesRequest,
    server_tracker: &TaskTracker,
    features: Features,
) -> RebaseResult<RebaseStatus> {
    let start = Instant::now();
    let workspace = get_workspace(ctx).await?;
    let updating_head = request.change_set_id == workspace.default_change_set_id();

    // Gather everything we need to detect conflicts and updates from the inbound message.
    let mut to_rebase_change_set = ChangeSet::get_by_id(ctx, request.change_set_id).await?;

    // if the change set isn't active, do not do this work
    if !to_rebase_change_set.status.is_active() {
        debug!("Attempted to rebase for abandoned change set. Early returning");
        return Ok(RebaseStatus::Error {
            message: "Attempted to rebase for an abandoned change set.".to_string(),
        });
    }

    let to_rebase_workspace_snapshot_address = to_rebase_change_set.workspace_snapshot_address;

    match workspace.snapshot_kind() {
        WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot => {
            rebase_legacy(
                ctx,
                to_rebase_workspace_snapshot_address,
                edda,
                request,
                features,
                updating_head,
                &mut to_rebase_change_set,
            )
            .await?;
        }
        WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot => {
            rebase_split(
                ctx,
                to_rebase_workspace_snapshot_address,
                edda,
                request,
                features,
                updating_head,
                &mut to_rebase_change_set,
            )
            .await?;
        }
    }

    {
        let ctx_clone = ctx.clone();
        server_tracker.spawn(async move {
            if let Err(err) =
                evict_unused_snapshots(&ctx_clone, &to_rebase_workspace_snapshot_address).await
            {
                error!(?err, "eviction error");
            }
            // TODO: RebaseBatch eviction?
        });
    }

    if updating_head && *workspace.pk() != WorkspacePk::NONE {
        //todo(brit): what do we want to do about change sets that haven't
        // been applied yet, but are approved? (like gh merge-queue)
        // should we 'unapprove' them?
        let all_open_change_sets = ChangeSet::list_active(ctx).await?;
        for target_change_set in all_open_change_sets.into_iter().filter(|cs| {
            cs.id != workspace.default_change_set_id()
                && cs.id != to_rebase_change_set.id
                && request.from_change_set_id != Some(cs.id)
        }) {
            let workspace_pk = *workspace.pk();
            let updates_address = request.updates_address;
            {
                let ctx_clone = ctx.clone();
                server_tracker.spawn(async move {
                    debug!(
                        "replaying batch {} onto {} from {}",
                        updates_address, target_change_set.id, to_rebase_change_set.id
                    );

                    if let Err(err) = replay_changes(
                        &ctx_clone,
                        workspace_pk,
                        target_change_set.id,
                        updates_address,
                        to_rebase_change_set.id,
                    )
                    .await
                    {
                        error!(
                            err = ?err,
                            "error replaying rebase batch {} changes onto {}",
                            updates_address,
                            target_change_set.id
                        );
                    }
                });
            }
        }
    }

    {
        if let Some(event_session_id) = request.event_session_id {
            let ctx_clone = ctx.clone();
            let server_tracker_clone = server_tracker.to_owned();
            server_tracker.spawn(async move {
                if let Err(err) = ctx_clone
                    .publish_pending_audit_logs(Some(server_tracker_clone), Some(event_session_id))
                    .await
                {
                    error!(?err, "failed to publish pending audit logs");
                }
            });
        }
    }

    if !updating_head {
        if let Some(source_change_set_id) = request.from_change_set_id {
            let mut event =
                WsEvent::change_set_applied(ctx, source_change_set_id, request.change_set_id, None)
                    .await?;
            event.set_workspace_pk(request.workspace_id);
            event.set_change_set_id(Some(request.change_set_id));
            event.publish_immediately(ctx).await?;
        }
    }

    debug!("rebase elapsed: {:?}", start.elapsed());

    Ok(RebaseStatus::Success {
        updates_performed: request.updates_address,
    })
}

async fn rebase_split(
    ctx: &mut DalContext,
    to_rebase_workspace_snapshot_address: WorkspaceSnapshotAddress,
    edda: &EddaClient,
    request: &EnqueueUpdatesRequest,
    _features: Features,
    updating_head: bool,
    to_rebase_change_set: &mut ChangeSet,
) -> RebaseResult<()> {
    let span = current_span_for_instrument_at!("info");

    let original_workspace_snapshot =
        SplitSnapshot::find(ctx, to_rebase_workspace_snapshot_address).await?;

    let to_rebase_workspace_snapshot =
        SplitSnapshot::find(ctx, to_rebase_workspace_snapshot_address).await?;

    let batch_address = match &request.updates_address {
        RebaseBatchAddressKind::Legacy(_) => {
            return Err(RebaseError::UnexpectedRebaseBatchAddressKind);
        }
        RebaseBatchAddressKind::Split(split_address) => split_address,
    };

    let rebase_batch = ctx
        .layer_db()
        .split_snapshot_rebase_batch()
        .read_wait_for_memory(batch_address)
        .await?
        .ok_or(RebaseError::MissingRebaseBatch(request.updates_address))?;

    debug!("rebase batch: {:?}", rebase_batch);

    let from_different_change_set = !updating_head
        && request
            .from_change_set_id
            .is_some_and(|from_id| from_id != to_rebase_change_set.id);

    let corrected_transforms = to_rebase_workspace_snapshot
        .correct_transforms(rebase_batch.to_vec(), from_different_change_set)
        .await?;

    to_rebase_workspace_snapshot
        .perform_updates(corrected_transforms.as_slice())
        .await?;

    let new_snapshot_address = to_rebase_workspace_snapshot.write(ctx).await?;
    debug!("Workspace snapshot updated to {}", new_snapshot_address);

    to_rebase_change_set
        .update_pointer(ctx, new_snapshot_address)
        .await?;

    if let Err(err) =
        billing_publish::for_head_change_set_pointer_update(ctx, to_rebase_change_set).await
    {
        error!(si.error.message = ?err, "Failed to publish billing for change set pointer update on HEAD");
    }

    ctx.set_workspace_split_snapshot(to_rebase_workspace_snapshot.clone());

    // Before replying to the requester or sending the Edda request, we must commit.
    ctx.commit_no_rebase().await?;

    send_updates_to_edda_split_snapshot(
        ctx,
        &original_workspace_snapshot,
        &to_rebase_workspace_snapshot,
        edda,
        request,
        span,
    )
    .await?;

    Ok(())
}

async fn rebase_legacy(
    ctx: &mut DalContext,
    to_rebase_workspace_snapshot_address: WorkspaceSnapshotAddress,
    edda: &EddaClient,
    request: &EnqueueUpdatesRequest,
    _features: Features,
    updating_head: bool,
    to_rebase_change_set: &mut ChangeSet,
) -> RebaseResult<()> {
    let span = current_span_for_instrument_at!("info");
    let start = Instant::now();

    let to_rebase_workspace_snapshot =
        WorkspaceSnapshot::find(ctx, to_rebase_workspace_snapshot_address).await?;
    // Rather than clone the above snapshot we want an independent copy of this snapshot
    let original_workspace_snapshot =
        WorkspaceSnapshot::find(ctx, to_rebase_workspace_snapshot_address).await?;

    let batch_address = match &request.updates_address {
        RebaseBatchAddressKind::Legacy(rebase_batch_address) => rebase_batch_address,
        RebaseBatchAddressKind::Split(_) => {
            return Err(RebaseError::UnexpectedRebaseBatchAddressKind);
        }
    };

    let rebase_batch = ctx
        .layer_db()
        .rebase_batch()
        .read_wait_for_memory(batch_address)
        .await?
        .ok_or(RebaseError::MissingRebaseBatch(request.updates_address))?;

    debug!(
        to_rebase_workspace_snapshot_address = %to_rebase_workspace_snapshot_address,
        updates_address = %request.updates_address,
    );
    debug!("after snapshot fetch and parse: {:?}", start.elapsed());
    span.record(
        "si.rebase.snapshot_fetch_parse_time",
        start.elapsed().as_millis(),
    );

    let corrected_updates = to_rebase_workspace_snapshot
        .correct_transforms(
            rebase_batch.updates().to_vec(),
            !updating_head
                && request
                    .from_change_set_id
                    .is_some_and(|from_id| from_id != to_rebase_change_set.id),
        )
        .await?;
    debug!("corrected transforms: {:?}", start.elapsed());
    span.record(
        "si.rebase.correct_transforms_time",
        start.elapsed().as_millis(),
    );

    to_rebase_workspace_snapshot
        .perform_updates(&corrected_updates)
        .await?;

    debug!("updates complete: {:?}", start.elapsed());

    if !corrected_updates.is_empty() {
        // Once all updates have been performed, we can write out, and update the pointer.
        to_rebase_workspace_snapshot.write(ctx).await?;
        debug!("snapshot written: {:?}", start.elapsed());
        to_rebase_change_set
            .update_pointer(ctx, to_rebase_workspace_snapshot.id().await)
            .await?;

        if let Err(err) =
            billing_publish::for_head_change_set_pointer_update(ctx, to_rebase_change_set).await
        {
            error!(si.error.message = ?err, "Failed to publish billing for change set pointer update on HEAD");
        }

        debug!("pointer updated: {:?}", start.elapsed());
        span.record("si.rebase.pointer_updated", start.elapsed().as_millis());

        ctx.set_workspace_snapshot(to_rebase_workspace_snapshot.clone());
    }

    let updates_count = rebase_batch.updates().len();
    span.record("si.updates.count", updates_count.to_string());
    span.record(
        "si.corrected_updates.count",
        corrected_updates.len().to_string(),
    );
    debug!("rebase performed: {:?}", start.elapsed());
    span.record("si.rebase.rebase_time", start.elapsed().as_millis());

    // Before replying to the requester or sending the Edda request, we must commit.
    ctx.commit_no_rebase().await?;

    send_updates_to_edda_legacy_snapshot(
        ctx,
        &original_workspace_snapshot,
        &to_rebase_workspace_snapshot,
        edda,
        request,
        span,
    )
    .await?;

    Ok(())
}

async fn send_updates_to_edda_split_snapshot(
    ctx: &DalContext,
    original_workspace_snapshot: &SplitSnapshot,
    to_rebase_workspace_snapshot: &SplitSnapshot,
    edda: &edda_client::Client,
    request: &EnqueueUpdatesRequest,
    span: Span,
) -> Result<(), RebaseError> {
    let changes = original_workspace_snapshot
        .detect_changes(to_rebase_workspace_snapshot)
        .instrument(tracing::info_span!(
            "rebaser.perform_rebase.detect_changes_for_edda_request"
        ))
        .await?;
    let change_batch_address = ctx.write_change_batch(changes).await?;
    let edda_update_request_id = edda
        .update_from_workspace_snapshot(
            request.workspace_id,
            request.change_set_id,
            original_workspace_snapshot.id().await,
            to_rebase_workspace_snapshot.id().await,
            change_batch_address,
        )
        .await?;
    span.record("si.edda_request.id", edda_update_request_id.to_string());
    Ok(())
}

async fn send_updates_to_edda_legacy_snapshot(
    ctx: &mut DalContext,
    original_workspace_snapshot: &WorkspaceSnapshot,
    to_rebase_workspace_snapshot: &WorkspaceSnapshot,
    edda: &edda_client::Client,
    request: &EnqueueUpdatesRequest,
    span: Span,
) -> Result<(), RebaseError> {
    let changes = original_workspace_snapshot
        .detect_changes(to_rebase_workspace_snapshot)
        .instrument(tracing::info_span!(
            "rebaser.perform_rebase.detect_changes_for_edda_request"
        ))
        .await?;
    let change_batch_address = ctx.write_change_batch(changes).await?;
    let edda_update_request_id = edda
        .update_from_workspace_snapshot(
            request.workspace_id,
            request.change_set_id,
            original_workspace_snapshot.id().await,
            to_rebase_workspace_snapshot.id().await,
            change_batch_address,
        )
        .await?;
    span.record("si.edda_request.id", edda_update_request_id.to_string());
    Ok(())
}

pub(crate) async fn evict_unused_snapshots(
    ctx: &DalContext,
    workspace_snapshot_address: &WorkspaceSnapshotAddress,
) -> RebaseResult<()> {
    if !ChangeSet::workspace_snapshot_address_in_use(ctx, workspace_snapshot_address).await? {
        ctx.layer_db().workspace_snapshot().evict(
            workspace_snapshot_address,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;
    }
    Ok(())
}

async fn replay_changes(
    ctx: &DalContext,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
    updates_address: RebaseBatchAddressKind,
    from_change_set_id: ChangeSetId,
) -> RebaseResult<()> {
    ctx.run_async_rebase_from_change_set(
        workspace_pk,
        change_set_id,
        updates_address,
        from_change_set_id,
    )
    .await?;

    Ok(())
}

async fn get_workspace(ctx: &DalContext) -> RebaseResult<Workspace> {
    let workspace_pk = ctx
        .tenancy()
        .workspace_pk_opt()
        .ok_or(RebaseError::WorkspacePkExpected)?;

    Ok(Workspace::get_by_pk(ctx, workspace_pk).await?)
}
