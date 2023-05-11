#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
// TODO(fnichol): document all, then drop `missing_errors_doc`
#![allow(clippy::missing_errors_doc)]

use std::{borrow::Cow, env, fmt, result::Result};

use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::mpsc;

pub use opentelemetry::{self, trace::SpanKind};
pub use tracing;

pub mod prelude {
    pub use super::{FormattedSpanKind, SpanExt, SpanKind};
    pub use tracing::{
        self, debug, debug_span, enabled, error, event, event_enabled, field::Empty, info,
        info_span, instrument, span, span_enabled, trace, trace_span, warn, Instrument, Level,
        Span,
    };
}

pub struct FormattedSpanKind(pub SpanKind);

impl fmt::Display for FormattedSpanKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            SpanKind::Client => write!(f, "client"),
            SpanKind::Server => write!(f, "server"),
            SpanKind::Producer => write!(f, "producer"),
            SpanKind::Consumer => write!(f, "consumer"),
            SpanKind::Internal => write!(f, "internal"),
        }
    }
}

pub trait SpanExt {
    fn record_ok(&self);
    fn record_err<E>(&self, err: E) -> E
    where
        E: std::error::Error;
}

impl SpanExt for tracing::Span {
    fn record_ok(&self) {
        self.record("otel.status_code", "OK");
    }

    fn record_err<E>(&self, err: E) -> E
    where
        E: std::error::Error,
    {
        self.record("otel.status_code", "ERROR");
        self.record("otel.status_message", err.to_string().as_str());
        err
    }
}

#[remain::sorted]
#[derive(Clone, Copy, Debug)]
pub enum UpdateOpenTelemetry {
    Disable,
    Enable,
}

/// A telemetry client trait which can update tracing verbosity, toggle OpenTelemetry services,
/// etc.
///
/// It is designed to be consumed by library authors without the need to depend on the entire
/// binary/server infrastructure of tracing-rs/OpenTelemetry/etc.
#[async_trait]
pub trait TelemetryClient: Clone + Send + Sync + 'static {
    async fn set_verbosity(&mut self, updated: Verbosity) -> Result<(), ClientError>;
    async fn increase_verbosity(&mut self) -> Result<(), ClientError>;
    async fn decrease_verbosity(&mut self) -> Result<(), ClientError>;
    async fn set_custom_tracing(
        &mut self,
        directives: impl Into<String> + Send + 'async_trait,
    ) -> Result<(), ClientError>;
    async fn enable_opentelemetry(&mut self) -> Result<(), ClientError>;
    async fn disable_opentelemetry(&mut self) -> Result<(), ClientError>;
}

/// A telemetry type that can report its tracing level.
pub trait TelemetryLevel: Send + Sync {
    fn is_debug_or_lower(&self) -> bool;
}

/// A telemetry client which holds handles to a process' tracing and OpenTelemetry setup.
#[derive(Clone, Debug)]
pub struct ApplicationTelemetryClient {
    app_modules: Vec<&'static str>,
    tracing_level: TracingLevel,
    tracing_level_tx: mpsc::Sender<TracingLevel>,
    opentelemetry_tx: mpsc::Sender<UpdateOpenTelemetry>,
}

impl ApplicationTelemetryClient {
    pub fn new(
        app_modules: Vec<&'static str>,
        tracing_level: TracingLevel,
        tracing_level_tx: mpsc::Sender<TracingLevel>,
        opentelemetry_tx: mpsc::Sender<UpdateOpenTelemetry>,
    ) -> Self {
        Self {
            app_modules,
            tracing_level,
            tracing_level_tx,
            opentelemetry_tx,
        }
    }
}

#[async_trait]
impl TelemetryClient for ApplicationTelemetryClient {
    async fn set_verbosity(&mut self, updated: Verbosity) -> Result<(), ClientError> {
        match self.tracing_level {
            TracingLevel::Verbosity {
                ref mut verbosity, ..
            } => {
                *verbosity = updated;
            }
            TracingLevel::Custom(_) => {
                self.tracing_level = TracingLevel::new(updated, Some(self.app_modules.as_slice()));
            }
        }

        self.tracing_level_tx
            .send(self.tracing_level.clone())
            .await?;
        Ok(())
    }

