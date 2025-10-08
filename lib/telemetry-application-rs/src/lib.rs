#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
// TODO(fnichol): document all, then drop `missing_errors_doc`
#![allow(clippy::missing_errors_doc)]

use std::{
    borrow::Cow,
    env,
    future::{
        Future,
        IntoFuture,
    },
    io::{
        self,
        IsTerminal,
    },
    ops::Deref,
    path::{
        Path,
        PathBuf,
    },
    pin::Pin,
    result,
    sync::Arc,
    thread,
    time::{
        Duration,
        Instant,
    },
};

use console_subscriber::ConsoleLayer;
use derive_builder::Builder;
use logroller::{
    LogRoller,
    LogRollerBuilder,
    Rotation,
    RotationSize,
};
use opentelemetry_sdk::{
    Resource,
    metrics::SdkMeterProvider,
    propagation::TraceContextPropagator,
    resource::EnvResourceDetector,
    runtime,
    trace::{
        self,
        Config,
        Tracer,
    },
};
use opentelemetry_semantic_conventions::resource;
pub use telemetry::{
    ApplicationTelemetryClient,
    TelemetryClient,
    tracing,
};
use telemetry::{
    TelemetryCommand,
    TracingLevel,
    Verbosity,
    opentelemetry::{
        KeyValue,
        global::{
            self,
        },
        metrics::MetricsError,
        trace::{
            TraceError,
            TracerProvider,
        },
    },
    prelude::*,
    tracing::{
        Event,
        Subscriber,
        subscriber::Interest,
    },
};
use thiserror::Error;
use tokio::{
    signal::unix::{
        self,
        SignalKind,
    },
    sync::{
        mpsc,
        oneshot,
    },
    time,
};
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};
use tracing::Metadata;
use tracing_appender::non_blocking::{
    self,
    NonBlocking,
};
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::{
    EnvFilter,
    Layer,
    Registry,
    filter::{
        FilterExt,
        ParseError,
    },
    fmt::{
        self,
        format::{
            DefaultFields,
            FmtSpan,
            Format,
            Json,
            JsonFields,
        },
    },
    layer::{
        Context,
        Filter,
        SubscriberExt,
    },
    registry::LookupSpan,
    reload,
    util::{
        SubscriberInitExt,
        TryInitError,
    },
};

pub mod prelude {
    pub use telemetry::{
        ApplicationTelemetryClient,
        TelemetryClient,
        prelude::*,
    };

    pub use super::{
        LogFormat,
        TelemetryConfig,
    };
}

// Rust crates that will not output span or event telemetry, no matter what the default level is
// set to. In other words, each of these crates/modules will have `MODULE=off` as their value.
const DEFAULT_NEVER_MODULES: &[&str] = &["h2", "hyper"];

