use chrono::Utc;
use futures::StreamExt;
use naxum::{
    extract::{message_parts::Headers, State},
    response::{IntoResponse, Response},
};
use si_data_nats::{InnerMessage, Subject};
use si_pool_noodle::{
    ActionRunRequest, ActionRunResultSuccess, CycloneClient, CycloneRequest, FunctionResult,
    FunctionResultFailure, ProgressMessage, ResolverFunctionRequest, ResolverFunctionResultSuccess,
    SchemaVariantDefinitionRequest, SchemaVariantDefinitionResultSuccess, SensitiveStrings,
    ValidationRequest, ValidationResultSuccess,
};
use std::{collections::HashMap, result, str::Utf8Error, sync::Arc, time::Duration};
use telemetry::prelude::*;
use telemetry_utils::metric;
use thiserror::Error;
use tokio::sync::{oneshot, Mutex};
use veritech_core::{
    ExecutionId, VeritechValueDecryptError, NATS_ACTION_RUN_DEFAULT_SUBJECT_SUFFIX,
    NATS_RESOLVER_FUNCTION_DEFAULT_SUBJECT_SUFFIX,
    NATS_SCHEMA_VARIANT_DEFINITION_DEFAULT_SUBJECT_SUFFIX, NATS_VALIDATION_DEFAULT_SUBJECT_SUFFIX,
    REPLY_INBOX_HEADER_NAME,
};

use crate::{app_state::AppState, request::DecryptRequest, Publisher, PublisherError};

pub use kill::process_kill_request;

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
    #[error("request deserializing error: {0}")]
    RequestDerializing(#[source] serde_json::Error),
    #[error("utf8 error when creating subject")]
    Utf8(#[from] Utf8Error),
    #[error("failed to decrypt request: {0}")]
    VeritechValueDecrypt(#[from] VeritechValueDecryptError),
}

type HandlerResult<T> = result::Result<T, HandlerError>;

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        error!(si.error.message = ?self, "failed to process message");
        Response::server_error()
    }
}

pub async fn process_request(
    State(state): State<AppState>,
    subject: Subject,
    Headers(maybe_headers): Headers,
    msg: InnerMessage,
) -> HandlerResult<()> {
    let reply_subject = match maybe_headers
        .and_then(|headers| headers.get(REPLY_INBOX_HEADER_NAME).map(|v| v.to_string()))
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
            (Some(_), Some(_), Some(_), Some(_), Some(_)) => {}
            _ => return Err(HandlerError::InvalidIncomingSubject(subject)),
        }
    } else {
        match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some(_), Some(_), Some(_), Some(_)) => {}
            _ => return Err(HandlerError::InvalidIncomingSubject(subject)),
        }
    }

    // Now that we have validated all subject parts beforehand, we can match on the kind.
    match (parts.next(), parts.next()) {
        (Some(NATS_ACTION_RUN_DEFAULT_SUBJECT_SUFFIX), None) => {
            let request: ActionRunRequest =
                serde_json::from_slice(&msg.payload).map_err(HandlerError::RequestDerializing)?;
            info!(execution_kind = %NATS_ACTION_RUN_DEFAULT_SUBJECT_SUFFIX, execution_id = %request.execution_id, "validated request and about to execute");
            action_run_request(state, request, reply_subject).await?
        }
        (Some(NATS_RESOLVER_FUNCTION_DEFAULT_SUBJECT_SUFFIX), None) => {
            let request: ResolverFunctionRequest =
                serde_json::from_slice(&msg.payload).map_err(HandlerError::RequestDerializing)?;
            info!(execution_kind = %NATS_RESOLVER_FUNCTION_DEFAULT_SUBJECT_SUFFIX, execution_id = %request.execution_id, "validated request and about to execute");
            resolver_function_request_task(state, request, reply_subject).await?
        }
        (Some(NATS_SCHEMA_VARIANT_DEFINITION_DEFAULT_SUBJECT_SUFFIX), None) => {
            let request: SchemaVariantDefinitionRequest =
                serde_json::from_slice(&msg.payload).map_err(HandlerError::RequestDerializing)?;
            info!(execution_kind = %NATS_SCHEMA_VARIANT_DEFINITION_DEFAULT_SUBJECT_SUFFIX, execution_id = %request.execution_id, "validated request and about to execute");
            schema_variant_definition_request(state, request, reply_subject).await?
        }
        (Some(NATS_VALIDATION_DEFAULT_SUBJECT_SUFFIX), None) => {
            let request: ValidationRequest =
                serde_json::from_slice(&msg.payload).map_err(HandlerError::RequestDerializing)?;
            info!(execution_kind = %NATS_VALIDATION_DEFAULT_SUBJECT_SUFFIX, execution_id = %request.execution_id, "validated request and about to execute");
            validation_request(state, request, reply_subject).await?
        }
        _ => return Err(HandlerError::InvalidIncomingSubject(subject)),
    }

    Ok(())
}

