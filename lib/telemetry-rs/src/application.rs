use std::{borrow::Cow, env, io, ops::Deref, time::Duration};

use derive_builder::Builder;
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
use tokio::{signal::unix, sync::mpsc};
use tracing::{debug, info, warn, Subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{
    filter::ParseError, fmt::format::FmtSpan, layer::Layered, prelude::*, reload,
    util::TryInitError, EnvFilter, Registry,
};

use crate::{
    tracing::trace, Client, TelemetryClient, TracingLevel, UpdateOpenTelemetry, Verbosity,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    DirectivesParse(#[from] ParseError),
    #[error(transparent)]
    Trace(#[from] TraceError),
    #[error(transparent)]
    TryInit(#[from] TryInitError),
    #[error(transparent)]
    Update(#[from] reload::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Builder, Debug, Default)]
pub struct Config {
    #[builder(setter(into), default = r#"env!("CARGO_PKG_NAME").to_string()"#)]
    service_name: String,

    #[builder(setter(into), default = r#"env!("CARGO_PKG_VERSION").to_string()"#)]
    service_version: String,

    #[allow(dead_code)]
    #[builder(setter(into))]
    service_namespace: String,

    #[builder(default)]
    app_modules: Vec<&'static str>,

    #[allow(dead_code)]
    #[builder(
        setter(into, strip_option),
        default = "self.default_log_env_var_prefix()?"
    )]
    log_env_var_prefix: Option<String>,

    #[builder(setter(into, strip_option), default = "self.default_log_env_var()?")]
    log_env_var: Option<String>,

    #[builder(
        setter(into, strip_option),
        default = "self.default_secondary_log_env_var()"
    )]
    secondary_log_env_var: Option<String>,

    #[builder(default = "true")]
    enable_opentelemetry: bool,
}

impl Config {
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

impl ConfigBuilder {
    fn default_log_env_var_prefix(
        &self,
    ) -> std::result::Result<Option<String>, ConfigBuilderError> {
        match &self.service_namespace {
            Some(service_namespace) => Ok(Some(service_namespace.to_uppercase())),
            None => Err(ConfigBuilderError::ValidationError(
                "service_namespace must be set".to_string(),
            )),
        }
    }

    fn default_log_env_var(&self) -> std::result::Result<Option<String>, ConfigBuilderError> {
        match (&self.log_env_var_prefix, &self.service_name) {
            (Some(Some(prefix)), Some(service_name)) => Ok(Some(format!(
                "{}_{}_LOG",
                prefix.to_uppercase(),
                service_name.to_uppercase()
            ))),
            (Some(None) | None, Some(service_name)) => {
                Ok(Some(format!("{}_LOG", service_name.to_uppercase())))
            }
            (None | Some(_), None) => Err(ConfigBuilderError::ValidationError(
                "service_name must be set".to_string(),
            )),
        }
    }

    fn default_secondary_log_env_var(&self) -> Option<String> {
        match &self.log_env_var_prefix {
            Some(Some(prefix)) => Some(format!("{}_LOG", prefix.to_uppercase())),
            Some(None) | None => None,
        }
    }
}

type EnvLayerHandle = reload::Handle<Option<EnvFilter>, Registry>;

type OtelLayer = Option<
    OpenTelemetryLayer<
        Layered<reload::Layer<Option<EnvFilter>, Registry>, Registry, Registry>,
        Tracer,
    >,
>;

type OtelLayerHandler = reload::Handle<
    Option<
        OpenTelemetryLayer<
            Layered<reload::Layer<Option<EnvFilter>, Registry>, Registry, Registry>,
            Tracer,
        >,
    >,
    Layered<reload::Layer<Option<EnvFilter>, Registry>, Registry, Registry>,
>;

pub fn init(config: Config) -> Result<Client> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracing_level = default_tracing_level(&config);
    let (subscriber, env_handle, otel_handle, inner_otel_layer) =
        tracing_subscriber(&config, &tracing_level)?;
    subscriber.try_init()?;
    let telemetry_client = start_telemetry_update_tasks(
        config,
        tracing_level,
        env_handle,
        otel_handle,
        inner_otel_layer,
    );

    Ok(telemetry_client)
}

fn default_tracing_level(config: &Config) -> TracingLevel {
    if let Some(log_env_var) = config.log_env_var.as_deref() {
        if let Ok(value) = env::var(log_env_var.to_uppercase()) {
            if !value.is_empty() {
                return TracingLevel::custom(value);
            }
        }
    }
    if let Some(log_env_var) = config.secondary_log_env_var.as_deref() {
        if let Ok(value) = env::var(log_env_var.to_uppercase()) {
            if !value.is_empty() {
                return TracingLevel::custom(value);
            }
        }
    }

    TracingLevel::new(Verbosity::default(), Some(config.app_modules.as_ref()))
}

fn tracing_subscriber(
    config: &Config,
    tracing_level: &TracingLevel,
) -> Result<(
    impl Subscriber + Send + Sync,
    EnvLayerHandle,
    OtelLayerHandler,
    OtelLayer,
)> {
    let directives = TracingDirectives::from(tracing_level);
    let env_filter = EnvFilter::try_new(directives.as_str())?;
    let (env_filter_layer, env_handle) = reload::Layer::new(Some(env_filter));

    let (otel_layer, otel_handle) = reload::Layer::new(Some(
        tracing_opentelemetry::layer().with_tracer(try_tracer(config)?),
    ));
    let mut inner_otel_layer = None;
    if !config.enable_opentelemetry {
        otel_handle.modify(|layer| {
            inner_otel_layer = layer.take();
        })?;
    }

    let registry = Registry::default()
        .with(env_filter_layer)
        .with(otel_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_thread_ids(true)
                .with_span_events(FmtSpan::NONE),
        );

    Ok((registry, env_handle, otel_handle, inner_otel_layer))
}

fn try_tracer(config: &Config) -> std::result::Result<Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_env())
        .with_trace_config(trace::config().with_resource(telemetry_resource(config)))
        .install_batch(opentelemetry::runtime::Tokio)
}

