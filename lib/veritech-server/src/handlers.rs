use std::{
    collections::HashMap,
    result,
    str::Utf8Error,
    sync::Arc,
    time::Duration,
};

use chrono::Utc;
use futures::StreamExt;
pub use kill::process_kill_request;
use nats_std::headers;
use naxum::{
    Message,
    extract::{
        State,
        message_parts::Headers,
    },
    response::{
        IntoResponse,
        Response,
    },
};
use serde::{
    Serialize,
    de::DeserializeOwned,
};
use si_data_nats::{
    HeaderMap,
    InnerMessage,
    Subject,
};
// seems strange to get these cyclone_core types from si_pool_noodle?
use si_pool_noodle::{
    ActionRunResultSuccess,
    CycloneClient,
    CycloneRequest,
    CycloneRequestable,
    ExecutionError,
    FunctionResultFailure,
    FunctionResultFailureError,
    ManagementResultSuccess,
    ProgressMessage,
    ResolverFunctionResultSuccess,
    SchemaVariantDefinitionResultSuccess,
    SensitiveStrings,
    ValidationResultSuccess,
    errors::PoolNoodleError,
};
use telemetry::prelude::*;
use telemetry_utils::metric;
use thiserror::Error;
use tokio::sync::{
    Mutex,
    oneshot,
};
use veritech_core::{
    ExecutionId,
    VeritechRequest,
    VeritechRequestError,
    VeritechValueDecryptError,
};

use crate::{
    Publisher,
    PublisherError,
    app_state::AppState,
    request::DecryptRequest,
};

