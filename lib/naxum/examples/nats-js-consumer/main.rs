use std::{env, error, str, time::Duration};

use async_nats::jetstream;
use naxum::{
    extract::State, handler::Handler, middleware::trace::TraceLayer, BoxError, ServiceExt,
};
use tokio::{
    signal::unix::{self, SignalKind},
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
const DEFAULT_TRACING_DIRECTIVES: &str = "nats_js_consumer=trace,naxum=trace,info";

#[derive(Clone, Debug)]
struct AppState {}

async fn default(
    State(_state): State<AppState>,
    msg: jetstream::Message,
) -> naxum::response::Result<()> {
    info!(subject = msg.subject.as_str(), "processing message");

    time::sleep(Duration::from_millis(10)).await;
    let payload = str::from_utf8(&msg.payload).expect("TODO");

    msg.ack().await.expect("TODO");

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
        .concurrency_limit(500)
        .layer(TraceLayer::new())
        .timeout(Duration::from_millis(100))
        .service(default.with_state(AppState {}))
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
