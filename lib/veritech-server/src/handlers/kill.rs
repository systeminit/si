use naxum::{
    extract::{message_parts::Reply, State},
    Json,
};
use si_data_nats::Subject;
use si_pool_noodle::{FunctionResult, FunctionResultFailure, KillExecutionRequest};
use telemetry::prelude::*;

use crate::{app_state::KillAppState, Publisher};

use super::{kill_sender_remove_blocking, timestamp, HandlerError, HandlerResult};

pub async fn process_kill_request(
    State(state): State<KillAppState>,
    Reply(maybe_reply): Reply,
    Json(request): Json<KillExecutionRequest>,
) -> HandlerResult<()> {
    info!(execution_id = %request.execution_id, "received request to kill execution");

    let reply_mailbox = match maybe_reply {
        Some(reply) => reply,
        None => return Err(HandlerError::NoReplyInbox),
    };

    kill_execution_request_task(&state, request, reply_mailbox).await;

    Ok(())
}

async fn kill_execution_request_task(
    state: &KillAppState,
    request: KillExecutionRequest,
    reply_mailbox: Subject,
) {
    let publisher = Publisher::new(&state.nats, &reply_mailbox);

    let execution_id = request.execution_id;

    let result = match kill_execution_request(state, execution_id.to_owned()).await {
        Ok(()) => FunctionResult::Success(()),
        Err(err) => FunctionResult::Failure(FunctionResultFailure::new_for_veritech_server_error(
            execution_id,
            err.to_string(),
            timestamp(),
        )),
    };

    if let Err(err) = publisher.publish_result(&result).await {
        error!(?err, "failed to publish result");
    }
}

#[instrument(name = "veritech.kill_execution_request", level = "info", skip_all)]
async fn kill_execution_request(state: &KillAppState, execution_id: String) -> HandlerResult<()> {
    let span = Span::current();

    // NOTE(nick): in the instances of multiple veritechs, only one will have the kill sender.
    // Right now, we are returning a formal error here. We may want to reconsider this.
    let kill_sender = kill_sender_remove_blocking(&state.kill_senders, execution_id.to_owned())
        .await?
        .ok_or(HandlerError::MissingKillSender(execution_id.to_owned()))
        .map_err(|err| span.record_err(err))?;

    if kill_sender.send(()).is_err() {
        return Err(span.record_err(HandlerError::CouldNotSendKillSignal(execution_id)));
    }

    span.record_ok();
    Ok(())
}