mod kill;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("could not send kill signal for execution id: {0}")]
    CouldNotSendKillSignal(ExecutionId),
    #[error("cyclone pool error: {0}")]
    CyclonePool(#[source] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("cyclone timed out: {0:?}")]
    CycloneTimeout(Duration),
    #[error("invalid incoming subject: {0}")]
    InvalidIncomingSubject(Subject),
    #[error("function execution killed: {0}")]
    Killed(ExecutionId),
    #[error("missing kill sender for execution id: {0}")]
    MissingKillSender(ExecutionId),
    #[error("no reply inbox provided")]
    NoReplyInbox,
    #[error("pool noodle client error: {0}")]
    PoolNoodleClient(#[from] si_pool_noodle::ClientError),
    #[error("pool noodle execution action run: {0}")]
    PoolNoodleExecutionActionRun(#[from] si_pool_noodle::ExecutionError<ActionRunResultSuccess>),
    #[error("pool noodle execution management: {0}")]
    PoolNoodleExecutionManagement(#[from] si_pool_noodle::ExecutionError<ManagementResultSuccess>),
    #[error("pool noodle execution resovler function: {0}")]
    PoolNoodleExecutionResolverFunction(
        #[from] si_pool_noodle::ExecutionError<ResolverFunctionResultSuccess>,
    ),
    #[error("pool noodle execution schema variant definition: {0}")]
    PoolNoodleExecutionSchemaVariantDefinition(
        #[from] si_pool_noodle::ExecutionError<SchemaVariantDefinitionResultSuccess>,
    ),
    #[error("pool noodle execution validation: {0}")]
    PoolNoodleExecutionValidation(#[from] si_pool_noodle::ExecutionError<ValidationResultSuccess>),
    #[error("publisher error: {0}")]
    Publisher(#[from] PublisherError),
    #[error("utf8 error when creating subject")]
    Utf8(#[from] Utf8Error),
    #[error("veritech request error: {0}")]
    VeritechRequest(#[from] VeritechRequestError),
    #[error("failed to decrypt request: {0}")]
    VeritechValueDecrypt(#[from] VeritechValueDecryptError),
}

type HandlerResult<T> = result::Result<T, HandlerError>;

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        error!(si.error.message = ?self, "failed to process message");
        Response::default_internal_server_error()
    }
}

pub async fn process_request(
    State(state): State<AppState>,
    subject: Subject,
    Headers(maybe_headers): Headers,
    msg: Message<InnerMessage>,
) -> HandlerResult<()> {
    metric!(counter.veritech.handlers_doing_work = 1);
    let result = process_request_inner(state, subject, maybe_headers, msg).await;
    metric!(counter.veritech.handlers_doing_work = -1);
    result
}

pub async fn process_request_inner(
    state: AppState,
    subject: Subject,
    maybe_headers: Option<HeaderMap>,
    msg: Message<InnerMessage>,
) -> HandlerResult<()> {
    let span = Span::current();

    let reply_subject = match maybe_headers
        .and_then(|headers| headers.get(headers::REPLY_INBOX).map(|v| v.to_string()))
    {
        Some(header_value) => Subject::from_utf8(header_value)?,
        None => return Err(HandlerError::NoReplyInbox),
    };

    let mut parts = subject.as_str().split('.');

    // Based on whether or not there is a prefix, we need to determine how many parts there are
    // before the exact subject part we are interested in.
    if state.nats_subject_has_prefix() {
        match (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            (Some(_), Some(_), Some(_), Some(workspace_id), Some(change_set_id)) => {
                span.record("si.workspace.id", workspace_id);
                span.record("si.change_set.id", change_set_id);
            }
            _ => return Err(HandlerError::InvalidIncomingSubject(subject)),
        }
    } else {
        match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some(_), Some(_), Some(workspace_id), Some(change_set_id)) => {
                span.record("si.workspace.id", workspace_id);
                span.record("si.change_set.id", change_set_id);
            }
            _ => return Err(HandlerError::InvalidIncomingSubject(subject)),
        }
    }

    let (Some(request_subject), None) = (parts.next(), parts.next()) else {
        return Err(HandlerError::InvalidIncomingSubject(subject));
    };

    let veritech_request =
        VeritechRequest::from_subject_and_payload(request_subject, &msg.payload)?;

    info!(
        execution_kind = veritech_request.subject_suffix().to_owned(),
        execution_id = veritech_request.execution_id().to_owned(),
        "validated request and about to execute"
    );

    match veritech_request {
        VeritechRequest::ActionRun(request) => {
            dispatch_request(state, request, reply_subject).await?
        }
        VeritechRequest::Management(request) => {
            dispatch_request(state, *request, reply_subject).await?
        }
        VeritechRequest::Resolver(request) => {
            dispatch_request(state, request, reply_subject).await?
        }
        VeritechRequest::SchemaVariantDefinition(request) => {
            dispatch_request(state, request, reply_subject).await?
        }
        VeritechRequest::Validation(request) => {
            dispatch_request(state, request, reply_subject).await?
        }
        // Kill requests do not get handled here
        VeritechRequest::KillExecution(_) => {
            return Err(HandlerError::InvalidIncomingSubject(subject));
        }
    }

    Ok(())
}

async fn dispatch_request<Request>(
    state: AppState,
    mut request: Request,
    reply_mailbox: Subject,
) -> HandlerResult<()>
where
    Request: CycloneRequestable + DecryptRequest + Serialize + Clone + Send + Sync,
    Request::Response: Serialize + DeserializeOwned + std::fmt::Debug + std::marker::Unpin,
    HandlerError: From<ExecutionError<<Request as CycloneRequestable>::Response>>,
{
    let span = current_span_for_instrument_at!("info");
    let mut client = match state.cyclone_pool.get().await {
        Ok(client) => client,
        Err(err) => {
            if let PoolNoodleError::ExecutionPoolStarved = err {
                metric!(counter.veritech.pool_exhausted = 1);
            }
            return Err(span.record_err(HandlerError::CyclonePool(Box::new(err))));
        }
    };

    request.inc_run_metric();

    let mut sensitive_strings = SensitiveStrings::default();
    // Decrypt the relevant contents of the request and track any resulting sensitive strings
    // to be redacted
    request.decrypt(&mut sensitive_strings, &state.decryption_key)?;

    // NOTE(nick,fletcher): we need to create a owned client here because publisher has its own lifetime. Yeehaw.
    let nats_for_publisher = state.nats.clone();
    let publisher = Publisher::new(&nats_for_publisher, &reply_mailbox);
    let execution_id = request.execution_id().to_owned();
    let cyclone_request = CycloneRequest::from_parts(request.clone(), sensitive_strings);

    let (kill_sender, kill_receiver) = oneshot::channel::<()>();
    {
        state
            .kill_senders
            .lock()
            .await
            .insert(execution_id.to_owned(), kill_sender);
    }

    let unstarted_progress = client
        .prepare_execution(cyclone_request)
        .await
        .map_err(|err| {
            request.dec_run_metric();
            span.record_err(err)
        })?;

    let progress_loop = async {
        let mut progress = unstarted_progress.start().await.map_err(|err| {
            request.dec_run_metric();
            span.record_err(err)
        })?;

        while let Some(msg) = progress.next().await {
            match msg {
                Ok(ProgressMessage::OutputStream(output)) => {
                    publisher.publish_output(&output).await.map_err(|err| {
                        request.dec_run_metric();
                        span.record_err(err)
                    })?;
                }
                Ok(ProgressMessage::Heartbeat) => {
                    trace!("received heartbeat message");
                }
                Err(err) => {
                    warn!(si.error.message = ?err, "next progress message was an error, bailing out");
                    break;
                }
            }
        }
        publisher.finalize_output().await.map_err(|err| {
            request.dec_run_metric();
            span.record_err(err)
        })?;

        let function_result = progress.finish().await.map_err(|err| {
            request.dec_run_metric();
            span.record_err(err)
        })?;

        HandlerResult::Ok(function_result)
    };

    // we do not want to return errors at this point as it will Nack the message and end up auto-retrying
    // functions that may have failed for legitimate reasons and should not be retried
    let timeout = state.cyclone_client_execution_timeout;
    let result = tokio::select! {
        _ = tokio::time::sleep(timeout) => {
            error!("hit timeout for communicating with cyclone server:{:?}", &timeout);
            kill_sender_remove_blocking(&state.kill_senders, execution_id.to_owned()).await?;
            Err(HandlerError::CycloneTimeout(
                timeout,
            ))
        },
        Ok(_) = kill_receiver => {
            Err(HandlerError::Killed(execution_id.clone()))
        }
        func_result = progress_loop => {
            kill_sender_remove_blocking(&state.kill_senders, execution_id.to_owned()).await?;
            func_result
        },
    };

    match result {
        // Got an Ok - let anyone subscribing to a reply know
        Ok(function_result) => {
            if let Err(err) = publisher.publish_result(&function_result).await {
                error!(si.error.message = ?err, "failed to publish errored result");
            }

            request.dec_run_metric();
            span.record_ok();
        }
        // Got an error that we don't want to recover from here - need to let anyone subscribing know we're done
        // so they're not waiting forever and can decide how to proceed
        // Construct the Error result to propagate to subscribers
        Err(ref err) => {
            let func_result_error = match err {
                HandlerError::CycloneTimeout(timeout) => {
                    warn!(si.error.message = ?err, "timed out trying to run function to completion: {:?}", timeout);
                    let func_res_failure = FunctionResultFailure::new_for_veritech_server_error(
                        execution_id.to_owned(),
                        "timed out trying to run function to completion",
                        timestamp(),
                    );
                    si_pool_noodle::FunctionResult::Failure::<Request>(func_res_failure)
                }
                HandlerError::Killed(execution_id) => {
                    warn!(si.error.message = ?err, si.func_run.id = ?execution_id, "function killed during execution: {:?} via signal", execution_id);
                    let func_res_failure = FunctionResultFailure::new(
                        execution_id.to_owned(),
                        FunctionResultFailureError {
                            kind: si_pool_noodle::FunctionResultFailureErrorKind::KilledExecution,
                            message: "function execution terminated".to_owned(),
                        },
                        timestamp(),
                    );
                    si_pool_noodle::FunctionResult::Failure::<Request>(func_res_failure)
                }
                err => {
                    error!(si.error.message = ?err, si.func_run.id = ?execution_id.to_owned(), "failure trying to run function to completion");
                    let func_res_failure = FunctionResultFailure::new_for_veritech_server_error(
                        execution_id.to_owned(),
                        "timed out trying to run function to completion",
                        timestamp(),
                    );
                    si_pool_noodle::FunctionResult::Failure::<Request>(func_res_failure)
                }
            };
            request.dec_run_metric();
            if let Err(err) = publisher.publish_result(&func_result_error).await {
                error!(si.error.message = ?err, "failed to publish errored result");
            }
        }
    }

    Ok(())
}

#[instrument(
    name = "veritech.kill_sender_remove_blocking",
    level = "debug",
    skip_all
)]
async fn kill_sender_remove_blocking(
    kill_senders: &Arc<Mutex<HashMap<ExecutionId, oneshot::Sender<()>>>>,
    execution_id: String,
) -> HandlerResult<Option<oneshot::Sender<()>>> {
    let span = current_span_for_instrument_at!("debug");

    let maybe_kill_sender = { kill_senders.lock().await.remove(&execution_id) };

    if maybe_kill_sender.is_some() {
        debug!(%execution_id, "removed kill sender for execution id");
    } else {
        debug!(%execution_id, "no kill sender found when removing for execution id");
    }

    span.record_ok();
    Ok(maybe_kill_sender)
}

fn timestamp() -> u64 {
    // NOTE(nick,fletcher,scott): this should never panic. This is okay to do in very specific circumstances, like this
    // one. If this panics, look out your window because the aliens are likely invading from another galaxy.
    #[allow(clippy::expect_used, clippy::panic)]
    u64::try_from(std::cmp::max(Utc::now().timestamp(), 0)).expect("timestamp not be negative")
}
