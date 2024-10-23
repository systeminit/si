use std::future::IntoFuture;

use naxum::{
    extract::State,
    handler::Handler,
    middleware::{ack::AckLayer, matched_subject::MatchedSubjectLayer, trace::TraceLayer},
    response::{IntoResponse, Response},
    Message, MessageHead, ServiceBuilder, ServiceExt, TowerServiceExt,
};
use si_data_nats::{async_nats, jetstream, Client, Subject};
use telemetry::tracing::error;
use telemetry_nats::propagation;
use thiserror::Error;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

pub const FINAL_MESSAGE_HEADER_KEY: &str = "X-Final-Message";

#[derive(Debug, Error)]
pub enum NaxumForwarderError {}

type Result<T> = std::result::Result<T, NaxumForwarderError>;

#[derive(Debug)]
pub struct NaxumForwarder;

impl NaxumForwarder {
    pub async fn new(
        nats: Client,
        concurrency_limit: usize,
        token: CancellationToken,
        source_stream: async_nats::jetstream::stream::Stream,
        source_subject: Subject,
        destination_subject: Subject,
    ) -> Result<()> {
        let connection_metadata = nats.metadata_clone();
        let context = jetstream::new(nats);

        let state = AppState::new(context, destination_subject, token);

        // Create a consumer from the source stream.
        let incoming = {
            source_stream
                .create_consumer(async_nats::jetstream::consumer::pull::Config {
                    durable_name: Some(format!(
                        "forwarder-{source_subject}-to-{destination_subject}"
                    )),
                    filter_subject: source_subject.to_string(),
                    ..Default::default()
                })
                .await?
                .messages()
                .await?
        };

        // Modify the stream to take until we see a final message.
        let incoming = incoming.take_while(|msg| match msg {
            Ok(msg) => {
                if let Some(headers) = msg.headers() {
                    if headers.get(FINAL_MESSAGE_HEADER_KEY).is_some() {
                        return false;
                    }
                }
                true
            }
            Err(err) => {
                error!(?err, "error with message taken from stream");
                true
            }
        });

        let app = ServiceBuilder::new()
            .layer(
                TraceLayer::new()
                    .make_span_with(
                        telemetry_nats::NatsMakeSpan::builder(connection_metadata).build(),
                    )
                    .on_response(telemetry_nats::NatsOnResponse::new()),
            )
            .layer(AckLayer::new())
            .service(handler.with_state(state))
            .map_response(Response::into_response);

        let inner =
            naxum::serve_with_incoming_limit(incoming, app.into_make_service(), concurrency_limit)
                .with_graceful_shutdown(naxum::wait_on_cancelled(token));

        let _ = Box::new(inner.into_future());

        Ok(())
    }
}

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum HandlerError {}

type HandlerResult<T> = std::result::Result<T, HandlerError>;

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        error!(si.error.message = ?self, "failed to process message");
        Response::default_internal_server_error()
    }
}

pub(crate) async fn handler(
    State(state): State<AppState>,
    msg: Message<async_nats::Message>,
) -> HandlerResult<()> {
    let ack = state
        .context
        .publish_with_headers(
            state.destination_subject,
            propagation::empty_injected_headers(),
            msg.payload,
        )
        .await?;
    ack.await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub context: jetstream::Context,
    pub destination_subject: Subject,
    pub token: CancellationToken,
}

impl AppState {
    pub fn new(
        context: jetstream::Context,
        destination_subject: Subject,
        token: CancellationToken,
    ) -> Self {
        Self {
            context,
            destination_subject,
            token,
        }
    }
}