#[instrument(name = "veritech.action_run_request", level = "info", skip_all)]
async fn action_run_request(
    state: AppState,
    mut payload_request: ActionRunRequest,
    reply_mailbox: Subject,
) -> HandlerResult<()> {
    let span = Span::current();

    let mut client = state
        .cyclone_pool
        .get()
        .await
        .map_err(|err| span.record_err(HandlerError::CyclonePool(Box::new(err))))?;
    metric!(counter.function_run.action = 1);

    let mut sensitive_strings = SensitiveStrings::default();
    // Decrypt the relevant contents of the request and track any resulting sensitive strings
    // to be redacted
    payload_request.decrypt(&mut sensitive_strings, &state.decryption_key)?;

    // NOTE(nick,fletcher): we need to create a owned client here because publisher has its own lifetime. Yeehaw.
    let nats_for_publisher = state.nats.clone();
    let publisher = Publisher::new(&nats_for_publisher, &reply_mailbox);

    let execution_id = payload_request.execution_id.clone();
    let cyclone_request = CycloneRequest::from_parts(payload_request, sensitive_strings);

    let (kill_sender, kill_receiver) = oneshot::channel::<()>();
    {
        state
            .kill_senders
            .lock()
            .await
            .insert(execution_id.to_owned(), kill_sender);
    }

    let unstarted_progress = client
        .prepare_action_run_execution(cyclone_request)
        .await
        .map_err(|err| {
            metric!(counter.function_run.action = -1);
            span.record_err(err)
        })?;

    let progress_loop = async {
        let mut progress = unstarted_progress.start().await.map_err(|err| {
            metric!(counter.function_run.action = -1);
            span.record_err(err)
        })?;

        while let Some(msg) = progress.next().await {
            match msg {
                Ok(ProgressMessage::OutputStream(output)) => {
                    publisher.publish_output(&output).await.map_err(|err| {
                        metric!(counter.function_run.action = -1);
                        span.record_err(err)
                    })?;
                }
                Ok(ProgressMessage::Heartbeat) => {
                    trace!("received heartbeat message");
                }
                Err(err) => {
                    warn!(error = ?err, "next progress message was an error, bailing out");
                    break;
                }
            }
        }
        publisher.finalize_output().await.map_err(|err| {
            metric!(counter.function_run.action = -1);
            span.record_err(err)
        })?;

        let function_result = progress.finish().await.map_err(|err| {
            metric!(counter.function_run.action = -1);
            span.record_err(err)
        })?;

        HandlerResult::Ok(function_result)
    };

    // we do not want to return errors at this point as it will retry functions that may have
    // failed for legitimate reasons and should not be retried
    let timeout = state.cyclone_client_execution_timeout;
    let result = tokio::select! {
        _ = tokio::time::sleep(timeout) => {
            error!("hit timeout for communicating with cyclone server");
            kill_sender_remove_blocking(&state.kill_senders, execution_id).await?;
            Err(HandlerError::CycloneTimeout(
                timeout,
            ))
        },
        Ok(_) = kill_receiver => {
            Err(HandlerError::Killed(execution_id))
        }
        func_result = progress_loop => {
            kill_sender_remove_blocking(&state.kill_senders, execution_id).await?;
            func_result
        },
    };

    match result {
        Ok(function_result) => {
            if let Err(err) = publisher.publish_result(&function_result).await {
                metric!(counter.function_run.action = -1);
                error!(error = ?err, "failed to publish errored result");
            }

            metric!(counter.function_run.action = -1);
            span.record_ok();
        }
        Err(HandlerError::CycloneTimeout(timeout)) => {
            metric!(counter.function_run.action = -1);
            warn!(error = ?timeout, "timed out trying to run function to completion");
        }
        Err(HandlerError::Killed(execution_id)) => {
            metric!(counter.function_run.action = -1);
            info!(error = ?execution_id, "function killed during execution via signal");
        }
        Err(err) => {
            metric!(counter.function_run.action = -1);
            error!(error = ?err, "failure trying to run function to completion");
        }
    }
    Ok(())
}

