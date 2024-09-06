//! Provides a ["billing events"](billing_events) server via a future.

use std::future::{Future, IntoFuture as _};
use std::io;

use app_state::AppState;
use app_state::NoopAppState;
use billing_events::{BillingEventsError, BillingEventsWorkQueue};
use data_warehouse_stream_client::DataWarehouseStreamClient;
use naxum::handler::Handler as _;
use naxum::middleware::ack::AckLayer;
use naxum::middleware::trace::{DefaultMakeSpan, DefaultOnRequest, TraceLayer};
use naxum::{ServiceBuilder, ServiceExt as _};
use si_data_nats::async_nats::error::Error as AsyncNatsError;
use si_data_nats::async_nats::jetstream::consumer::pull::Stream;
use si_data_nats::async_nats::jetstream::consumer::StreamErrorKind;
use si_data_nats::NatsClient;
use si_data_nats::{
    async_nats::{self, jetstream::stream::ConsumerErrorKind},
    jetstream,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

mod app_state;
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
///
/// If a delivery stream name is not provided, billing evenets will not be published to the data
/// warehouse stream!
#[instrument(name = "billing_events_server.init.new", level = "info")]
pub async fn new(
    nats: NatsClient,
    delivery_stream_name: Option<String>,
    token: CancellationToken,
) -> BillingEventsServerResult<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
    let incoming = {
        let queue = BillingEventsWorkQueue::get_or_create(jetstream::new(nats)).await?;
        let consumer_subject = queue.workspace_update_subject("*");
        queue
            .stream()
            .await?
            .create_consumer(incoming_consumer_config(consumer_subject))
            .await?
            .messages()
            .await?
    };

    match delivery_stream_name {
        Some(delivery_stream_name) => {
            info!(%delivery_stream_name, "creating billing events server in data warehouse stream delivery mode...");
            let client = DataWarehouseStreamClient::new(delivery_stream_name).await;
            let state = AppState::new(client);
            build_app(state, incoming, token)
        }
        None => {
            info!("creating building events server in no-op mode...");
            let state = NoopAppState::new();
            build_noop_app(state, incoming, token)
        }
    }
}

fn build_app(
    state: AppState,
    incoming: Stream,
    token: CancellationToken,
) -> BillingEventsServerResult<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
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

fn build_noop_app(
    state: NoopAppState,
    incoming: Stream,
    token: CancellationToken,
) -> BillingEventsServerResult<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
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
        .service(handlers::process_request_noop.with_state(state));

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
