use std::{
    result,
    str::FromStr,
    sync::Arc,
    time::Instant,
};

use edda_core::nats;
use naxum::{
    extract::State,
    response::{
        IntoResponse,
        Response,
    },
};
use si_data_nats::{
    NatsClient,
    Subject,
    async_nats::jetstream::{
        consumer::{
            StreamError,
            push,
        },
        stream::ConsumerError,
    },
};
use si_events::{
    ChangeSetId,
    WorkspacePk,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{
    Notify,
    watch,
};
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

use crate::{
    app_state::AppState,
    change_set_processor_task::{
        ChangeSetProcessorTask,
        ChangeSetProcessorTaskError,
    },
    compressing_stream::CompressingStream,
};

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
                Response::default_bad_request()
            }
            // While propagated as an `Err`, a task being interupted is expected behavior and is
            // not an error (rather we use `Err` to ensure the task persists in the stream)
            HandlerError::TaskInterrupted(subject) => {
                debug!(subject, "task interrupted");
                Response::default_service_unavailable()
            }
            _ => {
                error!(si.error.message = ?self, "failed to process message");
                Response::default_internal_server_error()
            }
        }
    }
}

pub(crate) async fn default(State(state): State<AppState>, subject: Subject) -> Result<()> {
    let AppState {
        metadata,
        nats,
        frigg,
        edda_updates,
        parallel_build_limit,
        requests_stream,
        ctx_builder,
        quiescent_period,
        token: server_token,
        server_tracker,
    } = state;
    let subject_prefix = nats.metadata().subject_prefix();

    let subject_str = subject.as_str();
    let (workspace, change_set) = parse_subject(subject_prefix, subject_str)?;

    let requests_stream_filter_subject =
        nats::subject::request_for_change_set(subject_prefix, workspace.str, change_set.str);

    let tracker = TaskTracker::new();

    // We want to independently control the lifecyle of our tasks
    let tasks_token = CancellationToken::new();

    let quiesced_token = CancellationToken::new();
    let quiesced_notify = Arc::new(Notify::new());

    let (last_compressing_heartbeat_tx, last_compressing_heartbeat_rx) =
        watch::channel(Instant::now());

    let incoming = requests_stream
        .create_consumer(edda_requests_per_change_set_consumer_config(
            &nats,
            &requests_stream_filter_subject,
        ))
        .await
        .map_err(HandlerError::ConsumerCreate)?
        .messages()
        .await
        .map_err(HandlerError::Subscribe)?;
    let incoming = CompressingStream::new(
        incoming,
        requests_stream.clone(),
        last_compressing_heartbeat_tx,
    );

    let processor_task = ChangeSetProcessorTask::create(
        metadata.clone(),
        nats,
        incoming,
        frigg,
        edda_updates,
        parallel_build_limit,
        workspace.id,
        change_set.id,
        ctx_builder,
        quiescent_period,
        quiesced_notify.clone(),
        quiesced_token.clone(),
        last_compressing_heartbeat_rx,
        tasks_token.clone(),
        server_tracker,
    );

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
        // The processor tasks has signaled to shutdown from a quiet period
        _ = quiesced_notify.notified() => {
            info!(
                service.instance.id = metadata.instance_id(),
                si.workspace.id = %workspace.str,
                si.change_set.id = %change_set.str,
                "quiesced notified, starting to shut down",
            );

            // Fire the quiesced_token so that the processing task immediately stops
            // processing additional requests
            quiesced_token.cancel();

            Ok(())
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
                ));
            }
            // Prefix part not found but expected
            None => {
                return Err(HandlerError::SubjectParse(
                    subject_str.to_string(),
                    format!("expected subject prefix not found; expected={prefix}"),
                ));
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

fn edda_requests_per_change_set_consumer_config(
    nats: &NatsClient,
    filter_subject: &Subject,
) -> push::OrderedConfig {
    push::OrderedConfig {
        deliver_subject: nats.new_inbox(),
        filter_subject: filter_subject.to_string(),
        ..Default::default()
    }
}