async fn resolver_function_request_task(
    state: AppState,
    cyclone_request: ResolverFunctionRequest,
    reply_mailbox: Subject,
) -> HandlerResult<()> {
    let execution_id = cyclone_request.execution_id.clone();

    // NOTE(nick,fletcher,scott): we need to create a owned client here because publisher has its own lifetime. Yeehaw.
    let nats_for_publisher = state.nats.clone();
    let publisher = Publisher::new(&nats_for_publisher, &reply_mailbox);

    let function_result = match resolver_function_request(state, &publisher, cyclone_request).await
    {
        Ok(fr) => fr,
        Err(HandlerError::CyclonePool(err)) => return Err(HandlerError::CyclonePool(err)),
        Err(err) => {
            dbg!(&err);
            error!(error = ?err, "failure trying to run function to completion");
            si_pool_noodle::FunctionResult::Failure::<ResolverFunctionResultSuccess>(
                FunctionResultFailure::new_for_veritech_server_error(
                    execution_id.clone(),
                    err.to_string(),
                    timestamp(),
                ),
            )
        }
    };

    if let Err(err) = publisher.finalize_output().await {
        error!(error = ?err, "failed to finalize output by sending final message");
        let result = si_pool_noodle::FunctionResult::Failure::<ResolverFunctionResultSuccess>(
            FunctionResultFailure::new_for_veritech_server_error(
                execution_id,
                "failed to finalize output by sending final message",
                timestamp(),
            ),
        );
        if let Err(err) = publisher.publish_result(&result).await {
            error!(error = ?err, "failed to publish errored result");
        }
        return Ok(());
    }

    if let Err(err) = publisher.publish_result(&function_result).await {
        error!(error = ?err, "failed to publish result");
    };

    Ok(())
}

#[instrument(name = "veritech.resolver_function_request", level = "info", skip_all)]
async fn resolver_function_request(
    state: AppState,
    publisher: &Publisher<'_>,
    mut request: ResolverFunctionRequest,
) -> HandlerResult<FunctionResult<ResolverFunctionResultSuccess>> {
    let span = Span::current();

    let mut client = state
        .cyclone_pool
        .get()
        .await
        .map_err(|err| span.record_err(HandlerError::CyclonePool(Box::new(err))))?;
    metric!(counter.function_run.resolver = 1);

    let mut sensitive_strings = SensitiveStrings::default();
    // Decrypt the relevant contents of the request and track any resulting sensitive strings
    // to be redacted
    request.decrypt(&mut sensitive_strings, &state.decryption_key)?;

    let execution_id = request.execution_id.clone();
    let cyclone_request = CycloneRequest::from_parts(request, sensitive_strings);

    let (kill_sender, kill_receiver) = oneshot::channel::<()>();
    {
        state
            .kill_senders
            .lock()
            .await
            .insert(execution_id.to_owned(), kill_sender);
    }

    let unstarted_progress = client
        .prepare_resolver_execution(cyclone_request)
        .await
        .map_err(|err| {
            metric!(counter.function_run.resolver = -1);
            span.record_err(err)
        })?;

    let progress_loop = async {
        let mut progress = unstarted_progress
            .start()
            .await
            .map_err(|err| span.record_err(err))?;

        while let Some(msg) = progress.next().await {
            match msg {
                Ok(ProgressMessage::OutputStream(output)) => {
                    publisher.publish_output(&output).await.map_err(|err| {
                        metric!(counter.function_run.resolver = -1);
                        span.record_err(err)
                    })?
                }
                Ok(ProgressMessage::Heartbeat) => {
                    trace!("received heartbeat message");
                }
                Err(err) => {
                    warn!(error = ?err, "next progress message was an error, bailing out");
                    break;
                }
            }
        }

        let function_result = progress.finish().await.map_err(|err| {
            metric!(counter.function_run.resolver = -1);
            span.record_err(err)
        })?;

        HandlerResult::Ok(function_result)
    };

    let timeout = state.cyclone_client_execution_timeout;
    let function_result = tokio::select! {
        _ = tokio::time::sleep(timeout) => {
            error!(?timeout, "hit timeout for communicating with cyclone server");
            kill_sender_remove_blocking(&state.kill_senders, execution_id).await?;
            Err(HandlerError::CycloneTimeout(
                timeout,
            ))
        },
        Ok(_) = kill_receiver => {
            Err(HandlerError::Killed(execution_id))
        }
        func_result = progress_loop => {
            kill_sender_remove_blocking(&state.kill_senders, execution_id).await?;
            func_result
        },
    }?;

    metric!(counter.function_run.resolver = -1);
    span.record_ok();
    Ok(function_result)
}