#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("directives parse error: {0}")]
    DirectivesParse(#[from] ParseError),
    #[error("file appender error: {0}")]
    FileAppender(#[from] logroller::LogRollerError),
    #[error("metrics error {0}")]
    Metrics(#[from] MetricsError),
    #[error("error creating signal handler: {0}")]
    Signal(#[source] io::Error),
    #[error("failed to parse span event fmt token: {0}")]
    SpanEventParse(String),
    #[error("trace error: {0}")]
    Trace(#[from] TraceError),
    #[error("try init error: {0}")]
    TryInit(#[from] TryInitError),
    #[error("update error: {0}")]
    Update(#[from] reload::Error),
}

type Result<T> = result::Result<T, Error>;

#[derive(Clone, Builder, Debug, Default)]
pub struct TelemetryConfig {
    #[builder(setter(into), default = r#"env!("CARGO_PKG_NAME")"#)]
    service_name: &'static str,

    #[builder(setter(into), default = r#"env!("CARGO_PKG_VERSION")"#)]
    service_version: &'static str,

    #[allow(dead_code)]
    #[builder(setter(into))]
    service_namespace: &'static str,

    #[builder(setter(each(name = "app_module"), into), default)]
    app_modules: Vec<&'static str>,

    #[builder(setter(each(name = "interesting_module"), into), default)]
    interesting_modules: Vec<&'static str>,

    #[builder(
        setter(each(name = "never_module"), into),
        default = "self.default_never_modules()"
    )]
    never_modules: Vec<&'static str>,

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

    #[builder(setter(into), default = "self.default_no_color()")]
    no_color: Option<bool>,

    #[builder(setter(into), default = "None")]
    force_color: Option<bool>,

    #[builder(setter(into), default)]
    log_format: LogFormat,

    #[builder(setter(into), default = "None")]
    log_file_directory: Option<PathBuf>,

    #[builder(setter(into), default = "false")]
    tokio_console: bool,

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
    fn default_never_modules(&self) -> Vec<&'static str> {
        DEFAULT_NEVER_MODULES.to_vec()
    }

    fn default_log_env_var_prefix(
        &self,
    ) -> result::Result<Option<String>, TelemetryConfigBuilderError> {
        match &self.service_namespace {
            Some(service_namespace) => Ok(Some(service_namespace.to_uppercase())),
            None => Err(TelemetryConfigBuilderError::ValidationError(
                "service_namespace must be set".to_string(),
            )),
        }
    }

    fn default_log_env_var(&self) -> result::Result<Option<String>, TelemetryConfigBuilderError> {
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
    ) -> result::Result<Option<String>, TelemetryConfigBuilderError> {
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

    fn default_no_color(&self) -> Option<bool> {
        // Checks a known/standard var as a fallback. Code upstack will check for an `SI_*`
        // prefixed version which should have a higher precendence.
        //
        // See: <http://no-color.org/>
        #[allow(clippy::disallowed_methods)] // See rationale in comment above
        std::env::var_os("NO_COLOR").map(|value| !value.is_empty())
    }
}

pub fn init(
    config: TelemetryConfig,
    tracker: &TaskTracker,
    shutdown_token: CancellationToken,
) -> Result<(ApplicationTelemetryClient, TelemetryShutdownGuard)> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracing_level = default_tracing_level(&config);
    let span_events_fmt = default_span_events_fmt(&config)?;

    let (subscriber, handles) = tracing_subscriber(&config, &tracing_level, span_events_fmt)?;
    subscriber.try_init()?;

    debug!(
        ?config,
        directives = TracingDirectives::from(&tracing_level).as_str(),
        "telemetry configuration"
    );

    if config.tokio_console {
        warn!("tokio-console support is enabled; this could impact production performance");
    }

    let (client, guard) = create_client(config, tracing_level, handles, tracker, shutdown_token)?;

    Ok((client, guard))
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
        TracingLevel::new(
            Verbosity::default(),
            Some(config.app_modules.as_ref()),
            Some(config.interesting_modules.as_ref()),
            Some(config.never_modules.as_ref()),
        )
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

#[inline]
fn file_appender(config: &TelemetryConfig, directory: &Path) -> Result<LogRoller> {
    let filename = format!("{}.log", config.service_name);

    LogRollerBuilder::new(directory, Path::new(&filename))
        .rotation(Rotation::SizeBased(RotationSize::MB(128)))
        .build()
        .map_err(Into::into)
}

fn tracing_subscriber(
    config: &TelemetryConfig,
    tracing_level: &TracingLevel,
    span_events_fmt: FmtSpan,
) -> Result<(impl Subscriber + Send + Sync + use<>, TelemetryHandles)> {
    let directives = TracingDirectives::from(tracing_level);
    let shared_env_filter = SharedEnvFilter::try_new(directives.as_str())?;

    let (console_writer, console_non_blocking_guard) =
        tracing_appender::non_blocking(std::io::stdout());

    // We can save on boxing everything by using `Option`s and register them unconditionally--very
    // cool trick!
    //
    // See: https://users.rust-lang.org/t/type-hell-in-tracing-multiple-output-layers/126764/13
    // See: https://github.com/MaterializeInc/materialize/blob/bcba457395c7b79cad9ac1cca7c8b4ad02508821/src/ore/src/tracing.rs#L511-L526

    let console_text = if matches!(config.log_format, LogFormat::Text) {
        let layer = text_layer(console_writer.clone(), span_events_fmt.clone())
            .with_ansi(should_add_ansi(config));
        let (filter, handle) = reload::Layer::new(shared_env_filter.clone());
        let layer = layer.with_filter(filter.and(ExcludeMetricsFilter));
        let reloader: ReloadHandle =
            Box::new(move |updated: SharedEnvFilter| handle.reload(updated).map_err(Into::into));

        (Some(layer), Some(reloader))
    } else {
        (None, None)
    };

    let console_json = if matches!(config.log_format, LogFormat::Json) {
        let layer = json_layer(console_writer.clone(), span_events_fmt.clone());
        let (filter, handle) = reload::Layer::new(shared_env_filter.clone());
        let layer = layer.with_filter(filter.and(ExcludeMetricsFilter));
        let reloader: ReloadHandle =
            Box::new(move |updated: SharedEnvFilter| handle.reload(updated).map_err(Into::into));

        (Some(layer), Some(reloader))
    } else {
        (None, None)
    };

    let file_json = if let Some(directory) = config.log_file_directory.as_deref() {
        let (file_writer, file_non_blocking_guard) =
            tracing_appender::non_blocking(file_appender(config, directory)?);

        let layer = json_layer(file_writer, span_events_fmt);
        let (filter, handle) = reload::Layer::new(shared_env_filter.clone());
        let layer = layer.with_filter(filter.and(ExcludeMetricsFilter));
        let reloader: ReloadHandle =
            Box::new(move |updated: SharedEnvFilter| handle.reload(updated).map_err(Into::into));

        (Some(layer), Some(reloader), Some(file_non_blocking_guard))
    } else {
        (None, None, None)
    };

    let (otel_layer, otel_filter_reload) = {
        let layer = tracing_opentelemetry::layer().with_tracer(otel_tracer(config)?);
        let (filter, handle) = reload::Layer::new(shared_env_filter.clone());
        let layer = layer.with_filter(filter.and(ExcludeMetricsFilter));

        let reloader =
            Box::new(move |updated: SharedEnvFilter| handle.reload(updated).map_err(Into::into));

        (layer, reloader)
    };

    let (metrics_layer, metrics_filter_reload) = {
        let metrics_provider = otel_metrics(config)?;
        global::set_meter_provider(metrics_provider.clone());
        let layer = MetricsLayer::new(metrics_provider);
        let (filter, handle) = reload::Layer::new(shared_env_filter.clone());
        let layer = layer.with_filter(filter.and(IncludeMetricsFilter));

        let reloader =
            Box::new(move |updated: SharedEnvFilter| handle.reload(updated).map_err(Into::into));

        (layer, reloader)
    };

    let tokio_console_layer = if config.tokio_console {
        let builder = ConsoleLayer::builder().with_default_env();
        let layer = builder.spawn();

        Some(layer)
    } else {
        None
    };

    let registry = Registry::default()
        .with(console_text.0)
        .with(console_json.0)
        .with(file_json.0)
        .with(otel_layer)
        .with(metrics_layer)
        .with(tokio_console_layer);

    let handles = TelemetryHandles {
        console_text_filter_reload: console_text.1,
        console_json_filter_reload: console_json.1,
        file_json_filter_reload: file_json.1,
        otel_filter_reload,
        metrics_filter_reload,
        _console_non_blocking_guard: console_non_blocking_guard,
        _file_non_blocking_guard: file_json.2,
    };

    Ok((registry, handles))
}

fn otel_tracer(config: &TelemetryConfig) -> Result<Tracer> {
    Ok(opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(Config::default().with_resource(telemetry_resource(config)))
        .with_batch_config(
            trace::BatchConfigBuilder::default()
                .with_max_queue_size(4096)
                .build(),
        )
        .install_batch(runtime::Tokio)?
        .tracer(config.service_name))
}

fn otel_metrics(config: &TelemetryConfig) -> result::Result<SdkMeterProvider, MetricsError> {
    opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            config.service_name,
        )]))
        .with_period(Duration::from_secs(1))
        .with_timeout(Duration::from_secs(10))
        .build()
}

