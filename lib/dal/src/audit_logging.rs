//! This module provides audit logging functionality to the rest of the crate.

use std::collections::HashMap;
use std::collections::HashSet;

use audit_logs::AuditLogsError;
use audit_logs::AuditLogsStream;
use futures::StreamExt;
use pending_events::PendingEventsError;
use pending_events::PendingEventsStream;
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

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AuditLoggingError {
    #[error("async nats batch error: {0}")]
    AsyncNatsBatch(#[from] async_nats::error::Error<BatchErrorKind>),
    #[error("async nats consumer error: {0}")]
    AsyncNatsConsumer(#[from] async_nats::error::Error<ConsumerErrorKind>),
    #[error("async nats request error: {0}")]
    AsyncNatsRequest(#[from] async_nats::error::Error<RequestErrorKind>),
    #[error("audit logs error: {0}")]
    AuditLogs(#[from] AuditLogsError),
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

    // TODO(nick): somewhere in this function (or alongside it), we need to deserialize the events and send them up to
    // the frontend. The frontend can make its own decision to include it into its paginated and filtered view, but we
    // will make one event that sends an array of published audit logs. For chunking, we can do a naive array list
    // length check or something else (e.g. size check) and send multiple messages, if need be. The first move might be
    // to add a broadcast channel were shuttle sends the message paylods in bytestream format. Then, after shuttle
    // exits, the dal can deserialize those messages.

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
        destination_stream.subject(workspace_id.into()),
    )
    .await?;

    // Run the audit logs shuttle instance. If a tracker has been provided, we can spawn the
    // shuttle instance using it. If we are using a tracker purely within this function, we cannot
    // reliably use it to run the shuttle instance, so we will close and wait once shuttle exits.
    if provided_tracker {
        tracker.spawn(async move {
            if let Err(err) = audit_logs_shuttle.try_run().await {
                error!(?err, "audit logs shuttle error");
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
        });
    }

    Ok(())
}

#[instrument(name = "audit_logging.write", level = "debug", skip_all, fields(kind))]
pub(crate) async fn write(ctx: &DalContext, kind: AuditLogKind, entity_name: String) -> Result<()> {
    // TODO(nick): nuke this from intergalactic orbit. Then do it again.
    let workspace_id = match ctx.workspace_pk() {
        Ok(workspace_id) => workspace_id,
        Err(TransactionsError::Tenancy(TenancyError::NoWorkspace)) => return Ok(()),
        Err(err) => return Err(AuditLoggingError::Transactions(Box::new(err))),
    };

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
                ctx.change_set_id().into(),
            ),
        )
        .await?;
    Ok(())
}

#[instrument(
    name = "audit_logging.write_to_head",
    level = "debug",
    skip_all,
    fields(kind)
)]
pub(crate) async fn write_to_head(
    ctx: &DalContext,
    kind: AuditLogKind,
    entity_name: String,
) -> Result<()> {
    // TODO(nick): nuke this from intergalactic orbit. Then do it again.
    let workspace_id = match ctx.workspace_pk() {
        Ok(workspace_id) => workspace_id,
        Err(TransactionsError::Tenancy(TenancyError::NoWorkspace)) => return Ok(()),
        Err(err) => return Err(AuditLoggingError::Transactions(Box::new(err))),
    };

    let default_changeset_id = ctx
        .get_workspace_default_change_set_id()
        .await
        .map_err(Box::new)?;

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
                default_changeset_id.into(),
            ),
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

#[allow(clippy::too_many_arguments)]
#[instrument(
    name = "audit_logging.list",
    level = "info",
    skip_all,
    fields(
        page,
        page_size,
        sort_timestamp_ascending,
        change_set_filter,
        entity_type_filter,
        kind_filter,
        user_filter
    )
)]
pub async fn list(
    ctx: &DalContext,
    page: usize,
    page_size: usize,
    sort_timestamp_ascending: bool,
    change_set_filter: HashSet<si_events::ChangeSetId>,
    entity_type_filter: HashSet<String>,
    kind_filter: HashSet<String>,
    user_filter: HashSet<Option<si_events::UserPk>>,
) -> Result<(Vec<FrontendAuditLog>, usize)> {
    if page == 0 || page_size == 0 {
        return Err(AuditLoggingError::CannotReturnListOfUnboundedSize(
            page, page_size,
        ));
    }

    let workspace_id = ctx.workspace_pk().map_err(Box::new)?;

    let stream_wrapper = AuditLogsStream::get_or_create(ctx.jetstream_context()).await?;
    let stream = stream_wrapper.stream().await?;
    let consumer = stream
        .create_consumer(async_nats::jetstream::consumer::pull::Config {
            filter_subject: stream_wrapper.subject(workspace_id.into()).to_string(),
            ..Default::default()
        })
        .await?;

    // FIXME(nick): find a way to perform true pagination with reverse timestamp sorting. This will
    // eventually become a performance problem.
    let total_message_count = stream.get_info().await?.state.messages;
    let total_message_count_as_usize = total_message_count as usize;

    // TODO(nick): remove this once we implement proper pagination and filtering.
    info!(
        ?total_message_count,
        ?total_message_count_as_usize,
        "total message count on the audit logs stream"
    );

    // FIXME(nick): this is bad because we really only need the total count for the filtered messages by subject and
    // not for the entire. Since this is a anti-pattern anyway, we'll avoid this problem when swapping our fake
    // pagination method out for a real one.
    let mut messages = consumer
        .fetch()
        .max_messages(total_message_count_as_usize)
        .messages()
        .await?;

    let mut parser = Parser::new(
        change_set_filter,
        entity_type_filter,
        kind_filter,
        user_filter,
    );

    let mut filtered_audit_logs = Vec::new();
    while let Some(message) = messages.next().await {
        let message = message.map_err(AuditLoggingError::Message)?;
        let audit_log: AuditLog = serde_json::from_slice(&message.payload)?;
        if let Some(filtered_audit_log) = parser.filter_and_assemble(ctx, audit_log).await? {
            filtered_audit_logs.push(filtered_audit_log);
        }
    }

    // Before performing fake pagination, we need to cache the total number of filtered audit logs.
    let filtered_audit_logs_total = filtered_audit_logs.len();
    let paginated_audit_logs = fake_pagination(
        filtered_audit_logs,
        page,
        page_size,
        sort_timestamp_ascending,
    );

    Ok((paginated_audit_logs, filtered_audit_logs_total))
}

