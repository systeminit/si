//! Provides a ["billing events"](billing_events) server via a future.

use std::future::Future;
use std::future::IntoFuture as _;
use std::io;

use billing_events::{BillingEventsError, BillingEventsWorkQueue};
use naxum::handler::Handler as _;
use naxum::middleware::ack::AckLayer;
use naxum::middleware::trace::{DefaultMakeSpan, DefaultOnRequest, TraceLayer};
use naxum::{ServiceBuilder, ServiceExt as _};
use si_data_nats::async_nats::error::Error as AsyncNatsError;
use si_data_nats::async_nats::jetstream::consumer::StreamErrorKind;
use si_data_nats::{
    async_nats::{self, jetstream::stream::ConsumerErrorKind},
    jetstream, NatsClient,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

mod handlers;

const CONCURRENCY_LIMIT: usize = 1000;
const CONSUMER_NAME: &str = "billing-events-server";

#[derive(Debug, Error)]
pub enum BillingEventsServerError {
    #[error("async nats consumer error: {0}")]
    AsyncNatsConsumer(#[from] AsyncNatsError<ConsumerErrorKind>),
    #[error("async nats stream error: {0}")]
    AsyncNatsStream(#[from] AsyncNatsError<StreamErrorKind>),
    #[error("billing events error: {0}")]
    BillingEvents(#[from] BillingEventsError),
}

type BillingEventsServerResult<T> = Result<T, BillingEventsServerError>;

/// Creates and returns a running ["billing events"](billing_events) server.
pub async fn new(
    nats: NatsClient,
    token: CancellationToken,
) -> BillingEventsServerResult<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
    let (state, incoming) = {
        let queue = BillingEventsWorkQueue::get_or_create(jetstream::new(nats)).await?;

        let state = AppState::new();
        let consumer_subject = queue.workspace_update_subject("*");

        (
            state,
            queue
                .stream()
                .await?
                .create_consumer(incoming_consumer_config(consumer_subject))
                .await?
                .messages()
                .await?,
        )
    };

    let app = ServiceBuilder::new()
        .layer(
            TraceLayer::new()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::TRACE))
                .on_response(
                    naxum::middleware::trace::DefaultOnResponse::new().level(Level::TRACE),
                ),
        )
        .layer(AckLayer::new())
        .service(handlers::process_request.with_state(state));

    let inner =
        naxum::serve_with_incoming_limit(incoming, app.into_make_service(), CONCURRENCY_LIMIT)
            .with_graceful_shutdown(naxum::wait_on_cancelled(token));

    Ok(Box::new(inner.into_future()))
}

#[inline]
fn incoming_consumer_config(
    subject: impl Into<String>,
) -> async_nats::jetstream::consumer::pull::Config {
    async_nats::jetstream::consumer::pull::Config {
        durable_name: Some(CONSUMER_NAME.to_owned()),
        filter_subject: subject.into(),
        ..Default::default()
    }
}

// NOTE(nick,jkeiser): we will likely use app state, so let's keep it around for now.
#[derive(Debug, Clone)]
struct AppState {}

impl AppState {
    pub fn new() -> Self {
        Self {}
    }
}
