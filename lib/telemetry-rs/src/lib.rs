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
    fmt::{
        Debug,
        Display,
    },
    ops::{
        Deref,
        DerefMut,
    },
    result::Result,
    sync::Arc,
};

use async_trait::async_trait;
pub use opentelemetry::{
    self,
    trace::SpanKind,
};
use thiserror::Error;
use tokio::sync::{
    Mutex,
    mpsc,
    oneshot,
};
use tokio_util::sync::CancellationToken;
pub use tracing;
use tracing::warn;
pub use tracing_opentelemetry::OpenTelemetrySpanExt;

pub mod prelude {
    pub use tracing::{
        self,
        Id as SpanId,
        Instrument,
        Level,
        Span,
        debug,
        debug_span,
        enabled,
        error,
        error_span,
        event,
        event_enabled,
        field::Empty,
        info,
        info_span,
        instrument,
        span,
        span_enabled,
        trace,
        trace_span,
        warn,
        warn_span,
    };

    pub use super::{
        MessagingOperation,
        SpanExt,
        SpanKind,
        SpanKindExt,
        current_span_for_instrument_at,
    };
}

#[remain::sorted]
#[derive(Clone, Copy, Debug)]
pub enum OtelStatusCode {
    Error,
    Ok,
    // Unset is not currently used, although represents a valid state.
    #[allow(dead_code)]
    Unset,
}

impl OtelStatusCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Ok => "OK",
            Self::Unset => "",
        }
    }
}

/// Represents valied states for OpenTelemetry's `messaging.operation` field.
///
/// `messaging.operation` has the following list of well-known values. If one of them applies, then
/// the respective value MUST be used, otherwise a custom value MAY be used.
///
/// See: <https://opentelemetry.io/docs/specs/semconv/attributes-registry/messaging/>
#[remain::sorted]
#[derive(Clone, Copy, Debug)]
pub enum MessagingOperation {
    /// A message is created. “Create” spans always refer to a single message and are used to
    /// provide a unique creation context for messages in batch publishing scenarios.
    Create,
    /// One or more messages are passed to a consumer. This operation refers to push-based
    /// scenarios, where consumer register callbacks which get called by messaging SDKs.
    Deliver,
    /// One or more messages are provided for publishing to an intermediary. If a single message is
    /// published, the context of the “Publish” span can be used as the creation context and no
    /// “Create” span needs to be created.
    Publish,
    /// One or more messages are requested by a consumer. This operation refers to pull-based
    /// scenarios, where consumers explicitly call methods of messaging SDKs to receive messages.
    Receive,
}

impl MessagingOperation {
    pub const CREATE_STR: &'static str = "create";
    pub const DELIVER_STR: &'static str = "deliver";
    pub const PUBLISH_STR: &'static str = "publish";
    pub const RECEIVE_STR: &'static str = "receive";

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => Self::CREATE_STR,
            Self::Deliver => Self::DELIVER_STR,
            Self::Publish => Self::PUBLISH_STR,
            Self::Receive => Self::RECEIVE_STR,
        }
    }
}

/// An extention trait for [`SpanKind`] providing string representations.
pub trait SpanKindExt {
    /// Returns a static str representation.
    fn as_str(&self) -> &'static str;

    /// Returns an allocated string representation.
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl SpanKindExt for SpanKind {
    fn as_str(&self) -> &'static str {
        match self {
            SpanKind::Client => "client",
            SpanKind::Server => "server",
            SpanKind::Producer => "producer",
            SpanKind::Consumer => "consumer",
            SpanKind::Internal => "internal",
        }
    }
}

pub trait SpanExt {
    fn record_ok(&self);
    fn record_err<E>(&self, err: E) -> E
    where
        E: Debug + Display;

    // fn record_status<F, T, E>(&self, f: F) -> std::result::Result<T, E>
    // where
    //     F: Fn() -> std::result::Result<T, E>,
    //     E: Debug + Display,
    // {
    //     match f() {
    //         Ok(ok) => {
    //             self.record_ok();
    //             Ok(ok)
    //         }
    //         Err(err) => Err(self.record_err(err)),
    //     }
    // }
}

impl SpanExt for tracing::Span {
    fn record_ok(&self) {
        self.record("otel.status_code", OtelStatusCode::Ok.as_str());
    }

