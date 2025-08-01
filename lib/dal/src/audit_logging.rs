//! This module provides audit logging functionality to the rest of the crate.

use audit_database::{
    AuditDatabaseContext,
    AuditDatabaseError,
    AuditLogRow,
};
use audit_logs_stream::{
    AuditLogsStream,
    AuditLogsStreamError,
};
use pending_events::{
    PendingEventsError,
    PendingEventsStream,
};
use serde::{
    Deserialize,
    Serialize,
};
use shuttle_server::{
    Shuttle,
    ShuttleError,
};
use si_events::audit_log::{
    AuditLog,
    AuditLogKind,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio_util::task::TaskTracker;

use crate::{
    ChangeSet,
    ChangeSetError,
    ChangeSetStatus,
    DalContext,
    TransactionsError,
    WsEvent,
    WsEventResult,
    WsPayload,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AuditLoggingError {
    #[error("audit database error: {0}")]
    AuditDatabase(#[from] AuditDatabaseError),
    #[error("audit logs stream error: {0}")]
    AuditLogsStream(#[from] AuditLogsStreamError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] Box<ChangeSetError>),
    #[error("pending events error: {0}")]
    PendingEventsError(#[from] PendingEventsError),
    #[error("shuttle error: {0}")]
    Shuttle(#[from] ShuttleError),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] Box<TransactionsError>),
}

type Result<T> = std::result::Result<T, AuditLoggingError>;

/// Publishes all pending [`AuditLogs`](AuditLog) to the audit logs stream for the event session.
///
/// Provide the "override" [`EventSessionId`] if you'd like to use a different identifier than
/// the one on [`self`](DalContext).
///
/// _Warning: the subject for the event session must have a [final message](write_final_message)._
#[instrument(
    name = "audit_logging.publish_pending",
    level = "debug",
    skip_all,
    fields(override_event_session_id)
)]
pub(crate) async fn publish_pending(
    ctx: &DalContext,
    tracker: Option<TaskTracker>,
    override_event_session_id: Option<si_events::EventSessionId>,
) -> Result<()> {
    // TODO(nick): nuke this from intergalactic orbit. Then do it again.
    let workspace_id = match ctx.workspace_pk() {
        Ok(workspace_id) => workspace_id,
        Err(TransactionsError::SiDb(si_db_err))
            if matches!(si_db_err.as_ref(), si_db::Error::NoWorkspace) =>
        {
            return Ok(());
        }
        Err(err) => return Err(AuditLoggingError::Transactions(Box::new(err))),
    };

    let (tracker, provided_tracker) = match tracker {
        Some(provided_tracker) => (provided_tracker, false),
        None => (TaskTracker::new(), true),
    };

    // Get a handle on the source and destination streams.
    let source_stream = PendingEventsStream::get_or_create(ctx.jetstream_context()).await?;
    let destination_stream = AuditLogsStream::get_or_create(ctx.jetstream_context()).await?;

    // Create a shuttle instance for shuttling audit logs from the pending events stream.
    let audit_logs_shuttle = Shuttle::new(
        ctx.nats_conn().to_owned(),
        source_stream.stream().await?,
        source_stream.subject_for_audit_log(
            workspace_id,
            ctx.change_set_id(),
            match override_event_session_id {
                Some(override_id) => override_id,
                None => ctx.event_session_id(),
            },
        ),
        destination_stream.publishing_subject_for_workspace(workspace_id),
    )
    .await?;

    // Run the audit logs shuttle instance. If a tracker has been provided, we can spawn the
    // shuttle instance using it. If we are using a tracker purely within this function, we cannot
    // reliably use it to run the shuttle instance, so we will close and wait once shuttle exits.
    let ctx_clone_for_ws_event = ctx.clone();
    if provided_tracker {
        tracker.spawn(async move {
            if let Err(err) = audit_logs_shuttle.try_run().await {
                error!(?err, "audit logs shuttle error");
            }

            match ChangeSet::find(
                &ctx_clone_for_ws_event,
                ctx_clone_for_ws_event.change_set_id(),
            )
            .await
            {
                Ok(Some(change_set)) => {
                    match WsEvent::audit_logs_published(
                        &ctx_clone_for_ws_event,
                        change_set.id,
                        change_set.status,
                    )
                    .await
                    {
                        Ok(event) => {
                            if let Err(err) =
                                event.publish_immediately(&ctx_clone_for_ws_event).await
                            {
                                error!(?err, "error when publishing ws event for audit logs");
                            }
                        }
                        Err(err) => error!(?err, "error when creating ws event for audit logs"),
                    }
                }
                Ok(None) => {
                    trace!("skipping ws event creation for audit logs: no change set found")
                }
                Err(err) => error!(
                    ?err,
                    "error when attempting to find change set for ws event for audit logs"
                ),
            }
        });
    } else {
        // TODO(nick): this needs a tracker. In fact, func runner does too. We'll need a long term
        // solution for spwaning tasks in the dal.
        tokio::spawn(async move {
            if let Err(err) = audit_logs_shuttle.try_run().await {
                error!(?err, "audit logs shuttle error");
            }

            tracker.close();
            tracker.wait().await;

            match ChangeSet::find(
                &ctx_clone_for_ws_event,
                ctx_clone_for_ws_event.change_set_id(),
            )
            .await
            {
                Ok(Some(change_set)) => {
                    match WsEvent::audit_logs_published(
                        &ctx_clone_for_ws_event,
                        change_set.id,
                        change_set.status,
                    )
                    .await
                    {
                        Ok(event) => {
                            if let Err(err) =
                                event.publish_immediately(&ctx_clone_for_ws_event).await
                            {
                                error!(?err, "error when publishing ws event for audit logs");
                            }
                        }
                        Err(err) => error!(?err, "error when creating ws event for audit logs"),
                    }
                }
                Ok(None) => {
                    trace!("skipping ws event creation for audit logs: no change set found")
                }
                Err(err) => error!(
                    ?err,
                    "error when attempting to find change set for ws event for audit logs"
                ),
            }
        });
    }

    Ok(())
}

#[instrument(
    name = "audit_logging.write",
    level = "debug",
    skip_all,
    fields(kind, entity_name, override_destination_change_set_id)
)]
pub(crate) async fn write(
    ctx: &DalContext,
    kind: AuditLogKind,
    entity_name: String,
    override_destination_change_set_id: Option<si_events::ChangeSetId>,
) -> Result<()> {
    // TODO(nick): nuke this from intergalactic orbit. Then do it again.
    let workspace_id = match ctx.workspace_pk() {
        Ok(workspace_id) => workspace_id,
        Err(TransactionsError::SiDb(si_db_err))
            if matches!(si_db_err.as_ref(), si_db::Error::NoWorkspace) =>
        {
            return Ok(());
        }
        Err(err) => return Err(AuditLoggingError::Transactions(Box::new(err))),
    };

    let destination_change_set_id =
        override_destination_change_set_id.unwrap_or(ctx.change_set_id());

    let pending_events_stream = PendingEventsStream::get_or_create(ctx.jetstream_context()).await?;
    pending_events_stream
        .publish_audit_log(
            workspace_id,
            ctx.change_set_id(),
            ctx.event_session_id(),
            &AuditLog::new(
                ctx.events_actor(),
                kind,
                entity_name,
                destination_change_set_id,
                ctx.authentication_method(),
            ),
            destination_change_set_id,
        )
        .await?;
    Ok(())
}

#[instrument(name = "audit_logging.write_final_message", level = "debug", skip_all)]
pub(crate) async fn write_final_message(ctx: &DalContext) -> Result<()> {
    // TODO(nick): nuke this from intergalactic orbit. Then do it again.
    let workspace_id = match ctx.workspace_pk() {
        Ok(workspace_id) => workspace_id,
        Err(TransactionsError::SiDb(si_db_err))
            if matches!(si_db_err.as_ref(), si_db::Error::NoWorkspace) =>
        {
            return Ok(());
        }
        Err(err) => return Err(AuditLoggingError::Transactions(Box::new(err))),
    };

    let pending_events_stream = PendingEventsStream::get_or_create(ctx.jetstream_context()).await?;
    pending_events_stream
        .publish_audit_log_final_message(workspace_id, ctx.change_set_id(), ctx.event_session_id())
        .await?;
    Ok(())
}

#[instrument(name = "audit_logging.list", level = "debug", skip_all, fields(size))]
pub async fn list(
    ctx: &DalContext,
    audit_database_context: &AuditDatabaseContext,
    size: usize,
    sort_ascending: bool,
) -> Result<(Vec<AuditLogRow>, bool)> {
    let workspace_id = ctx.workspace_pk().map_err(Box::new)?;
    let change_set_id = ctx.change_set_id();

    let change_set_ids = {
        let mut change_set_ids = vec![change_set_id];
        if ctx
            .get_workspace_default_change_set_id()
            .await
            .map_err(Box::new)?
            == change_set_id
        {
            // NOTE(nick,fletcher,brit,paul): we need to decide what this entails on HEAD in the long term. For now,
            // it is all non-open, non-abandoned change sets... which are just the applied ones. In the future, we may
            // or will need to ability to tell a story about abandoned change sets. This is for future us or future
            // victims to solve. Good luck!
            for applied_change_set in ChangeSet::list_all_applied(ctx, workspace_id)
                .await
                .map_err(Box::new)?
            {
                change_set_ids.push(applied_change_set.id);
            }
        }
        change_set_ids
    };

    Ok(AuditLogRow::list(
        audit_database_context,
        workspace_id,
        change_set_ids,
        size,
        sort_ascending,
    )
    .await?)
}

#[instrument(name = "audit_logging.list_for_component", level = "debug", skip_all, fields(size, component_id))]
pub async fn list_for_component(
    ctx: &DalContext,
    audit_database_context: &AuditDatabaseContext,
    component_id: crate::ComponentId,
    size: usize,
    sort_ascending: bool,
) -> Result<(Vec<AuditLogRow>, bool)> {
    let workspace_id = ctx.workspace_pk().map_err(Box::new)?;
    let change_set_id = ctx.change_set_id();

    let change_set_ids = {
        let mut change_set_ids = vec![change_set_id];
        if ctx
            .get_workspace_default_change_set_id()
            .await
            .map_err(Box::new)?
            == change_set_id
        {
            for applied_change_set in ChangeSet::list_all_applied(ctx, workspace_id)
                .await
                .map_err(Box::new)?
            {
                change_set_ids.push(applied_change_set.id);
            }
        }
        change_set_ids
    };

    Ok(AuditLogRow::list_for_component(
        audit_database_context,
        workspace_id,
        change_set_ids,
        component_id,
        size,
        sort_ascending,
    )
    .await?)
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogsPublishedPayload {
    change_set_id: crate::ChangeSetId,
    change_set_status: ChangeSetStatus,
}

impl WsEvent {
    pub async fn audit_logs_published(
        ctx: &DalContext,
        change_set_id: crate::ChangeSetId,
        change_set_status: ChangeSetStatus,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::AuditLogsPublished(AuditLogsPublishedPayload {
                change_set_id,
                change_set_status,
            }),
        )
        .await
    }
}