fn text_layer<S>(
    writer: NonBlocking,
    span_events_fmt: FmtSpan,
) -> fmt::Layer<S, DefaultFields, Format, NonBlocking>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer::<S>()
        .with_thread_ids(true)
        .with_span_events(span_events_fmt)
        .with_writer(writer)
}

fn json_layer<S>(
    writer: NonBlocking,
    span_events_fmt: FmtSpan,
) -> fmt::Layer<S, JsonFields, Format<Json>, NonBlocking>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer::<S>()
        .json()
        .with_thread_ids(true)
        .with_span_events(span_events_fmt)
        .with_writer(writer)
}

fn telemetry_resource(config: &TelemetryConfig) -> Resource {
    // TODO(fnichol): create opentelemetry-resource-detector-aws for ec2 & eks detection
    Resource::from_detectors(
        Duration::from_secs(3),
        vec![Box::new(EnvResourceDetector::new())],
    )
    .merge(&Resource::new(vec![
        KeyValue::new(resource::SERVICE_NAME, config.service_name.to_string()),
        KeyValue::new(
            resource::SERVICE_VERSION,
            config.service_version.to_string(),
        ),
        KeyValue::new(resource::SERVICE_NAMESPACE, "si"),
    ]))
}

fn create_client(
    config: TelemetryConfig,
    tracing_level: TracingLevel,
    handles: TelemetryHandles,
    tracker: &TaskTracker,
    shutdown_token: CancellationToken,
) -> Result<(ApplicationTelemetryClient, TelemetryShutdownGuard)> {
    let (update_telemetry_tx, update_telemetry_rx) = mpsc::unbounded_channel();

    let client = ApplicationTelemetryClient::new(
        config.service_name.to_string().into_boxed_str(),
        config.app_modules,
        config.interesting_modules,
        config.never_modules,
        tracing_level,
        update_telemetry_tx.clone(),
    );

    // Initialize process metrics observer
    let process_metrics_observer = {
        let meter = telemetry::opentelemetry::global::meter(config.service_name);
        match si_otel_metrics::ProcessMetricsObserver::init(meter) {
            Ok(observer) => {
                telemetry::tracing::debug!("process metrics observer initialized");
                Some(observer)
            }
            Err(err) => {
                telemetry::tracing::warn!(error = ?err, "failed to initialize process metrics observer");
                None
            }
        }
    };

    let guard = TelemetryShutdownGuard {
        update_telemetry_tx,
        _process_metrics_observer: process_metrics_observer,
    };

    // Spawn this task free of the tracker as we want it to outlive the tracker when shutting down
    tokio::spawn(TelemetryUpdateTask::new(handles, update_telemetry_rx).run());

    if config.signal_handlers {
        tracker.spawn(
            TelemetrySignalHandlerTask::create(client.clone(), shutdown_token.clone())
                .map_err(Error::Signal)?
                .run(),
        );
    }

    Ok((client, guard))
}

