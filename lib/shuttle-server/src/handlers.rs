use naxum::{
    Message,
    extract::State,
    response::{IntoResponse, Response},
};
use shuttle_core::DESTINATION_SUBJECT_SUFFIX_HEADER_KEY;
use si_data_nats::{
    Subject,
    async_nats::{self, jetstream},
};
use telemetry::tracing::error;
use telemetry_nats::propagation;
use thiserror::Error;

use crate::{FINAL_MESSAGE_HEADER_KEY, app_state::AppState};

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
    let destination_subject = match msg.headers() {
        Some(headers) => {
            if headers.get(FINAL_MESSAGE_HEADER_KEY).is_some() {
                state.self_shutdown_token.cancel();
                return Ok(());
            }

            if let Some(destination_subject_suffix) =
                headers.get(DESTINATION_SUBJECT_SUFFIX_HEADER_KEY)
            {
                Subject::from(format!(
                    "{}.{destination_subject_suffix}",
                    state.destination_subject
                ))
            } else {
                state.destination_subject
            }
        }
        None => state.destination_subject,
    };

    let ack = state
        .context
        .publish_with_headers(
            destination_subject,
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
