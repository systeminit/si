use std::{
    future::{
        Future,
        IntoFuture as _,
    },
    io,
    sync::Arc,
};

use app_state::{
    AppState,
    NoopAppState,
};
use billing_events::{
    BillingEventsError,
    BillingEventsWorkQueue,
};
use data_warehouse_stream_client::{
    DataWarehouseStreamClient,
    DataWarehouseStreamClientError,
};
use naxum::{
    MessageHead,
    ServiceBuilder,
    ServiceExt as _,
    TowerServiceExt as _,
    extract::MatchedSubject,
    handler::Handler as _,
    middleware::{
        ack::AckLayer,
        matched_subject::{
            ForSubject,
            MatchedSubjectLayer,
        },
        trace::TraceLayer,
    },
    response::{
        IntoResponse,
        Response,
    },
};
use si_data_nats::{
    ConnectionMetadata,
    async_nats::{
        self,
        error::Error as AsyncNatsError,
        jetstream::{
            consumer::{
                StreamErrorKind,
                pull::Stream,
            },
            stream::ConsumerErrorKind,
        },
    },
    jetstream::Context,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

mod app_state;
mod handlers;

#[derive(Debug, Error)]
pub enum BillingEventsAppSetupError {
    #[error("async nats consumer error: {0}")]
    AsyncNatsConsumer(#[from] AsyncNatsError<ConsumerErrorKind>),
    #[error("async nats stream error: {0}")]
    AsyncNatsStream(#[from] AsyncNatsError<StreamErrorKind>),
    #[error("billing events error: {0}")]
    BillingEvents(#[from] BillingEventsError),
    #[error("data warehouse error: {0}")]
    DataWarehouse(#[from] DataWarehouseStreamClientError),
}

type Result<T> = std::result::Result<T, BillingEventsAppSetupError>;

#[instrument(
    name = "forklift.init.app.billing_events.build_and_run",
    level = "debug",
    skip_all
)]
pub(crate) async fn build_and_run(
    jetstream_context: Context,
    durable_consumer_name: String,
    connection_metadata: Arc<ConnectionMetadata>,
    concurrency_limit: usize,
    data_warehouse_stream_name: Option<&str>,
    token: CancellationToken,
) -> Result<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
    let incoming = {
        let queue = BillingEventsWorkQueue::get_or_create(jetstream_context).await?;
        let consumer_subject = queue.workspace_update_subject("*");
        queue
            .stream()
            .await?
            .create_consumer(async_nats::jetstream::consumer::pull::Config {
                durable_name: Some(durable_consumer_name),
                filter_subject: consumer_subject,
                ..Default::default()
            })
            .await?
            .messages()
            .await?
    };

    let inner = match data_warehouse_stream_name {
        Some(stream_name) => {
            info!(%stream_name, "creating billing events app in data warehouse stream delivery mode...");
            let client = DataWarehouseStreamClient::new(stream_name).await?;
            let state = AppState::new(client);
            build_app(
                state,
                connection_metadata,
                incoming,
                concurrency_limit,
                token.clone(),
            )?
        }
        None => {
            info!("creating billing events app in no-op mode...");
            let state = NoopAppState::new();
            build_noop_app(
                state,
                connection_metadata,
                incoming,
                concurrency_limit,
                token.clone(),
            )?
        }
    };

    Ok(inner)
}

#[instrument(
    name = "forklift.init.app.billing_events.build_and_run.build_app",
    level = "debug",
    skip_all
)]
fn build_app(
    state: AppState,
    connection_metadata: Arc<ConnectionMetadata>,
    incoming: Stream,
    concurrency_limit: usize,
    token: CancellationToken,
) -> Result<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
    let app = ServiceBuilder::new()
        .layer(MatchedSubjectLayer::new().for_subject(
            ForkliftBillingEventsForSubject::with_prefix(connection_metadata.subject_prefix()),
        ))
        .layer(
            TraceLayer::new()
                .make_span_with(telemetry_nats::NatsMakeSpan::builder(connection_metadata).build())
                .on_response(telemetry_nats::NatsOnResponse::new()),
        )
        .layer(AckLayer::new())
        .service(handlers::process_request.with_state(state))
        .map_response(Response::into_response);

    let inner =
        naxum::serve_with_incoming_limit(incoming, app.into_make_service(), concurrency_limit)
            .with_graceful_shutdown(naxum::wait_on_cancelled(token));

    Ok(Box::new(inner.into_future()))
}

#[instrument(
    name = "forklift.init.app.billing_events.build_and_run.build_noop_app",
    level = "debug",
    skip_all
)]
fn build_noop_app(
    state: NoopAppState,
    connection_metadata: Arc<ConnectionMetadata>,
    incoming: Stream,
    concurrency_limit: usize,
    token: CancellationToken,
) -> Result<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
    let app = ServiceBuilder::new()
        .layer(MatchedSubjectLayer::new().for_subject(
            ForkliftBillingEventsForSubject::with_prefix(connection_metadata.subject_prefix()),
        ))
        .layer(
            TraceLayer::new()
                .make_span_with(telemetry_nats::NatsMakeSpan::builder(connection_metadata).build())
                .on_response(telemetry_nats::NatsOnResponse::new()),
        )
        .layer(AckLayer::new())
        .service(handlers::process_request_noop.with_state(state))
        .map_response(Response::into_response);

    let inner =
        naxum::serve_with_incoming_limit(incoming, app.into_make_service(), concurrency_limit)
            .with_graceful_shutdown(naxum::wait_on_cancelled(token));

    Ok(Box::new(inner.into_future()))
}

#[derive(Clone, Debug)]
struct ForkliftBillingEventsForSubject {
    prefix: Option<()>,
}

impl ForkliftBillingEventsForSubject {
    fn with_prefix(prefix: Option<&str>) -> Self {
        Self {
            prefix: prefix.map(|_p| ()),
        }
    }
}

impl<R> ForSubject<R> for ForkliftBillingEventsForSubject
where
    R: MessageHead,
{
    fn call(&mut self, req: &mut naxum::Message<R>) {
        let mut parts = req.subject().split('.');

        match self.prefix {
            Some(_) => {
                if let (Some(prefix), Some(p1), Some(p2), Some(_workspace_id), None) = (
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                ) {
                    let matched = format!("{prefix}.{p1}.{p2}.:workspace_id");
                    req.extensions_mut().insert(MatchedSubject::from(matched));
                };
            }
            None => {
                if let (Some(p1), Some(p2), Some(_workspace_id), None) =
                    (parts.next(), parts.next(), parts.next(), parts.next())
                {
                    let matched = format!("{p1}.{p2}.:workspace_id");
                    req.extensions_mut().insert(MatchedSubject::from(matched));
                };
            }
        }
    }
}
