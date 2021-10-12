use std::time::Duration;

use opentelemetry::{
    global,
    sdk::{
        propagation::TraceContextPropagator,
        resource::{EnvResourceDetector, OsResourceDetector, ProcessResourceDetector},
        trace::{self, Tracer},
        Resource,
    },
    trace::TraceError,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_semantic_conventions::resource;
use thiserror::Error;
use tracing_subscriber::{prelude::*, util::TryInitError, Registry};

#[derive(Debug, Error)]
pub enum TelemetryInitError {
    #[error("{0}")]
    Trace(#[from] TraceError),
    #[error("{0}")]
    TryInit(#[from] TryInitError),
}

pub fn init() -> Result<(), TelemetryInitError> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    tracing_subscriber()?.try_init()?;
    Ok(())
}

pub fn tracing_subscriber() -> Result<impl tracing::Subscriber + Send + Sync, TelemetryInitError> {
    Ok(Registry::default()
        .with(
            tracing_subscriber::EnvFilter::try_from_env("SI_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .with(tracing_opentelemetry::layer().with_tracer(try_tracer()?)))
}

fn try_tracer() -> Result<Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_env())
        .with_trace_config(trace::config().with_resource(telemetry_resource()))
        .install_batch(opentelemetry::runtime::Tokio)
}

fn telemetry_resource() -> Resource {
    // TODO(fnichol): create opentelemetry-resource-detector-aws for ec2 & eks detection
    Resource::from_detectors(
        Duration::from_secs(3),
        vec![
            Box::new(EnvResourceDetector::new()),
            Box::new(OsResourceDetector),
            Box::new(ProcessResourceDetector),
        ],
    )
    .merge(&Resource::new(vec![
        resource::SERVICE_NAME.string(env!("CARGO_PKG_NAME")),
        resource::SERVICE_VERSION.string(env!("CARGO_PKG_VERSION")),
        resource::SERVICE_NAMESPACE.string("si"),
    ]))
}