#[instrument(
    name = "veritech.schema_variant_definition_request",
    level = "info",
    skip_all
)]
async fn schema_variant_definition_request(
    state: AppState,
    mut payload_request: SchemaVariantDefinitionRequest,
    reply_mailbox: Subject,
) -> HandlerResult<()> {
    let span = Span::current();

    let mut client = state
        .cyclone_pool
        .get()
        .await
        .map_err(|err| span.record_err(HandlerError::CyclonePool(Box::new(err))))?;
    metric!(counter.function_run.schema_variant_definition = 1);

    let mut sensitive_strings = SensitiveStrings::default();
    // Decrypt the relevant contents of the request and track any resulting sensitive strings
    // to be redacted
    payload_request.decrypt(&mut sensitive_strings, &state.decryption_key)?;

    // NOTE(nick,fletcher): we need to create a owned client here because publisher has its own lifetime. Yeehaw.
    let nats_for_publisher = state.nats.clone();
    let publisher = Publisher::new(&nats_for_publisher, &reply_mailbox);

    let execution_id = payload_request.execution_id.clone();
    let cyclone_request = CycloneRequest::from_parts(payload_request, sensitive_strings);

    let (kill_sender, kill_receiver) = oneshot::channel::<()>();
    {
        state
            .kill_senders
            .lock()
            .await
            .insert(execution_id.to_owned(), kill_sender);
    }

    let unstarted_progress = client
        .prepare_schema_variant_definition_execution(cyclone_request)
        .await
        .map_err(|err| {
            metric!(counter.function_run.schema_variant_definition = -1);
            span.record_err(err)
        })?;

    let progress_loop = async {
        let mut progress = unstarted_progress.start().await.map_err(|err| {
            metric!(counter.function_run.schema_variant_definition = -1);
            span.record_err(err)
        })?;

        while let Some(msg) = progress.next().await {
            match msg {
                Ok(ProgressMessage::OutputStream(output)) => {
                    publisher.publish_output(&output).await.map_err(|err| {
                        metric!(counter.function_run.schema_variant_definition = -1);
                        span.record_err(err)
                    })?;
                }
                Ok(ProgressMessage::Heartbeat) => {
                    trace!("received heartbeat message");
                }
                Err(err) => {
                    warn!(error = ?err, "next progress message was an error, bailing out");
                    break;
                }
            }
        }
        publisher.finalize_output().await.map_err(|err| {
            metric!(counter.function_run.schema_variant_definition = -1);
            span.record_err(err)
        })?;

        let function_result = progress.finish().await.map_err(|err| {
            metric!(counter.function_run.schema_variant_definition = -1);
            span.record_err(err)
        })?;

        HandlerResult::Ok(function_result)
    };

    // we do not want to return errors at this point as it will retry functions that may have
    // failed for legitimate reasons and should not be retried
    let timeout = state.cyclone_client_execution_timeout;
    let result = tokio::select! {
        _ = tokio::time::sleep(timeout) => {
            error!("hit timeout for communicating with cyclone server");
            kill_sender_remove_blocking(&state.kill_senders, execution_id).await?;
            Err(HandlerError::CycloneTimeout(
                timeout,
            ))
        },
        Ok(_) = kill_receiver => {
            Err(HandlerError::Killed(execution_id))
        }
        func_result = progress_loop => {
            kill_sender_remove_blocking(&state.kill_senders, execution_id).await?;
            func_result
        },
    };

    match result {
        Ok(function_result) => {
            if let Err(err) = publisher.publish_result(&function_result).await {
                metric!(counter.function_run.schema_variant_definition = -1);
                error!(error = ?err, "failed to publish errored result");
            }

            metric!(counter.function_run.schema_variant_definition = -1);
            span.record_ok();
        }
        Err(HandlerError::CycloneTimeout(timeout)) => {
            metric!(counter.function_run.schema_variant_definition = -1);
            warn!(error = ?timeout, "timed out trying to run function to completion");
        }
        Err(HandlerError::Killed(execution_id)) => {
            metric!(counter.function_run.schema_variant_definition = -1);
            info!(error = ?execution_id, "function killed during execution via signal");
        }
        Err(err) => {
            metric!(counter.function_run.schema_variant_definition = -1);
            error!(error = ?err, "failure trying to run function to completion");
        }
    }

    Ok(())
}