    async fn increase_verbosity(&mut self) -> Result<(), ClientError> {
        match self.tracing_level {
            TracingLevel::Verbosity { verbosity, .. } => {
                let updated = verbosity.increase();
                self.set_verbosity(updated).await
            }
            TracingLevel::Custom(_) => Err(ClientError::CustomHasNoVerbosity),
        }
    }

    async fn decrease_verbosity(&mut self) -> Result<(), ClientError> {
        match self.tracing_level {
            TracingLevel::Verbosity { verbosity, .. } => {
                let updated = verbosity.decrease();
                self.set_verbosity(updated).await
            }
            TracingLevel::Custom(_) => Err(ClientError::CustomHasNoVerbosity),
        }
    }

    async fn set_custom_tracing(
        &mut self,
        directives: impl Into<String> + Send + 'async_trait,
    ) -> Result<(), ClientError> {
        let updated = TracingLevel::custom(directives);
        self.tracing_level = updated;
        self.tracing_level_tx
            .send(self.tracing_level.clone())
            .await?;
        Ok(())
    }

    async fn enable_opentelemetry(&mut self) -> Result<(), ClientError> {
        self.opentelemetry_tx
            .send(UpdateOpenTelemetry::Enable)
            .await
            .map_err(Into::into)
    }

    async fn disable_opentelemetry(&mut self) -> Result<(), ClientError> {
        self.opentelemetry_tx
            .send(UpdateOpenTelemetry::Disable)
            .await
            .map_err(Into::into)
    }
}

impl TelemetryLevel for ApplicationTelemetryClient {
    fn is_debug_or_lower(&self) -> bool {
        self.tracing_level.is_debug_or_lower()
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

    async fn increase_verbosity(&mut self) -> Result<(), ClientError> {
        Ok(())
    }

    async fn decrease_verbosity(&mut self) -> Result<(), ClientError> {
        Ok(())
    }

    async fn set_custom_tracing(
        &mut self,
        _directives: impl Into<String> + Send + 'async_trait,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    async fn enable_opentelemetry(&mut self) -> Result<(), ClientError> {
        Ok(())
    }

    async fn disable_opentelemetry(&mut self) -> Result<(), ClientError> {
        Ok(())
    }
}

impl TelemetryLevel for NoopClient {
    fn is_debug_or_lower(&self) -> bool {
        env::var("SI_TEST_VERBOSE").is_ok()
    }
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("custom tracing level has no verbosity")]
    CustomHasNoVerbosity,
    #[error("error while updating opentelemetry")]
    UpdateOpenTelemetry(#[from] mpsc::error::SendError<UpdateOpenTelemetry>),
    #[error("error while updating tracing level")]
    UpdateTracingLevel(#[from] mpsc::error::SendError<TracingLevel>),
}

#[remain::sorted]
#[derive(Clone, Debug)]
pub enum TracingLevel {
    Custom(String),
    Verbosity {
        verbosity: Verbosity,
        app_modules: Option<Vec<Cow<'static, str>>>,
    },
}

impl TracingLevel {
    pub fn new(verbosity: Verbosity, app_modules: Option<impl IntoAppModules>) -> Self {
        Self::Verbosity {
            verbosity,
            app_modules: app_modules.map(IntoAppModules::into_app_modules),
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

#[remain::sorted]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
#[allow(clippy::enum_variant_names)]
pub enum Verbosity {
    DebugAppAndInfoAll,
    InfoAll,
    TraceAll,
    TraceAppAndDebugAll,
    TraceAppAndInfoAll,
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
            1 => Self::DebugAppAndInfoAll,
            2 => Self::TraceAppAndInfoAll,
            3 => Self::TraceAppAndDebugAll,
            _ => Self::TraceAll,
        }
    }
}

impl From<Verbosity> for u8 {
    fn from(value: Verbosity) -> Self {
        match value {
            Verbosity::InfoAll => 0,
            Verbosity::DebugAppAndInfoAll => 1,
            Verbosity::TraceAppAndInfoAll => 2,
            Verbosity::TraceAppAndDebugAll => 3,
            Verbosity::TraceAll => 4,
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