    fn record_err<E>(&self, err: E) -> E
    where
        E: Debug + Display,
    {
        self.record("otel.status_code", OtelStatusCode::Error.as_str());
        self.record("otel.status_message", err.to_string().as_str());
        err
    }
}

/// A telemetry client trait which can update tracing verbosity.
///
/// It is designed to be consumed by library authors without the need to depend on the entire
/// binary/server infrastructure of tracing-rs/OpenTelemetry/etc.
#[async_trait]
pub trait TelemetryClient: Clone + Send + Sync + 'static {
    async fn set_verbosity(&mut self, updated: Verbosity) -> Result<(), ClientError>;
    async fn modify_verbosity(&mut self) -> Result<(), ClientError>;
    async fn set_custom_tracing(
        &mut self,
        directives: impl Into<String> + Send + 'async_trait,
    ) -> Result<(), ClientError>;
}

/// A telemetry type that can report its tracing level.
#[async_trait]
pub trait TelemetryLevel: Send + Sync {
    async fn is_debug_or_lower(&self) -> bool;
}

/// A telemetry client which holds handles to a process' tracing and OpenTelemetry setup.
#[derive(Clone, Debug)]
pub struct ApplicationTelemetryClient {
    service_name: Box<str>,
    app_modules: Arc<Vec<&'static str>>,
    interesting_modules: Arc<Vec<&'static str>>,
    never_modules: Arc<Vec<&'static str>>,
    tracing_level: Arc<Mutex<TracingLevel>>,
    update_telemetry_tx: mpsc::UnboundedSender<TelemetryCommand>,
}

impl ApplicationTelemetryClient {
    pub fn new(
        service_name: Box<str>,
        app_modules: Vec<&'static str>,
        interesting_modules: Vec<&'static str>,
        never_modules: Vec<&'static str>,
        tracing_level: TracingLevel,
        update_telemetry_tx: mpsc::UnboundedSender<TelemetryCommand>,
    ) -> Self {
        Self {
            service_name,
            app_modules: Arc::new(app_modules),
            interesting_modules: Arc::new(interesting_modules),
            never_modules: Arc::new(never_modules),
            tracing_level: Arc::new(Mutex::new(tracing_level)),
            update_telemetry_tx,
        }
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub async fn set_verbosity_and_wait(&mut self, updated: Verbosity) -> Result<(), ClientError> {
        let (tx, rx) = oneshot::channel();

        self.set_verbosity_inner(updated, Some(tx)).await?;

        if let Err(err) = rx.await {
            warn!(error = ?err, "sender already closed while waiting on verbosity change");
        }

        Ok(())
    }

    pub async fn modify_verbosity_and_wait(&mut self) -> Result<(), ClientError> {
        let (tx, rx) = oneshot::channel();

        self.modify_verbosity_inner(Some(tx)).await?;

        if let Err(err) = rx.await {
            warn!(
                error = ?err,
                "sender already closed while waiting on verbosity increase change",
            );
        }

        Ok(())
    }

    pub async fn set_custom_tracing_and_wait(
        &mut self,
        directives: impl Into<String> + Send,
    ) -> Result<(), ClientError> {
        let (tx, rx) = oneshot::channel();

        self.set_custom_tracing_inner(directives, Some(tx)).await?;

        if let Err(err) = rx.await {
            warn!(error = ?err, "sender already closed while waiting on custom tracing change");
        }

        Ok(())
    }

    async fn set_verbosity_inner(
        &mut self,
        updated: Verbosity,
        wait: Option<oneshot::Sender<()>>,
    ) -> Result<(), ClientError> {
        let mut guard = self.tracing_level.lock().await;
        let tracing_level = guard.deref_mut();

        match tracing_level {
            TracingLevel::Verbosity { verbosity, .. } => {
                *verbosity = updated;
            }
            TracingLevel::Custom(_) => {
                *tracing_level = TracingLevel::new(
                    updated,
                    Some(self.app_modules.as_slice()),
                    Some(self.interesting_modules.as_slice()),
                    Some(self.never_modules.as_slice()),
                );
            }
        }

        self.update_telemetry_tx
            .send(TelemetryCommand::TracingLevel {
                level: tracing_level.clone(),
                wait,
            })?;

        Ok(())
    }

    async fn modify_verbosity_inner(
        &mut self,
        wait: Option<oneshot::Sender<()>>,
    ) -> Result<(), ClientError> {
        let guard = self.tracing_level.lock().await;
        match guard.deref() {
            TracingLevel::Verbosity { verbosity, .. } => {
                let updated = match verbosity.is_max() {
                    true => Verbosity::InfoAll,
                    false => verbosity.increase(),
                };
                drop(guard);
                self.set_verbosity_inner(updated, wait).await
            }
            TracingLevel::Custom(_) => Err(ClientError::CustomHasNoVerbosity),
        }
    }

    async fn set_custom_tracing_inner(
        &mut self,
        directives: impl Into<String> + Send,
        wait: Option<oneshot::Sender<()>>,
    ) -> Result<(), ClientError> {
        let mut guard = self.tracing_level.lock().await;
        let tracing_level = guard.deref_mut();

        let updated = TracingLevel::custom(directives);
        *tracing_level = updated;
        self.update_telemetry_tx
            .send(TelemetryCommand::TracingLevel {
                level: tracing_level.clone(),
                wait,
            })?;
        Ok(())
    }
}

#[async_trait]
impl TelemetryClient for ApplicationTelemetryClient {
    async fn set_verbosity(&mut self, updated: Verbosity) -> Result<(), ClientError> {
        self.set_verbosity_inner(updated, None).await
    }

