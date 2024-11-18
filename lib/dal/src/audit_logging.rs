//! This module provides audit logging functionality to the rest of the crate.

use std::collections::HashMap;

use audit_logs::AuditLogsStream;
use audit_logs::AuditLogsStreamError;
use futures::StreamExt;
use pending_events::PendingEventsError;
use pending_events::PendingEventsStream;
use serde::Deserialize;
use serde::Serialize;
use shuttle_server::Shuttle;
use shuttle_server::ShuttleError;
use si_data_nats::async_nats;
use si_data_nats::async_nats::jetstream::consumer::pull::BatchErrorKind;
use si_data_nats::async_nats::jetstream::context::RequestErrorKind;
use si_data_nats::async_nats::jetstream::stream::ConsumerErrorKind;
use si_events::audit_log::AuditLog;
use si_events::audit_log::AuditLogKind;
use si_events::Actor;
use si_frontend_types::AuditLog as FrontendAuditLog;
use si_frontend_types::AuditLogDeserializedMetadata as FrontendAuditLogDeserializedMetadata;
use telemetry::prelude::*;
use thiserror::Error;
use tokio_util::task::TaskTracker;

use crate::ChangeSet;
use crate::ChangeSetError;
use crate::ChangeSetStatus;
use crate::DalContext;
use crate::TenancyError;
use crate::TransactionsError;
use crate::User;
use crate::UserError;
use crate::WsEvent;
use crate::WsEventResult;
use crate::WsPayload;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AuditLoggingError {
    #[error("async nats batch error: {0}")]
    AsyncNatsBatch(#[from] async_nats::error::Error<BatchErrorKind>),
    #[error("async nats consumer error: {0}")]
    AsyncNatsConsumer(#[from] async_nats::error::Error<ConsumerErrorKind>),
    #[error("async nats request error: {0}")]
    AsyncNatsRequest(#[from] async_nats::error::Error<RequestErrorKind>),
    #[error("audit logs stream error: {0}")]
    AuditLogsStream(#[from] AuditLogsStreamError),
    #[error("cannot return list of unbounded size: both page ({0}) and page size ({1})")]
    CannotReturnListOfUnboundedSize(usize, usize),
    #[error("change set error: {0}")]
    ChangeSet(#[from] Box<ChangeSetError>),
    #[error("change set not found by id: {0}")]
    ChangeSetNotFound(si_events::ChangeSetId),
    #[error("message error: {0}")]
    Message(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("pending events error: {0}")]
    PendingEventsError(#[from] PendingEventsError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("shuttle error: {0}")]
    Shuttle(#[from] ShuttleError),
    #[error("transactions error: {0}")]
    Transactions(#[from] Box<TransactionsError>),
    #[error("user error: {0}")]
    User(#[from] Box<UserError>),
    #[error("user not found for id: {0}")]
    UserNotFound(si_events::UserPk),
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
        Err(TransactionsError::Tenancy(TenancyError::NoWorkspace)) => return Ok(()),
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
        tracker.to_owned(),
        source_stream.stream().await?,
        source_stream.subject_for_audit_log(
            workspace_id.into(),
            ctx.change_set_id().into(),
            match override_event_session_id {
                Some(override_id) => override_id,
                None => ctx.event_session_id(),
            },
        ),
        destination_stream.publishing_subject_for_workspace(workspace_id.into()),
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

            match WsEvent::audit_logs_published(&ctx_clone_for_ws_event).await {
                Ok(event) => {
                    if let Err(err) = event.publish_immediately(&ctx_clone_for_ws_event).await {
                        error!(?err, "error when publishing ws event for audit logs");
                    }
                }
                Err(err) => error!(?err, "error when creating ws event for audit logs"),
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

            match WsEvent::audit_logs_published(&ctx_clone_for_ws_event).await {
                Ok(event) => {
                    if let Err(err) = event.publish_immediately(&ctx_clone_for_ws_event).await {
                        error!(?err, "error when publishing ws event for audit logs");
                    }
                }
                Err(err) => error!(?err, "error when creating ws event for audit logs"),
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
        Err(TransactionsError::Tenancy(TenancyError::NoWorkspace)) => return Ok(()),
        Err(err) => return Err(AuditLoggingError::Transactions(Box::new(err))),
    };

    let destination_change_set_id =
        override_destination_change_set_id.unwrap_or(ctx.change_set_id().into());

    let pending_events_stream = PendingEventsStream::get_or_create(ctx.jetstream_context()).await?;
    pending_events_stream
        .publish_audit_log(
            workspace_id.into(),
            ctx.change_set_id().into(),
            ctx.event_session_id(),
            &AuditLog::new(
                ctx.events_actor(),
                kind,
                entity_name,
                destination_change_set_id,
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
        Err(TransactionsError::Tenancy(TenancyError::NoWorkspace)) => return Ok(()),
        Err(err) => return Err(AuditLoggingError::Transactions(Box::new(err))),
    };

    let pending_events_stream = PendingEventsStream::get_or_create(ctx.jetstream_context()).await?;
    pending_events_stream
        .publish_audit_log_final_message(
            workspace_id.into(),
            ctx.change_set_id().into(),
            ctx.event_session_id(),
        )
        .await?;
    Ok(())
}

#[instrument(name = "audit_logging.list", level = "info", skip_all, fields(size))]
pub async fn list(ctx: &DalContext, size: usize) -> Result<(Vec<FrontendAuditLog>, bool)> {
    let start = tokio::time::Instant::now();
    let workspace_id = ctx.workspace_pk().map_err(Box::new)?;

    let change_set_id = ctx.change_set_id();
    let head_change_set_id = ctx
        .get_workspace_default_change_set_id()
        .await
        .map_err(Box::new)?;
    let working_on_head = head_change_set_id == change_set_id;

    let stream_wrapper = AuditLogsStream::get_or_create(ctx.jetstream_context()).await?;
    let filter_subject = if working_on_head {
        stream_wrapper.consuming_subject_for_workspace(workspace_id.into())
    } else {
        stream_wrapper.subject_for_change_set(workspace_id.into(), change_set_id.into())
    };

    let stream = stream_wrapper.stream().await?;
    let last_sequence = stream.get_info().await?.state.last_sequence;
    let start_sequence = match last_sequence.checked_sub((size as u64) + 1) {
        Some(0) | None => 1,
        Some(difference) => difference,
    };

    info!(%last_sequence, %start_sequence, ?filter_subject, "creating ephemeral pull consumer for listing audit logs");

    let consumer = stream
        .create_consumer(async_nats::jetstream::consumer::pull::Config {
            filter_subject: filter_subject.to_string(),
            deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::ByStartSequence {
                start_sequence,
            },
            ..Default::default()
        })
        .await?;

    let mut assembler = FrontendAuditLogAssembler::new(ctx).await?;
    let mut frontend_audit_logs = Vec::new();

    let mut counter = 0;
    let mut can_load_more_logs = false;
    let mut messages = consumer.fetch().max_messages(size + 1).messages().await?;

    while let Some(message) = messages.next().await {
        counter += 1;

        // These are the two conditions in which we can load more logs and exit early.
        if size == 0 || counter == size + 1 {
            can_load_more_logs = true;
            break;
        }

        let message = message.map_err(AuditLoggingError::Message)?;
        let audit_log: AuditLog = serde_json::from_slice(&message.payload)?;
        if let Some(frontend_audit_log) = assembler.assemble(ctx, audit_log).await? {
            frontend_audit_logs.push(frontend_audit_log);
        }
    }

    // We must sort the logs on the way out because we cannot guarantee perfect ordering by the
    // time all messages are published to the stream.
    frontend_audit_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    info!(elapsed = ?start.elapsed(), "listing audit logs complete");

    Ok((frontend_audit_logs, can_load_more_logs))
}

#[derive(Debug)]
struct FrontendAuditLogAssembler {
    change_set_cache: HashMap<si_events::ChangeSetId, ChangeSet>,
    user_cache: HashMap<si_events::UserPk, User>,
    change_set_id: si_events::ChangeSetId,
    working_on_head: bool,
}

impl FrontendAuditLogAssembler {
    pub async fn new(ctx: &DalContext) -> Result<Self> {
        let head_change_set_id = ctx
            .get_workspace_default_change_set_id()
            .await
            .map_err(Box::new)?;
        let change_set_id = ctx.change_set_id();
        Ok(Self {
            change_set_cache: HashMap::new(),
            user_cache: HashMap::new(),
            change_set_id: change_set_id.into(),
            working_on_head: head_change_set_id == change_set_id,
        })
    }

    pub async fn assemble(
        &mut self,
        ctx: &DalContext,
        audit_log: AuditLog,
    ) -> Result<Option<FrontendAuditLog>> {
        match audit_log {
            AuditLog::V1(inner) => {
                let change_set_metadata = self
                    .find_change_set_metadata(ctx, inner.change_set_id)
                    .await?;

                // Before we continue, we need to know if we need to filter out the audit log based
                // on if we are working on HEAD.
                //
                // If we are working on HEAD, we show all audit logs without a change set, all
                // audit logs on HEAD, and all audit logs for abandoned or applied change sets.
                //
                // If we are not working on HEAD, we only show audit logs for our own change set as
                // well as certain audit logs that are relevant on HEAD, like "CreateChangeSet".
                if self.working_on_head {
                    if let Some((change_set_id, _, change_set_status)) = change_set_metadata {
                        if change_set_id != self.change_set_id {
                            match change_set_status {
                                ChangeSetStatus::Abandoned | ChangeSetStatus::Applied => {}
                                ChangeSetStatus::Approved
                                | ChangeSetStatus::Failed
                                | ChangeSetStatus::NeedsAbandonApproval
                                | ChangeSetStatus::NeedsApproval
                                | ChangeSetStatus::Open
                                | ChangeSetStatus::Rejected => {
                                    return Ok(None);
                                }
                            }
                        }
                    }
                } else {
                    match change_set_metadata {
                        Some((change_set_id, _, _)) => {
                            if change_set_id != self.change_set_id {
                                return Ok(None);
                            }
                        }
                        None => {
                            return Ok(None);
                        }
                    }
                }

                let (user_id, user_email, user_name) =
                    self.find_user_metadata(ctx, inner.actor).await?;

                let kind = inner.kind.to_string();
                let deserialized_metadata = FrontendAuditLogDeserializedMetadata::from(inner.kind);
                let (title, entity_type) = deserialized_metadata.title_and_entity_type();
                let (change_set_id, change_set_name) = match change_set_metadata {
                    Some((change_set_id, change_set_name, _)) => {
                        (Some(change_set_id), Some(change_set_name))
                    }
                    None => (None, None),
                };

                Ok(Some(FrontendAuditLog {
                    title: title.to_owned(),
                    user_id,
                    user_email,
                    user_name,
                    kind,
                    entity_name: inner.entity_name,
                    entity_type: entity_type.to_owned(),
                    timestamp: inner.timestamp,
                    change_set_id,
                    change_set_name,
                    metadata: serde_json::to_value(deserialized_metadata)?,
                }))
            }
        }
    }

    async fn find_change_set_metadata(
        &mut self,
        ctx: &DalContext,
        change_set_id: Option<si_events::ChangeSetId>,
    ) -> Result<Option<(si_events::ChangeSetId, String, ChangeSetStatus)>> {
        match change_set_id {
            Some(change_set_id) => {
                let (change_set_status, change_set_name) =
                    if let Some(change_set) = self.change_set_cache.get(&change_set_id) {
                        (change_set.status, change_set.name.to_owned())
                    } else {
                        let change_set = ChangeSet::find(ctx, change_set_id.into())
                            .await
                            .map_err(Box::new)?
                            .ok_or(AuditLoggingError::ChangeSetNotFound(change_set_id))?;
                        let found_data = (change_set.status, change_set.name.to_owned());
                        self.change_set_cache.insert(change_set_id, change_set);
                        found_data
                    };

                Ok(Some((change_set_id, change_set_name, change_set_status)))
            }
            None => Ok(None),
        }
    }

    async fn find_user_metadata(
        &mut self,
        ctx: &DalContext,
        actor: Actor,
    ) -> Result<(Option<si_events::UserPk>, Option<String>, Option<String>)> {
        match actor {
            Actor::System => Ok((None, None, None)),
            Actor::User(user_id) => {
                if let Some(user) = self.user_cache.get(&user_id) {
                    Ok((
                        Some(user_id),
                        Some(user.email().to_owned()),
                        Some(user.name().to_owned()),
                    ))
                } else {
                    let user = User::get_by_pk(ctx, user_id.into())
                        .await
                        .map_err(Box::new)?
                        .ok_or(AuditLoggingError::UserNotFound(user_id))?;
                    let found_data = (
                        Some(user_id),
                        Some(user.email().to_owned()),
                        Some(user.name().to_owned()),
                    );
                    self.user_cache.insert(user_id, user);
                    Ok(found_data)
                }
            }
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogsPublishedPayload {
    change_set_id: crate::ChangeSetId,
    change_set_status: ChangeSetStatus,
}

impl WsEvent {
    pub async fn audit_logs_published(ctx: &DalContext) -> WsEventResult<Self> {
        let change_set = ChangeSet::find(ctx, ctx.change_set_id())
            .await?
            .ok_or(ChangeSetError::ChangeSetNotFound(ctx.change_set_id()))?;
        WsEvent::new(
            ctx,
            WsPayload::AuditLogsPublished(AuditLogsPublishedPayload {
                change_set_id: change_set.id,
                change_set_status: change_set.status,
            }),
        )
        .await
    }
}
