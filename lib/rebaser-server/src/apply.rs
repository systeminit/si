use std::time::Duration;

use dal::{
    ChangeSet,
    ChangeSetId,
    ChangeSetStatus,
    DalContext,
    TransactionsError,
    WorkspaceSnapshotAddress,
    WorkspaceSnapshotError,
    WsEvent,
    WsEventError,
    workspace_snapshot::{
        DependentValueRoot,
        dependent_value_root::DependentValueRootError,
        selector::WorkspaceSnapshotSelectorDiscriminants,
    },
};
use rebaser_core::api_types::enqueue_updates_request::{
    ApplyToHeadRequestV4,
    BeginApplyMode,
    BeginApplyToHeadRequestV4,
    RebaseRequestV4,
};
use si_events::RebaseBatchAddressKind;
use telemetry::prelude::*;
use thiserror::Error;
use tokio_util::task::TaskTracker;

#[derive(Debug, Error)]
pub(crate) enum ApplyError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("change set apply error: {0}")]
    ChangeSetApply(#[from] dal::ChangeSetApplyError),
    #[error("change set {0} not marked for apply: {1:?}")]
    ChangeSetNotMarkedForApply(ChangeSetId, ChangeSetStatus),
    #[error("change set {0} not open for apply: {1:?}")]
    ChangeSetNotOpenForApply(ChangeSetId, ChangeSetStatus),
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] DependentValueRootError),
    #[error("dependent value execution still in progress, try applying again when finished")]
    DependentValueRootExists(ChangeSetStatus),
    #[error("rebaser client error: {0}")]
    RebaserClient(#[from] rebaser_client::ClientError),
    #[error("transaction error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub(crate) type Result<T> = std::result::Result<T, ApplyError>;

#[instrument(name = "rebaser.begin_apply", level = "info", skip_all)]
pub(crate) async fn begin_apply(
    ctx: &mut DalContext,
    request: &BeginApplyToHeadRequestV4,
) -> Result<ChangeSetStatus> {
    let mut change_set = ChangeSet::get_by_id(ctx, request.change_set_id).await?;

    let previous_change_set_status = if change_set.status == ChangeSetStatus::ApplyStarted
        && request.previous_change_set_status.is_some()
    {
        request
            .previous_change_set_status
            .expect("we know it is some")
    } else {
        change_set.status.into()
    };

    if !change_set.status.is_active_or_applying() {
        return Err(ApplyError::ChangeSetNotOpenForApply(
            change_set.id,
            change_set.status,
        ));
    }

    ctx.update_snapshot_to_visibility().await?;

    // Fail safe here in case we have raced the DVU
    if DependentValueRoot::roots_exist(ctx).await? {
        return Err(ApplyError::DependentValueRootExists(
            previous_change_set_status.into(),
        ));
    }

    if request.mode == BeginApplyMode::LockSchemasAndFuncs {
        ChangeSet::lock_schemas_and_funcs_for_apply(ctx).await?;
    }

    // A rebase is not required here, since the rebaser by definition has the
    // latest version of the snapshot. Just lock the schemas/funcs and write it
    // out.
    let new_address = ctx.workspace_snapshot()?.write(ctx).await?;

    change_set.update_pointer(ctx, new_address).await?;
    change_set
        .update_status(ctx, ChangeSetStatus::ApplyStarted)
        .await?;
    ctx.commit_no_rebase().await?;

    ChangeSet::apply_to_base_change_set(
        ctx,
        previous_change_set_status.into(),
        request.head_change_set_address,
        Some(request.mode),
    )
    .await?;

    Ok(previous_change_set_status.into())
}

#[instrument(name = "rebaser.get_apply_rebase_request", level = "info", skip_all)]
pub(crate) async fn get_apply_rebase_request(
    ctx: &DalContext,
    request: ApplyToHeadRequestV4,
) -> Result<Option<RebaseRequestV4>> {
    let ApplyToHeadRequestV4 {
        workspace_id,
        head_change_set_id,
        change_set_to_apply_id,
        event_session_id,
        ..
    } = request;

    let change_set = ChangeSet::get_by_id(ctx, change_set_to_apply_id).await?;
    // Being marked as "ApplyStarted" means: schemas and funcs are locked, and
    // change set has no DVU roots.
    if change_set.status != ChangeSetStatus::ApplyStarted {
        return Err(ApplyError::ChangeSetNotMarkedForApply(
            change_set_to_apply_id,
            change_set.status,
        ));
    }

    let mut ctx = ctx.clone();
    ctx.update_visibility_and_snapshot_to_visibility(change_set_to_apply_id)
        .await?;

    let snapshot_kind: WorkspaceSnapshotSelectorDiscriminants = ctx.workspace_snapshot()?.into();

    let maybe_rebase_batch_address = match snapshot_kind {
        WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot => {
            if let Some(rebase_batch) = change_set
                .detect_updates_that_will_be_applied_legacy(&ctx)
                .await?
            {
                Some(RebaseBatchAddressKind::Legacy(
                    ctx.write_legacy_rebase_batch(rebase_batch).await?,
                ))
            } else {
                None
            }
        }
        WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot => {
            if let Some(rebase_batch) = change_set
                .detect_updates_that_will_be_applied_split(&ctx)
                .await?
            {
                Some(RebaseBatchAddressKind::Split(
                    ctx.write_split_snapshot_rebase_batch(rebase_batch).await?,
                ))
            } else {
                None
            }
        }
    };

    Ok(
        maybe_rebase_batch_address.map(|updates_address| RebaseRequestV4 {
            workspace_id,
            change_set_id: head_change_set_id,
            updates_address,
            from_change_set_id: Some(change_set_to_apply_id),
            event_session_id: Some(event_session_id),
        }),
    )
}

#[instrument(name = "rebaser.mark_change_set_applied", level = "info", skip_all)]
pub(crate) async fn mark_change_set_applied(
    ctx: &DalContext,
    change_set_id: ChangeSetId,
    head_change_set_id: ChangeSetId,
) -> Result<()> {
    let mut change_set = ChangeSet::get_by_id(ctx, change_set_id).await?;
    if change_set.status == ChangeSetStatus::Applied {
        return Ok(());
    }

    change_set
        .update_status(ctx, ChangeSetStatus::Applied)
        .await?;
    let user = ChangeSet::extract_userid_from_context(ctx).await;

    WsEvent::change_set_applied(ctx, change_set_id, head_change_set_id, user)
        .await?
        .publish_on_commit(ctx)
        .await?;

    ctx.commit_no_rebase().await?;

    Ok(())
}

#[instrument(name = "rebaser.enqueue_apply_retry", level = "info", skip(ctx))]
pub(crate) async fn enqueue_apply_retry(
    ctx: &DalContext,
    request: ApplyToHeadRequestV4,
    head_change_set_address: WorkspaceSnapshotAddress,
) -> Result<()> {
    ctx.rebaser()
        .enqueue_begin_apply(
            request.workspace_id,
            request.change_set_to_apply_id,
            request.head_change_set_id,
            head_change_set_address,
            Some(request.previous_change_set_status),
            request.event_session_id,
            request.mode.unwrap_or(BeginApplyMode::LockSchemasAndFuncs),
        )
        .await?;

    Ok(())
}

const DEFAULT_DELAY: Duration = Duration::from_millis(200);

/// Enqueues an apply retry with a delay. When we retry because the DVU roots
/// exist, we don't want to retry immediately, since this could lead to us
/// spamming the change set and HEAD request streams with retry messages as fast
/// as we can process them until the DVU is finished. Instead wait 200ms and
/// then enqueue, to throttle the rate of retries.
#[instrument(name = "rebaser.enqueue_apply_retry", level = "info", skip(ctx))]
pub(crate) async fn enqueue_apply_retry_with_delay(
    ctx: &DalContext,
    request: ApplyToHeadRequestV4,
    head_change_set_address: WorkspaceSnapshotAddress,
    server_tracker: TaskTracker,
) {
    let ctx = ctx.clone();
    server_tracker.spawn(async move {
        tokio::time::sleep(DEFAULT_DELAY).await;
        if let Err(err) = ctx
            .rebaser()
            .enqueue_begin_apply(
                request.workspace_id,
                request.change_set_to_apply_id,
                request.head_change_set_id,
                head_change_set_address,
                Some(request.previous_change_set_status),
                request.event_session_id,
                request.mode.unwrap_or(BeginApplyMode::LockSchemasAndFuncs),
            )
            .await
        {
            error!(
                "Failed to enqueue begin apply {:?} with delay: {}",
                request, err
            );
        }
    });
}