fn telemetry_resource(config: &Config) -> Resource {
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
        resource::SERVICE_NAME.string(config.service_name.to_string()),
        resource::SERVICE_VERSION.string(config.service_version.to_string()),
        resource::SERVICE_NAMESPACE.string("si"),
    ]))
}

pub fn start_tracing_level_signal_handler_task(client: &Client) -> io::Result<()> {
    let user_defined1 = unix::signal(unix::SignalKind::user_defined1())?;
    let user_defined2 = unix::signal(unix::SignalKind::user_defined2())?;
    drop(tokio::spawn(tracing_level_signal_handler_task(
        client.clone(),
        user_defined1,
        user_defined2,
    )));
    Ok(())
}

async fn tracing_level_signal_handler_task(
    mut client: Client,
    mut user_defined1: unix::Signal,
    mut user_defined2: unix::Signal,
) {
    loop {
        tokio::select! {
            _ = user_defined1.recv() => {
                if let Err(err) = client.increase_verbosity().await {
                    warn!(error = ?err, "error while trying to increase verbosity");
                }
            }
            _ = user_defined2.recv() => {
                if let Err(err) = client.decrease_verbosity().await {
                    warn!(error = ?err, "error while trying to decrease verbosity");
                }
            }
            else => {
                // All other arms are closed, nothing let to do but return
                trace!("returning from tracing level signal handler with all select arms closed");
            }
        }
    }
}

fn start_telemetry_update_tasks(
    config: Config,
    tracing_level: TracingLevel,
    env_handle: EnvLayerHandle,
    otel_handle: OtelLayerHandler,
    otel_layer: OtelLayer,
) -> Client {
    let (env_handle_tx, env_handle_rx) = mpsc::channel(2);
    drop(tokio::spawn(update_tracing_level_task(
        env_handle,
        env_handle_rx,
    )));
    let (otel_handle_tx, otel_handle_rx) = mpsc::channel(2);
    drop(tokio::spawn(update_opentelemetry_task(
        otel_handle,
        otel_layer,
        otel_handle_rx,
    )));

    Client::new(
        config.app_modules,
        tracing_level,
        env_handle_tx,
        otel_handle_tx,
    )
}

async fn update_tracing_level_task(
    layer_handle: EnvLayerHandle,
    mut rx: mpsc::Receiver<TracingLevel>,
) {
    while let Some(tracing_level) = rx.recv().await {
        if let Err(err) = update_tracing_level(&layer_handle, tracing_level) {
            warn!(error = ?err, "failed to update tracing level, using prior value");
            continue;
        }
    }
    debug!("update_tracing_level_task received closed channel, ending task");
}

