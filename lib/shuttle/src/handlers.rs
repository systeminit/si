use naxum::{
    extract::State,
    response::{IntoResponse, Response},
    Message,
};
use si_data_nats::async_nats::{self, jetstream};
use telemetry::tracing::error;
use telemetry_nats::propagation;
use thiserror::Error;

use crate::{app_state::AppState, FINAL_MESSAGE_HEADER_KEY};

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum HandlerError {
    #[error("error publishing message: {0}")]
    NatsPublish(#[from] async_nats::jetstream::context::PublishError),
}

type HandlerResult<T> = std::result::Result<T, HandlerError>;

pub(crate) async fn default(
    State(state): State<AppState>,
    msg: Message<jetstream::Message>,
) -> HandlerResult<()> {
    if let Some(headers) = msg.headers() {
        if headers.get(FINAL_MESSAGE_HEADER_KEY).is_some() {
            state.token.cancel();
            return Ok(());
        }
    }

    let ack = state
        .context
        .publish_with_headers(
            state.destination_subject,
            propagation::empty_injected_headers(),
            msg.payload.to_owned(),
        )
        .await?;
    ack.await?;

    Ok(())
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        error!(si.error.message = ?self, "failed to process message");
        Response::default_internal_server_error()
    }
}