    async fn modify_verbosity(&mut self) -> Result<(), ClientError> {
        self.modify_verbosity_inner(None).await
    }

    async fn set_custom_tracing(
        &mut self,
        directives: impl Into<String> + Send + 'async_trait,
    ) -> Result<(), ClientError> {
        self.set_custom_tracing_inner(directives, None).await
    }
}

#[async_trait]
impl TelemetryLevel for ApplicationTelemetryClient {
    async fn is_debug_or_lower(&self) -> bool {
        self.tracing_level.lock().await.is_debug_or_lower()
    }
}

/// A "no-nothing" telemetry client suitable for test code.
///
/// Note that it will respond to `is_debug_or_lower` if the `SI_TEST_VERBOSE` environment variable
/// is set which may increase logging output in tests (typically useful when debugging).
#[derive(Clone, Copy, Debug)]
pub struct NoopClient;

#[async_trait]
impl TelemetryClient for NoopClient {
    async fn set_verbosity(&mut self, _updated: Verbosity) -> Result<(), ClientError> {
        Ok(())
    }

    async fn modify_verbosity(&mut self) -> Result<(), ClientError> {
        Ok(())
    }

    async fn set_custom_tracing(
        &mut self,
        _directives: impl Into<String> + Send + 'async_trait,
    ) -> Result<(), ClientError> {
        Ok(())
    }
}
#[async_trait]
impl TelemetryLevel for NoopClient {
    async fn is_debug_or_lower(&self) -> bool {
        #[allow(clippy::disallowed_methods)] // NoopClient is only used in testing and this
        // environment variable is prefixed with `SI_TEST_` to denote that it's only for testing
        // purposes.
        env::var("SI_TEST_VERBOSE").is_ok()
    }
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("custom tracing level has no verbosity")]
    CustomHasNoVerbosity,
    #[error("error while updating tracing level")]
    UpdateTracingLevel(#[from] mpsc::error::SendError<TelemetryCommand>),
}

#[remain::sorted]
#[derive(Debug)]
pub enum TelemetryCommand {
    Shutdown(CancellationToken),
    TracingLevel {
        level: TracingLevel,
        wait: Option<oneshot::Sender<()>>,
    },
}

#[remain::sorted]
#[derive(Clone, Debug)]
pub enum TracingLevel {
    Custom(String),
    Verbosity {
        verbosity: Verbosity,
        app_modules: Option<Vec<Cow<'static, str>>>,
        interesting_modules: Option<Vec<Cow<'static, str>>>,
        never_modules: Option<Vec<Cow<'static, str>>>,
    },
}

impl TracingLevel {
    pub fn new(
        verbosity: Verbosity,
        app_modules: Option<impl IntoAppModules>,
        interesting_modules: Option<impl IntoAppModules>,
        never_modules: Option<impl IntoAppModules>,
    ) -> Self {
        Self::Verbosity {
            verbosity,
            app_modules: app_modules.map(IntoAppModules::into_app_modules),
            interesting_modules: interesting_modules.map(IntoAppModules::into_app_modules),
            never_modules: never_modules.map(IntoAppModules::into_app_modules),
        }
    }