fn should_add_ansi(config: &TelemetryConfig) -> bool {
    if config.force_color.filter(|fc| *fc).unwrap_or(false) {
        // If we're forcing colors, then this is unconditionally true
        true
    } else {
        // Otherwise 2 conditions must be met:
        // 1. did we *not* ask for `no_color` (or: is `no_color` unset)
        // 2. is the standard output file descriptor refer to a terminal or TTY
        !config.no_color.filter(|nc| *nc).unwrap_or(false) && io::stdout().is_terminal()
    }
}

#[remain::sorted]
#[derive(Copy, Clone, Debug, Default)]
pub enum LogFormat {
    Json,
    #[default]
    Text,
}

#[must_use]
pub struct TelemetryShutdownGuard {
    update_telemetry_tx: mpsc::UnboundedSender<TelemetryCommand>,
    _process_metrics_observer: Option<si_otel_metrics::ProcessMetricsObserver>,
}

impl TelemetryShutdownGuard {
    pub async fn wait(self) -> result::Result<(), telemetry::ClientError> {
        let token = CancellationToken::new();
        self.update_telemetry_tx
            .send(TelemetryCommand::Shutdown(token.clone()))?;
        token.cancelled().await;
        Ok(())
    }
}

impl IntoFuture for TelemetryShutdownGuard {
    type Output = result::Result<(), telemetry::ClientError>;

