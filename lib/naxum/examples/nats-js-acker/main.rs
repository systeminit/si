use std::{
    collections::{hash_map::Entry, HashMap},
    env, error,
    str::{self, Utf8Error},
    sync::Arc,
    time::Duration,
};

use async_nats::jetstream;
use naxum::{
    extract::State,
    handler::Handler,
    middleware::{ack::AckLayer, trace::TraceLayer},
    response::{IntoResponse, Response},
    BoxError, ServiceExt,
};
use thiserror::Error;
use tokio::{
    signal::unix::{self, SignalKind},
    sync::Mutex,
    time,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tower::ServiceBuilder;
use tracing::{error, info};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt as _,
    util::SubscriberInitExt as _,
    EnvFilter, Registry,
};

const TRACING_LOG_ENV_VAR: &str = "SI_LOG";
const DEFAULT_TRACING_DIRECTIVES: &str = "nats_js_acker=trace,naxum=trace,info";

const MAX_RETRIES: u8 = 3;

#[derive(Clone, Debug, Default)]
struct AppState {
    retries: Arc<Mutex<HashMap<String, u8>>>,
}

#[derive(Debug, Error)]
enum AppError {
    #[error("utf8 error: {0}")]
    Utf8(#[from] Utf8Error),
    #[error("unprocessable")]
    Unprocessable,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!(error = ?self, "failed to process message");
        Response::server_error()
    }
}

async fn default(State(state): State<AppState>, msg: async_nats::Message) -> Result<(), AppError> {
    info!(subject = msg.subject.as_str(), headers = ?msg.headers, "processing message");

    // Introduce some simulated "work" time
    time::sleep(Duration::from_millis(10)).await;

    let payload = str::from_utf8(&msg.payload)?;

    // If subject ends with `.fail` then use the body as an ID and track 3 retries, simulating a
    // failure until the last retry which will succeed. Remember, returning an error in this
    // handler triggers the `Ack` middleware to `nack()` this message and it will be immediately
    // redelivered.
    if "fail" == msg.subject.as_str().split('.').last().unwrap_or_default() {
        let mut retries = state.retries.lock().await;
        match retries.entry(payload.to_string()) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() -= 1;
                if *entry.get() > 0 {
                    error!(retries = *entry.get(), "failed to process message");
                    return Err(AppError::Unprocessable);
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(MAX_RETRIES);
                error!(retries = MAX_RETRIES, "failed to process message");
                return Err(AppError::Unprocessable);
            }
        }
    }

    info!(payload, "finished message");

    Ok(())
}

async fn handle_error(err: BoxError) {
    if err.is::<tower::timeout::error::Elapsed>() {
        error!(error = ?err, "message took too long to process");
    } else {
        error!(error = ?err, "unknown error");
    }
}

#[allow(clippy::disallowed_methods)] // env vars are supporting alternatives in an example
#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    Registry::default()
        .with(
            EnvFilter::try_from_env(TRACING_LOG_ENV_VAR)
                .unwrap_or_else(|_| EnvFilter::new(DEFAULT_TRACING_DIRECTIVES)),
        )
        .with(
            fmt::layer()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .pretty(),
        )
        .try_init()?;

    let url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_owned());
    let subject = env::var("NATS_SUBJECT").unwrap_or_else(|_| "naxum.test.js.>".to_owned());
    let stream_name = env::var("NATS_STREAM").unwrap_or_else(|_| "NAXUM_TEST".to_owned());

    // Create a NATS client, JetStream context, a consumer, and finally an async `Stream` of
    // messages
    let client = async_nats::connect(url).await?;
    let context = jetstream::new(client);
    let stream = context
        .get_or_create_stream(jetstream::stream::Config {
            name: stream_name.clone(),
            subjects: vec![subject.clone()],
            ..Default::default()
        })
        .await?;
    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config::default())
        .await?;
    let messages = consumer.messages().await?;

    // Setup a Tower `Service` stack with some middleware
    let app = ServiceBuilder::new()
        // Only pull one message at a time, regardless of the consumer ack_pending policy. This is
        // only to help track tracing output to limit the concurrent work.
        .concurrency_limit(1)
        // Add tracing for each incoming message request
        .layer(TraceLayer::new())
        // Enforce a timeout on the total execution time of the request
        .timeout(Duration::from_millis(100))
        // Add `Ack` middleware which manages ack/nack/progress acking around the handler service
        .layer(AckLayer::new().progress_period(Duration::from_millis(5)))
        // Create a handler service with app state to respond to each incoming message request
        .service(default.with_state(AppState::default()))
        // Handle middleware errors, namely for the timeout middleware which can fail with an
        // elapsed error
        .handle_error(handle_error);

    // Use a Tokio `TaskTracker` and `CancellationToken` to support signal handling and graceful
    // shutdown
    let tracker = TaskTracker::new();
    let token = CancellationToken::new();

    let naxum_token = token.clone();
    tracker.spawn(async move {
        info!(
            stream = stream_name.as_str(),
            subject = subject.as_str(),
            "ready to receive messages with a nats jetstream consumer",
        );
        naxum::serve(messages, app.into_make_service())
            .with_graceful_shutdown(naxum::wait_on_cancelled(naxum_token))
            .await
    });

    // Create streams of `SIGINT` (i.e. `Ctrl+c`) and `SIGTERM` signals
    let mut sig_int = unix::signal(SignalKind::interrupt())?;
    let mut sig_term = unix::signal(SignalKind::terminate())?;

    // Wait until one of the signal streams gets a signal, after which we will close the task
    // tracker and cancel the token, signaling all holders of the token
    tokio::select! {
        _ = sig_int.recv() => {
            info!("received SIGINT, performing graceful shutdown");
            tracker.close();
            token.cancel();
        }
        _ = sig_term.recv() => {
            info!("received SIGTERM, performing graceful shutdown");
            tracker.close();
            token.cancel();
        }
    }

    // Wait for all tasks to finish
    tracker.wait().await;

    info!("graceful shutdown complete");
    Ok(())
}