#[derive(Debug)]
struct Parser {
    change_set_cache: HashMap<si_events::ChangeSetId, ChangeSet>,
    user_cache: HashMap<si_events::UserPk, User>,
    change_set_filter: HashSet<si_events::ChangeSetId>,
    entity_type_filter: HashSet<String>,
    kind_filter: HashSet<String>,
    user_filter: HashSet<Option<si_events::UserPk>>,
}

impl Parser {
    pub fn new(
        change_set_filter: HashSet<si_events::ChangeSetId>,
        entity_type_filter: HashSet<String>,
        kind_filter: HashSet<String>,
        user_filter: HashSet<Option<si_events::UserPk>>,
    ) -> Self {
        Self {
            change_set_cache: HashMap::new(),
            user_cache: HashMap::new(),
            change_set_filter,
            entity_type_filter,
            kind_filter,
            user_filter,
        }
    }

    pub async fn filter_and_assemble(
        &mut self,
        ctx: &DalContext,
        audit_log: AuditLog,
    ) -> Result<Option<FrontendAuditLog>> {
        match audit_log {
            AuditLog::V1(inner) => {
                // Gather data that may be in our caches.
                let (skip_due_to_change_set_status, change_set_id, change_set_name) =
                    self.change_set_data(ctx, inner.change_set_id).await?;
                if skip_due_to_change_set_status {
                    return Ok(None);
                }
                let (user_id, user_email, user_name) = self.user_data(ctx, inner.actor).await?;

                // Gather data based on the specific audit log kind.
                let kind = inner.kind.to_string();
                let deserialized_metadata = FrontendAuditLogDeserializedMetadata::from(inner.kind);
                let (title, entity_type) = deserialized_metadata.title_and_entity_type();

                // Check the filters.
                if let Some(change_set_id) = change_set_id {
                    if !self.change_set_filter.is_empty()
                        && !self.change_set_filter.contains(&change_set_id)
                    {
                        return Ok(None);
                    }
                } else if !self.change_set_filter.is_empty() {
                    return Ok(None);
                }
                if !self.entity_type_filter.is_empty()
                    && !self.entity_type_filter.contains(entity_type)
                {
                    return Ok(None);
                }
                if !self.kind_filter.is_empty() && !self.kind_filter.contains(&kind) {
                    return Ok(None);
                }
                if !self.user_filter.is_empty()
                    && !self.user_filter.contains(&user_id.map(Into::into))
                {
                    return Ok(None);
                }

                // If we made it here, then we are good to go.
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

    async fn change_set_data(
        &mut self,
        ctx: &DalContext,
        change_set_id: Option<si_events::ChangeSetId>,
    ) -> Result<(bool, Option<si_events::ChangeSetId>, Option<String>)> {
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

                match change_set_status {
                    ChangeSetStatus::Failed | ChangeSetStatus::Rejected => {
                        trace!(
                            ?change_set_status,
                            ?change_set_id,
                            "skipping change set for audit log assembly due to status"
                        );
                        Ok((true, None, None))
                    }
                    ChangeSetStatus::Abandoned
                    | ChangeSetStatus::Applied
                    | ChangeSetStatus::Approved
                    | ChangeSetStatus::NeedsAbandonApproval
                    | ChangeSetStatus::NeedsApproval
                    | ChangeSetStatus::Open => {
                        Ok((false, Some(change_set_id), Some(change_set_name)))
                    }
                }
            }
            None => Ok((false, None, None)),
        }
    }

    async fn user_data(
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

// FIXME(nick): we need to replace this with real pagination, but it has to work with reverse timestamp sorting. That
// will be tricky as the NATS JetStream consumer will need to consume in reverse order... or something analogous.
fn fake_pagination(
    logs: Vec<FrontendAuditLog>,
    page: usize,
    page_size: usize,
    sort_timestamp_ascending: bool,
) -> Vec<FrontendAuditLog> {
    let logs = if sort_timestamp_ascending {
        let mut logs = logs;
        logs.reverse();
        logs
    } else {
        logs
    };

    let mut current_page = 1;
    for chunk in logs.chunks(page_size) {
        if current_page == page {
            return chunk.to_vec();
        }
        current_page += 1;
    }

    logs
}