    pub fn custom(directives: impl Into<String>) -> Self {
        Self::Custom(directives.into())
    }

    pub fn is_debug_or_lower(&self) -> bool {
        match self {
            Self::Verbosity { verbosity, .. } => verbosity.is_debug_or_lower(),
            Self::Custom(string) => string.contains("debug") || string.contains("trace"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
#[allow(clippy::enum_variant_names)]
pub enum Verbosity {
    InfoAll,
    DebugAppInfoInterestingInfoAll,
    DebugAppDebugInterestingInfoAll,
    TraceAppDebugInterestingInfoAll,
    TraceAppTraceInterestingInfoAll,
    TraceAppTraceInterestingDebugAll,
    TraceAll,
}

impl Verbosity {
    #[must_use]
    pub fn increase(self) -> Self {
        self.as_u8().saturating_add(1).into()
    }

    #[must_use]
    pub fn decrease(self) -> Self {
        self.as_u8().saturating_sub(1).into()
    }

    fn is_debug_or_lower(&self) -> bool {
        !matches!(self, Self::InfoAll)
    }

    fn is_max(&self) -> bool {
        matches!(self, Self::TraceAll)
    }

    #[inline]
    fn as_u8(self) -> u8 {
        self.into()
    }
}

impl Default for Verbosity {
    fn default() -> Self {
        Self::InfoAll
    }
}

impl From<u8> for Verbosity {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::InfoAll,
            1 => Self::DebugAppInfoInterestingInfoAll,
            2 => Self::DebugAppDebugInterestingInfoAll,
            3 => Self::TraceAppDebugInterestingInfoAll,
            4 => Self::TraceAppTraceInterestingInfoAll,
            5 => Self::TraceAppTraceInterestingDebugAll,
            _ => Self::TraceAll,
        }
    }
}

impl From<Verbosity> for u8 {
    fn from(value: Verbosity) -> Self {
        match value {
            Verbosity::InfoAll => 0,
            Verbosity::DebugAppInfoInterestingInfoAll => 1,
            Verbosity::DebugAppDebugInterestingInfoAll => 2,
            Verbosity::TraceAppDebugInterestingInfoAll => 3,
            Verbosity::TraceAppTraceInterestingInfoAll => 4,
            Verbosity::TraceAppTraceInterestingDebugAll => 5,
            Verbosity::TraceAll => 6,
        }
    }
}

pub trait IntoAppModules {
    fn into_app_modules(self) -> Vec<Cow<'static, str>>;
}

impl IntoAppModules for Vec<String> {
    fn into_app_modules(self) -> Vec<Cow<'static, str>> {
        self.into_iter().map(Cow::Owned).collect()
    }
}

impl IntoAppModules for Vec<&'static str> {
    fn into_app_modules(self) -> Vec<Cow<'static, str>> {
        self.into_iter().map(Cow::Borrowed).collect()
    }
}

impl IntoAppModules for &[&'static str] {
    fn into_app_modules(self) -> Vec<Cow<'static, str>> {
        self.iter().map(|e| Cow::Borrowed(*e)).collect()
    }
}

#[macro_export]
macro_rules! current_span_for_instrument_at {
    ("error") => {
        if enabled!(Level::ERROR) {
            Span::current()
        } else {
            Span::none()
        }
    };
    ("warn") => {
        if enabled!(Level::WARN) {
            Span::current()
        } else {
            Span::none()
        }
    };
    ("info") => {
        if enabled!(Level::INFO) {
            Span::current()
        } else {
            Span::none()
        }
    };
    ("debug") => {
        if enabled!(Level::DEBUG) {
            Span::current()
        } else {
            Span::none()
        }
    };
    ("trace") => {
        if enabled!(Level::TRACE) {
            Span::current()
        } else {
            Span::none()
        }
    };
}
