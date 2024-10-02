use std::{result, str::FromStr, sync::Arc};

use naxum::{
    extract::State,
    response::{IntoResponse, Response},
};
use rebaser_core::nats;
use si_data_nats::{
    async_nats::jetstream::{
        consumer::{push, StreamError},
        stream::ConsumerError,
    },
    NatsClient, Subject,
};
use si_events::{ChangeSetId, WorkspacePk};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::Notify;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use ulid::Ulid;

use crate::{
    app_state::AppState,
    change_set_processor_task::{ChangeSetProcessorTask, ChangeSetProcessorTaskError},
    serial_dvu_task::{SerialDvuTask, SerialDvuTaskError},
    Shutdown,
};

const CONSUMER_NAME_PREFIX: &str = "rebaser-requests";

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum HandlerError {
    #[error("change set processor error: {0}")]
    ChangeSetProcessor(#[from] ChangeSetProcessorTaskError),
    #[error("change set processor unexpectedly completed without error")]
    ChangeSetProcessorCompleted,
    #[error("change set processor error on tokio join")]
    ChangeSetProcessorJoin,
    #[error("error creating per-change set consumer: {0}")]
    ConsumerCreate(#[source] ConsumerError),
    #[error("serial dvu error: {0}")]
    SerialDvu(#[from] SerialDvuTaskError),
    #[error("serial dvu unexpectedly completed without error")]
    SerialDvuCompleted,
    #[error("serial dvu error on tokio join")]
    SerialDvuJoin,
    #[error("failed to parse subject: subject={0}, reason={1}")]
    SubjectParse(String, String),
    #[error("error while subscribing for messages: {0}")]
    Subscribe(#[source] StreamError),
    #[error("task has remaining messages: {0}")]
    TaskHasMessages(String),
    #[error("task interupted: {0}")]
    TaskInterrupted(String),
}

type Error = HandlerError;

type Result<T> = result::Result<T, HandlerError>;

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        match self {
            HandlerError::SubjectParse(_, _) => {
                warn!(si.error.message = ?self, "subject parse error");
                Response::bad_request()
            }
            // While propagated as an `Err`, a task being interupted is expected behavior and is
            // not an error (rather we use `Err` to ensure the task persists in the stream)
            HandlerError::TaskInterrupted(subject) => {
                debug!(subject, "task interrupted");
                Response::service_unavailable()
            }
            _ => {
                error!(si.error.message = ?self, "failed to process message");
                Response::internal_server_error()
            }
        }
    }
}

pub(crate) async fn default(State(state): State<AppState>, subject: Subject) -> Result<()> {
    let AppState {
        metadata,
        nats,
        requests_stream,
        ctx_builder,
        quiescent_period,
        token: server_token,
    } = state;
    let subject_prefix = nats.metadata().subject_prefix();

    let subject_str = subject.as_str();
    let (workspace, change_set) = parse_subject(subject_prefix, subject_str)?;

    let requests_stream_filter_subject = nats::subject::enqueue_updates_for_change_set(
        subject_prefix,
        workspace.str,
        change_set.str,
    );

    let tracker = TaskTracker::new();

    // We want to indendently control the lifecyle of our tasks
    let tasks_token = CancellationToken::new();

    let run_dvu_notify = Arc::new(Notify::new());

    let quiesced_token = CancellationToken::new();
    let quiesced_notify = Arc::new(Notify::new());

    let incoming = requests_stream
        .create_consumer(rebaser_requests_per_change_set_consumer_config(
            &nats,
            &requests_stream_filter_subject,
            metadata.instance_id(),
            &workspace,
            &change_set,
        ))
        .await
        .map_err(HandlerError::ConsumerCreate)?
        .messages()
        .await
        .map_err(HandlerError::Subscribe)?;

    let dvu_task = SerialDvuTask::create(
        metadata.clone(),
        workspace.id,
        change_set.id,
        ctx_builder.clone(),
        run_dvu_notify.clone(),
        quiesced_notify.clone(),
        quiesced_token.clone(),
        tasks_token.clone(),
    );

    let processor_task = ChangeSetProcessorTask::create(
        metadata.clone(),
        nats,
        requests_stream.clone(),
        incoming,
        workspace.id,
        change_set.id,
        ctx_builder,
        run_dvu_notify,
        quiescent_period,
        quiesced_notify,
        quiesced_token,
        tasks_token.clone(),
    );

    let dvu_task_result = tracker.spawn(dvu_task.try_run());
    let processor_task_result = tracker.spawn(processor_task.try_run());
    tracker.close();

    let result = tokio::select! {
        biased;

        // Cancellation token has fired, time to shut down
        _ = server_token.cancelled() => {
            debug!(
                service.instance.id = metadata.instance_id(),
                si.workspace.id = %workspace.str,
                si.change_set.id = %change_set.str,
                "received cancellation",
            );
            // Task may not be complete but was interupted; reply `Err` to nack for task to persist
            // and retry to continue progress
            Err(Error::TaskInterrupted(subject_str.to_string()))
        }
        // Processor task completed
        processor_task_result_result = processor_task_result => {
            match processor_task_result_result {
                // Processor exited cleanly, but unexpectedly; reply `Err` to nack for task to
                // persist and retry
                Ok(Ok(())) => Err(Error::ChangeSetProcessorCompleted),
                // Processor exited with error; reply `Err` to nack for task to persist and retry
                Ok(Err(err)) => Err(Error::ChangeSetProcessor(err)),
                // Tokio join error on processor exit; reply `Err` to nack for task to persist and
                // retry
                Err(_join_err) => Err(Error::ChangeSetProcessorJoin),
            }
        }
        // Serial dvu task completed
        dvu_task_result_result = dvu_task_result => {
            match dvu_task_result_result {
                // A quiet period was found in the stream; reply `Ok` to ack and remove this task
                Ok(Ok(Shutdown::Quiesced)) => Ok(()),
                // Serial dvu exited cleanly, but unexpectedly; reply `Err` to nack for task to
                // persist and retry
                Ok(Ok(Shutdown::Graceful)) => Err(Error::SerialDvuCompleted),
                // Serial dvu exited with error; reply `Err` to nack for task to persist and retry
                Ok(Err(err)) => Err(Error::SerialDvu(err)),
                // Tokio join error on serial dvu exit; reply `Err` to nack for task to persist and
                // retry
                Err(_join_err) => Err(Error::SerialDvuJoin),
            }
        }
    };

    tasks_token.cancel();
    tracker.wait().await;

    // If the processor task was ended via a quiesced shutdown, then check one last time if there
    // are messages on the subject. This means that during the quiet period-triggered shutdown,
    // another message was published to our subject (such as during this handler waiting on the
    // serial dvu task to finish). In this case, we'll return a new `Err` variant to ensure the
    // task message is `nack`d and the task will be redelivered.
    if let Err(Error::ChangeSetProcessorCompleted) = result {
        match requests_stream
            .get_last_raw_message_by_subject(requests_stream_filter_subject.as_str())
            .await
        {
            // We found a message on the subject
            Ok(message) => {
                debug!(
                    messaging.message.id = message.sequence,
                    messaging.destination.name = message.subject.as_str(),
                    service.instance.id = metadata.instance_id(),
                    si.change_set.id = %change_set.str,
                    si.workspace.id = %workspace.str,
                    "message found after graceful shutdown",
                );
                Err(Error::TaskHasMessages(
                    requests_stream_filter_subject.to_string(),
                ))
            }
            // Either there was not a message or another error with this call. Either way, we can
            // return the current `result` value
            Err(_) => result,
        }
    } else {
        // In all other cases, return our computed `result` value
        result
    }
}

struct ParsedWorkspaceId<'a> {
    id: WorkspacePk,
    str: &'a str,
}

struct ParsedChangeSetId<'a> {
    id: ChangeSetId,
    str: &'a str,
}