    type IntoFuture =
        Pin<Box<dyn Future<Output = result::Result<(), telemetry::ClientError>> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(IntoFuture::into_future(self.wait()))
    }
}

type ReloadHandle = Box<dyn Fn(SharedEnvFilter) -> Result<()> + Send + Sync>;

struct TelemetryHandles {
    console_text_filter_reload: Option<ReloadHandle>,
    console_json_filter_reload: Option<ReloadHandle>,
    file_json_filter_reload: Option<ReloadHandle>,
    otel_filter_reload: ReloadHandle,
    metrics_filter_reload: ReloadHandle,
    _console_non_blocking_guard: non_blocking::WorkerGuard,
    _file_non_blocking_guard: Option<non_blocking::WorkerGuard>,
}

struct TelemetrySignalHandlerTask {
    client: ApplicationTelemetryClient,
    shutdown_token: CancellationToken,
    sig_usr1: unix::Signal,
    sig_quit: unix::Signal,
}

impl TelemetrySignalHandlerTask {
    const NAME: &'static str = "TelemetrySignalHandlerTask";

    fn create(
        client: ApplicationTelemetryClient,
        shutdown_token: CancellationToken,
    ) -> io::Result<Self> {
        let sig_usr1 = unix::signal(SignalKind::user_defined1())?;
        let sig_quit = unix::signal(SignalKind::quit())?;

        Ok(Self {
            client,
            shutdown_token,
            sig_usr1,
            sig_quit,
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
                    if let Err(err) = self.client.modify_verbosity().await {
                        warn!(
                            task = Self::NAME,
                            error = ?err,
                            "error while trying to modify verbosity",
                        );
                    }
                }
                Some(_) = self.sig_quit.recv() => {
                    self.generate_process_report()
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

    #[cfg(target_os = "linux")]
    fn generate_process_report(&self) {
        const THREAD_NAME: &str = "generate-process-report";

        let report_writer = self::linux::ReportWriter {
            service_name: self.client.service_name().to_string().into_boxed_str(),
            deadline: std::time::Duration::from_secs(5),
            handle: tokio::runtime::Handle::current(),
        };

        if let Err(err) = thread::Builder::new()
            .name(THREAD_NAME.to_string())
            .spawn(move || report_writer.generate())
        {
            error!(si.error = ?err, "failed to spawn {THREAD_NAME} thread");
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn generate_process_report(&self) {
        info!("generating a process report is only supported on linux systems");
    }
}

struct TelemetryUpdateTask {
    handles: TelemetryHandles,
    update_command_rx: mpsc::UnboundedReceiver<TelemetryCommand>,
    is_shutdown: bool,
}

impl TelemetryUpdateTask {
    const NAME: &'static str = "TelemetryUpdateTask";

    fn new(
        handles: TelemetryHandles,
        update_command_rx: mpsc::UnboundedReceiver<TelemetryCommand>,
    ) -> Self {
        Self {
            handles,
            update_command_rx,
            is_shutdown: false,
        }
    }

    async fn run(mut self) {
        while let Some(command) = self.update_command_rx.recv().await {
            match command {
                TelemetryCommand::TracingLevel { level, wait } => {
                    // We want a span around the update logging so this is transmitted to our
                    // OpenTelemetry endpoint. We may use this span (and associated events) as a
                    // deployment mutation event, for example adding a mark in Honeycomb.
                    //
                    // Also note that we're using the `in_scope` method as none of the containing
                    // code is asynchronous--if there were async code then we'd use the
                    // `.instrument()` combinator on the future.
                    let span = info_span!("telemetry_update_task.update_tracing_level");
                    span.in_scope(|| {
                        if let Err(err) = self.update_tracing_level(level) {
                            warn!(
                                task = Self::NAME,
                                error = ?err,
                                "failed to update tracing level, using prior value",
                            );
                        }
                        if let Some(tx) = wait {
                            if let Err(err) = tx.send(()) {
                                warn!(
                                    error = ?err,
                                    "receiver already closed when waiting on changing tracing level",
                                );
                            }
                        }
                    })
                }
                TelemetryCommand::Shutdown(token) => {
                    if !self.is_shutdown {
                        Self::shutdown().await;
                    }
                    self.is_shutdown = true;
                    token.cancel();
                    break;
                }
            }
        }

        debug!(task = Self::NAME, "shutdown complete");
    }

    fn update_tracing_level(&self, tracing_level: TracingLevel) -> Result<()> {
        let directives = TracingDirectives::from(tracing_level);
        let shared_env_filter = SharedEnvFilter::try_new(directives.as_str())?;

        if let Some(reload) = &self.handles.console_text_filter_reload {
            (reload)(shared_env_filter.clone())?;
        }
        if let Some(reload) = &self.handles.console_json_filter_reload {
            (reload)(shared_env_filter.clone())?;
        }
        if let Some(reload) = &self.handles.file_json_filter_reload {
            (reload)(shared_env_filter.clone())?;
        }

        (self.handles.otel_filter_reload)(shared_env_filter.clone())?;
        (self.handles.metrics_filter_reload)(shared_env_filter.clone())?;

        info!(
            task = Self::NAME,
            directives = directives.as_str(),
            "updated tracing levels",
        );

        Ok(())
    }

    async fn shutdown() {
        // TODO(fnichol): call to `shutdown_tracer_provider` blocks forever when called, causing
        // the services to not gracefully shut down in time.
        //
        // So guess what we're going to? Spawn it off on a thread so we don't block Tokio's
        // reactor!
        //
        // See: https://github.com/open-telemetry/opentelemetry-rust/issues/1395

        let (tx, wait_on_shutdown) = oneshot::channel();

        let started_at = Instant::now();
        let _ = thread::spawn(move || {
            telemetry::opentelemetry::global::shutdown_tracer_provider();
            tx.send(()).ok();
        });

        let timeout = Duration::from_secs(5);
        match time::timeout(timeout, wait_on_shutdown).await {
            Ok(Ok(_)) => debug!(
                time_ns = (Instant::now() - started_at).as_nanos(),
                "opentelemetry shutdown"
            ),
            Ok(Err(_)) => trace!("opentelmetry shutdown sender already closed"),
            Err(_elapsed) => {
                warn!(
                    ?timeout,
                    "opentelemetry shutdown took too long, not waiting for full shutdown"
                );
            }
        };
    }
}

struct IncludeMetricsFilter;

impl<S> Filter<S> for IncludeMetricsFilter {
    fn enabled(
        &self,
        metadata: &Metadata<'_>,
        _: &tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        metadata
            .fields()
            .iter()
            .any(|field| field.name() == "metrics")
    }
}

struct ExcludeMetricsFilter;

impl<S> Filter<S> for ExcludeMetricsFilter {
    fn enabled(
        &self,
        metadata: &Metadata<'_>,
        _: &tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        !metadata
            .fields()
            .iter()
            .any(|field| field.name() == "metrics")
    }
}

struct TracingDirectives(Cow<'static, str>);

impl From<TracingLevel> for TracingDirectives {
    fn from(value: TracingLevel) -> Self {
        match value {
            TracingLevel::Verbosity {
                verbosity,
                app_modules,
                interesting_modules,
                never_modules,
            } => Self::new(
                verbosity,
                &app_modules,
                &interesting_modules,
                &never_modules,
            ),
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
                interesting_modules,
                never_modules,
            } => Self::new(*verbosity, app_modules, interesting_modules, never_modules),
            TracingLevel::Custom(custom) => custom.clone().into(),
        }
    }
}

impl TracingDirectives {
    fn new(
        verbosity: Verbosity,
        app_modules: &Option<Vec<Cow<'static, str>>>,
        interesting_modules: &Option<Vec<Cow<'static, str>>>,
        never_modules: &Option<Vec<Cow<'static, str>>>,
    ) -> Self {
        let app_str = |level: &str| {
            app_modules.as_ref().map(|arr| {
                arr.iter()
                    .map(|m| format!("{m}={level}"))
                    .collect::<Vec<_>>()
                    .join(",")
            })
        };
        let interesting_str = |level: &str| {
            interesting_modules.as_ref().map(|arr| {
                arr.iter()
                    .map(|m| format!("{m}={level}"))
                    .collect::<Vec<_>>()
                    .join(",")
            })
        };
        let never_str = never_modules.as_ref().map(|arr| {
            arr.iter()
                .map(|m| format!("{m}=off"))
                .collect::<Vec<_>>()
                .join(",")
        });

        let directives_for = |app_level: &'static str,
                              interesting_level: &'static str,
                              default_level: &'static str| {
            match (
                app_str(app_level),
                interesting_str(interesting_level),
                never_str,
            ) {
                (None, None, None) => Cow::Borrowed(default_level),
                (None, None, Some(never)) => Cow::Owned(format!("{never},{default_level}")),
                (None, Some(interesting), None) => {
                    Cow::Owned(format!("{interesting},{default_level}"))
                }
                (None, Some(interesting), Some(never)) => {
                    Cow::Owned(format!("{interesting},{never},{default_level}"))
                }
                (Some(app), None, None) => Cow::Owned(format!("{app},{default_level}")),
                (Some(app), None, Some(never)) => {
                    Cow::Owned(format!("{app},{never},{default_level}"))
                }
                (Some(app), Some(interesting), None) => {
                    Cow::Owned(format!("{app},{interesting},{default_level}"))
                }
                (Some(app), Some(interesting), Some(never)) => {
                    Cow::Owned(format!("{app},{interesting},{never},{default_level}"))
                }
            }
        };

        let directives = match verbosity {
            Verbosity::InfoAll => directives_for("info", "info", "info"),
            Verbosity::DebugAppInfoInterestingInfoAll => directives_for("debug", "info", "info"),
            Verbosity::DebugAppDebugInterestingInfoAll => directives_for("debug", "debug", "info"),
            Verbosity::TraceAppDebugInterestingInfoAll => directives_for("trace", "debug", "info"),
            Verbosity::TraceAppTraceInterestingInfoAll => directives_for("trace", "trace", "info"),
            Verbosity::TraceAppTraceInterestingDebugAll => {
                directives_for("trace", "trace", "debug")
            }
            Verbosity::TraceAll => directives_for("trace", "trace", "trace"),
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

#[derive(Clone, Debug)]
struct SharedEnvFilter(Arc<EnvFilter>);

impl SharedEnvFilter {
    fn try_new<S>(dirs: S) -> result::Result<Self, ParseError>
    where
        S: AsRef<str>,
    {
        let env_filter = EnvFilter::try_new(dirs)?;
        Ok(Self(Arc::new(env_filter)))
    }
}

impl<S> Filter<S> for SharedEnvFilter
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn enabled(&self, meta: &Metadata<'_>, cx: &Context<'_, S>) -> bool {
        <EnvFilter as Filter<S>>::enabled(&self.0, meta, cx)
    }

    fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest {
        <EnvFilter as Filter<S>>::callsite_enabled(&self.0, meta)
    }

    fn event_enabled(&self, event: &Event<'_>, cx: &Context<'_, S>) -> bool {
        <EnvFilter as Filter<S>>::event_enabled(&self.0, event, cx)
    }

    fn max_level_hint(&self) -> Option<tracing::level_filters::LevelFilter> {
        <EnvFilter as Filter<S>>::max_level_hint(&self.0)
    }

    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        <EnvFilter as Filter<S>>::on_new_span(&self.0, attrs, id, ctx);
    }

    fn on_record(&self, id: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>) {
        <EnvFilter as Filter<S>>::on_record(&self.0, id, values, ctx);
    }

    fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>) {
        <EnvFilter as Filter<S>>::on_enter(&self.0, id, ctx);
    }

    fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>) {
        <EnvFilter as Filter<S>>::on_exit(&self.0, id, ctx);
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        <EnvFilter as Filter<S>>::on_close(&self.0, id, ctx);
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::{
        fs::File,
        io::{
            self,
            BufWriter,
            Write as _,
        },
        path::{
            Path,
            PathBuf,
        },
        time::Duration,
    };

    use chrono::{
        SecondsFormat,
        Utc,
    };
    use telemetry::prelude::*;
    use tokio::{
        runtime::{
            Dump,
            Handle,
        },
        time,
    };

    pub(super) struct ReportWriter {
        pub(super) service_name: Box<str>,
        pub(super) deadline: Duration,
        pub(super) handle: Handle,
    }

    impl ReportWriter {
        const TEMP_ENV_VARS: &[&str] = &["TMPDIR", "TMP", "TEMP"];

        pub(super) fn generate(self) {
            let Self {
                service_name,
                deadline,
                handle,
            } = self;

            let Ok(dump) = handle.block_on(async { time::timeout(deadline, handle.dump()).await })
            else {
                warn!("generate process report deadline elapsed");
                return;
            };

            let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
            let report_path = PathBuf::from(
                #[allow(clippy::disallowed_methods)]
                // We want to check for env vars in use for temp
                Self::TEMP_ENV_VARS
                    .iter()
                    .map(|name| std::env::var(name).ok())
                    .find(|maybe_value| maybe_value.is_some())
                    .flatten()
                    .unwrap_or_else(|| "/tmp".to_string()),
            )
            .join(format!("{service_name}.report.{timestamp}.md"));

            let Ok(file) = File::create(&report_path) else {
                warn!("failed to create file {} for report", report_path.display());
                return;
            };
            let file = BufWriter::new(file);

            if let Err(err) =
                Self::write_report(dump, &service_name, &timestamp, &report_path, file)
            {
                warn!(
                    si.error = ?err,
                    report = ?report_path,
                    "failed to write to report file, aborting report",
                );
            }
        }

        fn write_report(
            dump: Dump,
            service_name: &str,
            timestamp: &str,
            report_path: &Path,
            mut file: BufWriter<File>,
        ) -> io::Result<()> {
            info!(report = ?report_path, "writing process report file");

            file.write_fmt(format_args!(
                "# {service_name} Report ({timestamp})\n\n## Tokio Task Traces\n\n"
            ))?;

            for (i, task) in dump.tasks().iter().enumerate() {
                file.write_fmt(format_args!("### Task {i} Trace\n```\n"))?;

                let trace = task.trace().to_string();
                file.write_all(trace.as_bytes())?;
                file.write_all(b"\n```\n\n")?;
            }

            file.flush()?;
            Ok(())
        }
    }
}
