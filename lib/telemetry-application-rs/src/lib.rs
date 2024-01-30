#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
// TODO(fnichol): document all, then drop `missing_errors_doc`
#![allow(clippy::missing_errors_doc)]

use std::{borrow::Cow, env, io, ops::Deref, time::Duration};

use derive_builder::Builder;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    resource::{EnvResourceDetector, OsResourceDetector, ProcessResourceDetector},
    runtime,
    trace::{self, Tracer},
    Resource,
};
use opentelemetry_semantic_conventions::resource;
use telemetry::{
    opentelemetry::{global, trace::TraceError},
    tracing::{debug, info, trace, warn, Subscriber},
    TelemetryCommand, TracingLevel, Verbosity,
};
use thiserror::Error;
use tokio::{
    signal::unix::{self, SignalKind},
    sync::mpsc,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing_subscriber::{
    filter::ParseError,
    fmt::format::FmtSpan,
    layer::SubscriberExt as _,
    reload,
    util::{SubscriberInitExt as _, TryInitError},
    EnvFilter, Layer, Registry,
};

pub use telemetry::tracing;
pub use telemetry::{ApplicationTelemetryClient, TelemetryClient};

pub mod prelude {
    pub use super::TelemetryConfig;
    pub use telemetry::prelude::*;
    pub use telemetry::{ApplicationTelemetryClient, TelemetryClient};
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    DirectivesParse(#[from] ParseError),
    #[error("error creating signal handler: {0}")]
    Signal(#[source] io::Error),
    #[error("failed to parse span event fmt token: {0}")]
    SpanEventParse(String),
    #[error(transparent)]
    Trace(#[from] TraceError),
    #[error(transparent)]
    TryInit(#[from] TryInitError),
    #[error(transparent)]
    Update(#[from] reload::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Builder, Debug, Default)]
pub struct TelemetryConfig {
    #[builder(setter(into), default = r#"env!("CARGO_PKG_NAME")"#)]
    service_name: &'static str,

    #[builder(setter(into), default = r#"env!("CARGO_PKG_VERSION")"#)]
    service_version: &'static str,

    #[allow(dead_code)]
    #[builder(setter(into))]
    service_namespace: &'static str,

    #[builder(default)]
    app_modules: Vec<&'static str>,

    #[builder(setter(into, strip_option), default = "None")]
    custom_default_tracing_level: Option<String>,

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
        default = "self.default_log_span_events_env_var()?"
    )]
    log_span_events_env_var: Option<String>,

    #[builder(
        setter(into, strip_option),
        default = "self.default_secondary_log_env_var()"
    )]
    secondary_log_env_var: Option<String>,

    #[builder(
        setter(into, strip_option),
        default = "self.default_secondary_log_span_events_env_var()"
    )]
    secondary_log_span_events_env_var: Option<String>,

    #[builder(default = "true")]
    signal_handlers: bool,
}

impl TelemetryConfig {
    #[must_use]
    pub fn builder() -> TelemetryConfigBuilder {
        TelemetryConfigBuilder::default()
    }
}

impl TelemetryConfigBuilder {
    fn default_log_env_var_prefix(
        &self,
    ) -> std::result::Result<Option<String>, TelemetryConfigBuilderError> {
        match &self.service_namespace {
            Some(service_namespace) => Ok(Some(service_namespace.to_uppercase())),
            None => Err(TelemetryConfigBuilderError::ValidationError(
                "service_namespace must be set".to_string(),
            )),
        }
    }

    fn default_log_env_var(
        &self,
    ) -> std::result::Result<Option<String>, TelemetryConfigBuilderError> {
        match (&self.log_env_var_prefix, &self.service_name) {
            (Some(Some(prefix)), Some(service_name)) => Ok(Some(format!(
                "{}_{}_LOG",
                prefix.to_uppercase(),
                service_name.to_uppercase()
            ))),
            (Some(None) | None, Some(service_name)) => {
                Ok(Some(format!("{}_LOG", service_name.to_uppercase())))
            }
            (None | Some(_), None) => Err(TelemetryConfigBuilderError::ValidationError(
                "service_name must be set".to_string(),
            )),
        }
    }

    fn default_log_span_events_env_var(
        &self,
    ) -> std::result::Result<Option<String>, TelemetryConfigBuilderError> {
        match (&self.log_env_var_prefix, &self.service_name) {
            (Some(Some(prefix)), Some(service_name)) => Ok(Some(format!(
                "{}_{}_LOG_SPAN_EVENTS",
                prefix.to_uppercase(),
                service_name.to_uppercase()
            ))),
            (Some(None) | None, Some(service_name)) => Ok(Some(format!(
                "{}_LOG_SPAN_EVENTS",
                service_name.to_uppercase()
            ))),
            (None | Some(_), None) => Err(TelemetryConfigBuilderError::ValidationError(
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

    fn default_secondary_log_span_events_env_var(&self) -> Option<String> {
        match &self.log_env_var_prefix {
            Some(Some(prefix)) => Some(format!("{}_LOG_SPAN_EVENTS", prefix.to_uppercase())),
            Some(None) | None => None,
        }
    }
}

pub fn init(
    config: TelemetryConfig,
    tracker: &TaskTracker,
    shutdown_token: CancellationToken,
) -> Result<ApplicationTelemetryClient> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracing_level = default_tracing_level(&config);
    let span_events_fmt = default_span_events_fmt(&config)?;

    let (subscriber, handles) = tracing_subscriber(&config, &tracing_level, span_events_fmt)?;
    subscriber.try_init()?;

    let client = create_client(config, tracing_level, handles, tracker, shutdown_token)?;

    Ok(client)
}

fn default_tracing_level(config: &TelemetryConfig) -> TracingLevel {
    if let Some(log_env_var) = config.log_env_var.as_deref() {
        #[allow(clippy::disallowed_methods)] // We use consistently named env var names, always
        // prefixed with `SI_`
        if let Ok(value) = env::var(log_env_var.to_uppercase()) {
            if !value.is_empty() {
                return TracingLevel::custom(value);
            }
        }
    }
    if let Some(log_env_var) = config.secondary_log_env_var.as_deref() {
        #[allow(clippy::disallowed_methods)] // We use consistently named env var names, always
        // prefixed with `SI_`
        if let Ok(value) = env::var(log_env_var.to_uppercase()) {
            if !value.is_empty() {
                return TracingLevel::custom(value);
            }
        }
    }

    if let Some(ref directives) = config.custom_default_tracing_level {
        TracingLevel::custom(directives)
    } else {
        TracingLevel::new(Verbosity::default(), Some(config.app_modules.as_ref()))
    }
}

fn default_span_events_fmt(config: &TelemetryConfig) -> Result<FmtSpan> {
    if let Some(log_span_events_env_var) = config.log_span_events_env_var.as_deref() {
        #[allow(clippy::disallowed_methods)] // We use consistently named env var names, always
        // prefixed with `SI_`
        if let Ok(value) = env::var(log_span_events_env_var.to_uppercase()) {
            if !value.is_empty() {
                return fmt_span_from_str(&value);
            }
        }
    }
    if let Some(log_env_var) = config.secondary_log_span_events_env_var.as_deref() {
        #[allow(clippy::disallowed_methods)] // We use consistently named env var names, always
        // prefixed with `SI_`
        if let Ok(value) = env::var(log_env_var.to_uppercase()) {
            if !value.is_empty() {
                return fmt_span_from_str(&value);
            }
        }
    }

    Ok(FmtSpan::NONE)
}

fn fmt_span_from_str(value: &str) -> Result<FmtSpan> {
    let mut filters = Vec::new();
    for filter in value.to_ascii_lowercase().split(',') {
        match filter.trim() {
            "new" => filters.push(FmtSpan::NEW),
            "enter" => filters.push(FmtSpan::ENTER),
            "exit" => filters.push(FmtSpan::EXIT),
            "close" => filters.push(FmtSpan::CLOSE),
            "active" => filters.push(FmtSpan::ACTIVE),
            "full" => filters.push(FmtSpan::FULL),
            invalid => return Err(Error::SpanEventParse(invalid.to_string())),
        };
    }

    Ok(filters
        .into_iter()
        .fold(FmtSpan::NONE, |acc, filter| filter | acc))
}

fn tracing_subscriber(
    config: &TelemetryConfig,
    tracing_level: &TracingLevel,
    span_events_fmt: FmtSpan,
) -> Result<(impl Subscriber + Send + Sync, TelemetryHandles)> {
    let directives = TracingDirectives::from(tracing_level);

    let (console_log_layer, console_log_filter_reload) = {
        let layer = tracing_subscriber::fmt::layer()
            .with_thread_ids(true)
            .with_span_events(span_events_fmt);

        let env_filter = EnvFilter::try_new(directives.as_str())?;
        let (filter, handle) = reload::Layer::new(env_filter);
        let layer = layer.with_filter(filter);

        let reloader =
            Box::new(move |updated: EnvFilter| handle.reload(updated).map_err(Into::into));

        (layer, reloader)
    };

    let (otel_layer, otel_filter_reload) = {
        let layer = tracing_opentelemetry::layer().with_tracer(otel_tracer(config)?);

        let env_filter = EnvFilter::try_new(directives.as_str())?;
        let (filter, handle) = reload::Layer::new(env_filter);
        let layer = layer.with_filter(filter);

        let reloader =
            Box::new(move |updated: EnvFilter| handle.reload(updated).map_err(Into::into));

        (layer, reloader)
    };

    let registry = Registry::default();
    let registry = registry.with(console_log_layer);
    let registry = registry.with(otel_layer);

    let handles = TelemetryHandles {
        console_log_filter_reload,
        otel_filter_reload,
    };

    Ok((registry, handles))
}

fn otel_tracer(config: &TelemetryConfig) -> std::result::Result<Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(trace::config().with_resource(telemetry_resource(config)))
        .install_batch(runtime::Tokio)
}

fn telemetry_resource(config: &TelemetryConfig) -> Resource {
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

fn create_client(
    config: TelemetryConfig,
    tracing_level: TracingLevel,
    handles: TelemetryHandles,
    tracker: &TaskTracker,
    shutdown_token: CancellationToken,
) -> Result<ApplicationTelemetryClient> {
    let (update_telemetry_tx, update_telemetry_rx) = mpsc::unbounded_channel();

    let client =
        ApplicationTelemetryClient::new(config.app_modules, tracing_level, update_telemetry_tx);

    tracker.spawn(
        TelemetryUpdateTask::new(handles, shutdown_token.clone(), update_telemetry_rx).run(),
    );
    if config.signal_handlers {
        tracker.spawn(
            TelemetrySignalHandlerTask::create(client.clone(), shutdown_token.clone())
                .map_err(Error::Signal)?
                .run(),
        );
    }
    tracker.spawn(TelemetryShutdownTask::new(shutdown_token).run());

    Ok(client)
}

type ReloadHandle = Box<dyn Fn(EnvFilter) -> Result<()> + Send + Sync>;

struct TelemetryHandles {
    console_log_filter_reload: ReloadHandle,
    otel_filter_reload: ReloadHandle,
}

struct TelemetrySignalHandlerTask {
    client: ApplicationTelemetryClient,
    shutdown_token: CancellationToken,
    sig_usr1: unix::Signal,
    sig_usr2: unix::Signal,
}

impl TelemetrySignalHandlerTask {
    const NAME: &'static str = "TelemetrySignalHandlerTask";

    fn create(
        client: ApplicationTelemetryClient,
        shutdown_token: CancellationToken,
    ) -> io::Result<Self> {
        let sig_usr1 = unix::signal(SignalKind::user_defined1())?;
        let sig_usr2 = unix::signal(SignalKind::user_defined2())?;

        Ok(Self {
            client,
            shutdown_token,
            sig_usr1,
            sig_usr2,
        })
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    debug!(task = Self::NAME, "received cancellation");
                    break;
                }
                Some(_) = self.sig_usr1.recv() => {
                    if let Err(err) = self.client.increase_verbosity().await {
                        warn!(
                            task = Self::NAME,
                            error = ?err,
                            "error while trying to increase verbosity",
                        );
                    }
                }
                Some(_) = self.sig_usr2.recv() => {
                    if let Err(err) = self.client.decrease_verbosity().await {
                        warn!(
                            task = Self::NAME,
                            error = ?err,
                            "error while trying to decrease verbosity",
                        );
                    }
                }
                else => {
                    // All other arms are closed, nothing let to do but return
                    trace!(task = Self::NAME, "all signal listeners have closed");
                    break;
                }
            }
        }

        debug!(task = Self::NAME, "shutdown complete");
    }
}

struct TelemetryUpdateTask {
    handles: TelemetryHandles,
    shutdown_token: CancellationToken,
    update_command_rx: mpsc::UnboundedReceiver<TelemetryCommand>,
}

impl TelemetryUpdateTask {
    const NAME: &'static str = "TelemetryUpdateTask";

    fn new(
        handles: TelemetryHandles,
        shutdown_token: CancellationToken,
        update_command_rx: mpsc::UnboundedReceiver<TelemetryCommand>,
    ) -> Self {
        Self {
            handles,
            shutdown_token,
            update_command_rx,
        }
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    debug!(task = Self::NAME, "received cancellation");
                    break;
                }
                Some(command) = self.update_command_rx.recv() => match command {
                    TelemetryCommand::TracingLevel(tracing_level) => {
                        if let Err(err) = self.update_tracing_level(tracing_level) {
                            warn!(
                                task = Self::NAME,
                                error = ?err,
                                "failed to update tracing level, using prior value",
                            );
                        }
                    }
                },
                else => {
                    trace!(task = Self::NAME, "update command stream has closed");
                    break;
                }
            }
        }

        debug!(task = Self::NAME, "shutdown complete");
    }

    fn update_tracing_level(&self, tracing_level: TracingLevel) -> Result<()> {
        let directives = TracingDirectives::from(tracing_level);

        (self.handles.console_log_filter_reload)(EnvFilter::try_new(directives.as_str())?)?;
        (self.handles.otel_filter_reload)(EnvFilter::try_new(directives.as_str())?)?;

        info!(
            task = Self::NAME,
            "updated tracing levels to: {:?}",
            directives.as_str()
        );

        Ok(())
    }
}

struct TelemetryShutdownTask {
    shutdown_token: CancellationToken,
}

impl TelemetryShutdownTask {
    const NAME: &'static str = "TelemetryShutdownTask";

    fn new(shutdown_token: CancellationToken) -> Self {
        Self { shutdown_token }
    }

    async fn run(self) {
        self.shutdown_token.cancelled().await;

        debug!(task = Self::NAME, "received cancellation");
        // TODO(fnichol): call to `shutdown_tracer_provider` blocks forever when called, causing
        // the services to not gracefully shut down in time.
        //
        // See: https://github.com/open-telemetry/opentelemetry-rust/issues/1395
        //
        // telemetry::opentelemetry::global::shutdown_tracer_provider();
        debug!(task = Self::NAME, "shutdown complete");
    }
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
                        .map(|m| format!("{m}=info"))
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
                        .map(|m| format!("{m}=debug"))
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
                        .map(|m| format!("{m}=trace"))
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
                        .map(|m| format!("{m}=trace"))
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
                        .map(|m| format!("{m}=trace"))
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