#[inline]
fn parse_subject<'a>(
    subject_prefix: Option<&str>,
    subject_str: &'a str,
) -> Result<(ParsedWorkspaceId<'a>, ParsedChangeSetId<'a>)> {
    let mut parts = subject_str.split('.');

    if let Some(prefix) = subject_prefix {
        match parts.next() {
            // Prefix part matches expected/configured prefix
            Some(parsed_prefix) if parsed_prefix == prefix => {}
            // Prefix part does not match expected/configured prefix
            Some(unexpected) => {
                return Err(HandlerError::SubjectParse(
                    subject_str.to_string(),
                    format!(
                        "found unexpected subject prefix; expected={prefix}, parsed={unexpected}"
                    ),
                ))
            }
            // Prefix part not found but expected
            None => {
                return Err(HandlerError::SubjectParse(
                    subject_str.to_string(),
                    format!("expected subject prefix not found; expected={prefix}"),
                ))
            }
        };
    }

    match (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(), // assert last part is `None` to ensure there are no additional parts
    ) {
        (
            Some(_),
            Some(_),
            Some(workspace_id_str),
            Some(change_set_id_str),
            Some("process"),
            None,
        ) => {
            let workspace_id = WorkspacePk::from_str(workspace_id_str).map_err(|err| {
                HandlerError::SubjectParse(
                    subject_str.to_string(),
                    format!("workspace id parse error: {err}"),
                )
            })?;
            let change_set_id = ChangeSetId::from_str(change_set_id_str).map_err(|err| {
                HandlerError::SubjectParse(
                    subject_str.to_string(),
                    format!("change set id parse error: {err}"),
                )
            })?;

            Ok((
                ParsedWorkspaceId {
                    id: workspace_id,
                    str: workspace_id_str,
                },
                ParsedChangeSetId {
                    id: change_set_id,
                    str: change_set_id_str,
                },
            ))
        }
        _ => Err(HandlerError::SubjectParse(
            subject_str.to_string(),
            "subject failed to parse with unexpected parts".to_string(),
        )),
    }
}

fn rebaser_requests_per_change_set_consumer_config(
    nats: &NatsClient,
    filter_subject: &Subject,
    instance_id: &str,
    workspace: &ParsedWorkspaceId<'_>,
    change_set: &ParsedChangeSetId<'_>,
) -> push::OrderedConfig {
    push::OrderedConfig {
        name: Some(format!(
            "{CONSUMER_NAME_PREFIX}-{}-{}-{}-{}",
            workspace.str,
            change_set.str,
            instance_id,
            Ulid::new(),
        )),
        deliver_subject: nats.new_inbox(),
        filter_subject: filter_subject.to_string(),
        ..Default::default()
    }
}