fn update_tracing_level(layer_handle: &EnvLayerHandle, tracing_level: TracingLevel) -> Result<()> {
    let directives = TracingDirectives::from(tracing_level);
    let updated = EnvFilter::try_new(directives.as_str())?;

    layer_handle.modify(|layer| {
        layer.replace(updated);
    })?;
    info!("updated tracing levels to: {:?}", directives.as_str());

    Ok(())
}

async fn update_opentelemetry_task(
    layer_handle: OtelLayerHandler,
    mut otel_layer: OtelLayer,
    mut rx: mpsc::Receiver<UpdateOpenTelemetry>,
) {
    while let Some(update) = rx.recv().await {
        if let Err(err) = update_opentelemetry(&layer_handle, &mut otel_layer, update) {
            warn!(error = ?err, "failed to update opentelemetry, using prior setting");
            continue;
        }
    }
    debug!("update_opentelemetry_task received closed channel, ending task");
}

fn update_opentelemetry(
    layer_handle: &OtelLayerHandler,
    otel_layer: &mut OtelLayer,
    update: UpdateOpenTelemetry,
) -> Result<()> {
    match (update, &otel_layer) {
        (UpdateOpenTelemetry::Enable, Some(_)) => {
            layer_handle.modify(|layer| {
                *layer = otel_layer.take();
            })?;
            info!("enabled opentelemetry");
        }
        (UpdateOpenTelemetry::Disable, None) => {
            layer_handle.modify(|layer| {
                *otel_layer = layer.take();
            })?;
            info!("disabled opentelemetry");
        }

        (UpdateOpenTelemetry::Enable, None) => {
            debug!("opentelemtry already enabled, continuing");
        }
        (UpdateOpenTelemetry::Disable, Some(_)) => {
            debug!("opentelemtry already disabled, continuing");
        }
    }

    Ok(())
}

struct TracingDirectives(Cow<'static, str>);

impl From<TracingLevel> for TracingDirectives {
    fn from(value: TracingLevel) -> Self {
        match value {
            TracingLevel::Verbosity {
                verbosity,
                app_modules,
            } => Self::new(verbosity, &app_modules),
            TracingLevel::Custom(custom) => custom.into(),
        }
    }
}

impl From<&TracingLevel> for TracingDirectives {
    fn from(value: &TracingLevel) -> Self {
        match value {
            TracingLevel::Verbosity {
                verbosity,
                app_modules,
            } => Self::new(*verbosity, app_modules),
            TracingLevel::Custom(custom) => custom.clone().into(),
        }
    }
}

impl TracingDirectives {
    fn new(verbosity: Verbosity, app_modules: &Option<Vec<Cow<'static, str>>>) -> Self {
        let directives = match verbosity {
            Verbosity::InfoAll => match &app_modules {
                Some(mods) => Cow::Owned(format!(
                    "{},{}",
                    "info",
                    mods.iter()
                        .map(|m| format!("{}=info", m))
                        .collect::<Vec<_>>()
                        .join(",")
                )),
                None => Cow::Borrowed("info"),
            },
            Verbosity::DebugAppAndInfoAll => match &app_modules {
                Some(mods) => Cow::Owned(format!(
                    "{},{}",
                    "info",
                    mods.iter()
                        .map(|m| format!("{}=debug", m))
                        .collect::<Vec<_>>()
                        .join(",")
                )),
                None => Cow::Borrowed("debug"),
            },
            Verbosity::TraceAppAndInfoAll => match &app_modules {
                Some(mods) => Cow::Owned(format!(
                    "{},{}",
                    "info",
                    mods.iter()
                        .map(|m| format!("{}=trace", m))
                        .collect::<Vec<_>>()
                        .join(",")
                )),
                None => Cow::Borrowed("trace"),
            },
            Verbosity::TraceAppAndDebugAll => match &app_modules {
                Some(mods) => Cow::Owned(format!(
                    "{},{}",
                    "debug",
                    mods.iter()
                        .map(|m| format!("{}=trace", m))
                        .collect::<Vec<_>>()
                        .join(",")
                )),
                None => Cow::Borrowed("trace"),
            },
            Verbosity::TraceAll => match &app_modules {
                Some(mods) => Cow::Owned(format!(
                    "{},{}",
                    "trace",
                    mods.iter()
                        .map(|m| format!("{}=trace", m))
                        .collect::<Vec<_>>()
                        .join(",")
                )),
                None => Cow::Borrowed("trace"),
            },
        };

        Self(directives)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<String> for TracingDirectives {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl From<&'static str> for TracingDirectives {
    fn from(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl Deref for TracingDirectives {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
