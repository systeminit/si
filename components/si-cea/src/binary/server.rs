use crate::error::{CeaError, CeaResult};
use opentelemetry::{api::Provider, sdk};
use tracing;
use tracing_opentelemetry::layer;
use tracing_subscriber::{self, fmt, layer::SubscriberExt, EnvFilter, Registry};

pub mod prelude {
    pub use super::setup_tracing;
    pub use crate::agent::finalized_listener::FinalizedListener;
    pub use si_data::{Db, Storable};
    pub use si_settings::Settings;
}

pub fn setup_tracing(service_name: &'static str) -> CeaResult<()> {
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_process(opentelemetry_jaeger::Process {
            service_name: service_name.into(),
            tags: Vec::new(),
        })
        .init()
        .map_err(|err| CeaError::TracingError(Box::new(err)))?;
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::Always),
            ..Default::default()
        })
        .build();

    let tracer = provider.get_tracer(service_name);

    let fmt_layer = fmt::Layer::default();
    let opentelemetry_layer = layer().with_tracer(tracer);
    let env_filter_layer = EnvFilter::from_default_env();

    let subscriber = Registry::default()
        .with(env_filter_layer)
        .with(fmt_layer)
        .with(opentelemetry_layer);

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|err| CeaError::TracingError(Box::new(err)))?;

    Ok(())
}