#[instrument(name = "veritech.validation_request", level = "info", skip_all)]
async fn validation_request(
    state: AppState,
    mut payload_request: ValidationRequest,
    reply_mailbox: Subject,
) -> HandlerResult<()> {
    let span = Span::current();

    let mut client = state
        .cyclone_pool
        .get()
        .await
        .map_err(|err| span.record_err(HandlerError::CyclonePool(Box::new(err))))?;
    metric!(counter.function_run.validation = 1);

    let mut sensitive_strings = SensitiveStrings::default();
    // Decrypt the relevant contents of the request and track any resulting sensitive strings
    // to be redacted
    payload_request.decrypt(&mut sensitive_strings, &state.decryption_key)?;

    // NOTE(nick,fletcher): we need to create a owned client here because publisher has its own lifetime. Yeehaw.
    let nats_for_publisher = state.nats.clone();
    let publisher = Publisher::new(&nats_for_publisher, &reply_mailbox);

    let execution_id = payload_request.execution_id.clone();
    let cyclone_request = CycloneRequest::from_parts(payload_request, sensitive_strings);

    let (kill_sender, kill_receiver) = oneshot::channel::<()>();
    {
        state
            .kill_senders
            .lock()
            .await
            .insert(execution_id.to_owned(), kill_sender);
    }

    let unstarted_progress = client
        .prepare_validation_execution(cyclone_request)
        .await
        .map_err(|err| {
            metric!(counter.function_run.validation = -1);
            span.record_err(err)
        })?;

    let progress_loop = async {
        let mut progress = unstarted_progress.start().await.map_err(|err| {
            metric!(counter.function_run.validation = -1);
            span.record_err(err)
        })?;

        while let Some(msg) = progress.next().await {
            match msg {
                Ok(ProgressMessage::OutputStream(output)) => {
                    publisher.publish_output(&output).await.map_err(|err| {
                        metric!(counter.function_run.validation = -1);
                        span.record_err(err)
                    })?;
                }
                Ok(ProgressMessage::Heartbeat) => {
                    trace!("received heartbeat message");
                }
                Err(err) => {
                    warn!(error = ?err, "next progress message was an error, bailing out");
                    break;
                }
            }
        }
        publisher.finalize_output().await.map_err(|err| {
            metric!(counter.function_run.validation = -1);
            span.record_err(err)
        })?;

        let function_result = progress.finish().await.map_err(|err| {
            metric!(counter.function_run.validation = -1);
            span.record_err(err)
        })?;

        HandlerResult::Ok(function_result)
    };

    // we do not want to return errors at this point as it will retry functions that may have
    // failed for legitimate reasons and should not be retried
    let timeout = state.cyclone_client_execution_timeout;
    let result = tokio::select! {
        _ = tokio::time::sleep(timeout) => {
            error!("hit timeout for communicating with cyclone server");
            kill_sender_remove_blocking(&state.kill_senders, execution_id).await?;
            Err(HandlerError::CycloneTimeout(
                timeout,
            ))
        },
        Ok(_) = kill_receiver => {
            Err(HandlerError::Killed(execution_id))
        }
        func_result = progress_loop => {
            kill_sender_remove_blocking(&state.kill_senders, execution_id).await?;
            func_result
        },
    };

    match result {
        Ok(function_result) => {
            if let Err(err) = publisher.publish_result(&function_result).await {
                metric!(counter.function_run.action = -1);
                error!(error = ?err, "failed to publish errored result");
            }

            metric!(counter.function_run.validation = -1);
            span.record_ok();
        }
        Err(HandlerError::CycloneTimeout(timeout)) => {
            metric!(counter.function_run.validation = -1);
            warn!(error = ?timeout, "timed out trying to run function to completion");
        }
        Err(HandlerError::Killed(execution_id)) => {
            metric!(counter.function_run.validation = -1);
            info!(error = ?execution_id, "function killed during execution via signal");
        }
        Err(err) => {
            metric!(counter.function_run.validation = -1);
            error!(error = ?err, "failure trying to run function to completion");
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
    let span = Span::current();

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
