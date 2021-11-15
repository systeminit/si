use std::{borrow::Cow, result::Result};

use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::mpsc;

pub use opentelemetry::trace::SpanKind;

pub mod prelude {
    pub use super::{SpanExt, SpanKind};
    pub use tracing::{
        self, debug, debug_span, error, event, field::Empty, info, info_span, instrument, span,
        trace, trace_span, warn, Instrument, Level, Span,
    };
}

pub trait SpanExt {
    fn record_ok(&self);
    fn record_err<E>(&self, err: E) -> E
    where
        E: std::error::Error;
}

impl SpanExt for tracing::Span {
    fn record_ok(&self) {
        self.record(
            "otel.status_code",
            &opentelemetry::trace::StatusCode::Ok.as_str(),
        );
    }

    fn record_err<E>(&self, err: E) -> E
    where
        E: std::error::Error,
    {
        self.record(
            "otel.status_code",
            &opentelemetry::trace::StatusCode::Error.as_str(),
        );
        self.record("otel.status_message", &err.to_string().as_str());
        err
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UpdateOpenTelemetry {
    Enable,
    Disable,
}

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

#[derive(Clone, Debug)]
pub struct Client {
    app_modules: Vec<&'static str>,
    tracing_level: TracingLevel,
    tracing_level_tx: mpsc::Sender<TracingLevel>,
    opentelemetry_tx: mpsc::Sender<UpdateOpenTelemetry>,
}

impl Client {
    #[cfg(feature = "application")]
    pub(crate) fn new(
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
impl TelemetryClient for Client {
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

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("custom tracing level has no verbosity")]
    CustomHasNoVerbosity,
    #[error("error while updating opentelemetry")]
    UpdateOpenTelemetry(#[from] mpsc::error::SendError<UpdateOpenTelemetry>),
    #[error("error while updating tracing level")]
    UpdateTracingLevel(#[from] mpsc::error::SendError<TracingLevel>),
}

#[derive(Clone, Debug)]
pub enum TracingLevel {
    Verbosity {
        verbosity: Verbosity,
        app_modules: Option<Vec<Cow<'static, str>>>,
    },
    Custom(String),
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
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
#[allow(clippy::enum_variant_names)]
pub enum Verbosity {
    InfoAll,
    DebugAppAndInfoAll,
    TraceAppAndInfoAll,
    TraceAppAndDebugAll,
    TraceAll,
}

impl Verbosity {
    #[must_use]
    pub fn increase(self) -> Self {
        self.as_usize().saturating_add(1).into()
    }

    #[must_use]
    pub fn decrease(self) -> Self {
        self.as_usize().saturating_sub(1).into()
    }

    #[inline]
    fn as_usize(self) -> usize {
        self.into()
    }
}

impl Default for Verbosity {
    fn default() -> Self {
        Self::InfoAll
    }
}

impl From<usize> for Verbosity {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::InfoAll,
            1 => Self::DebugAppAndInfoAll,
            2 => Self::TraceAppAndInfoAll,
            3 => Self::TraceAppAndDebugAll,
            _ => Self::TraceAll,
        }
    }
}

impl From<Verbosity> for usize {
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
