use std::{fmt, ops::Deref};

use tracing::Span;

pub use opentelemetry::trace::SpanKind;

pub trait SpanExt {
    fn record_ok(&self);
    fn record_err<E>(&self, err: E) -> E
    where
        E: std::error::Error;
}

impl SpanExt for Span {
    fn record_ok(&self) {
        self.record("otel.status_code", &StatusCode::Ok.as_str());
    }

    fn record_err<E>(&self, err: E) -> E
    where
        E: std::error::Error,
    {
        self.record("otel.status_code", &StatusCode::Error.as_str());
        self.record("otel.status_message", &err.to_string().as_str());
        err
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatusCode {
    /// The default status.
    #[allow(dead_code)]
    Unset,
    /// OK is returned on success.
    #[allow(dead_code)]
    Ok,
    /// The operation contains an error.
    Error,
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unset => f.write_str(opentelemetry::trace::StatusCode::Unset.as_str()),
            Self::Ok => f.write_str(opentelemetry::trace::StatusCode::Ok.as_str()),
            Self::Error => f.write_str(opentelemetry::trace::StatusCode::Error.as_str()),
        }
    }
}

impl Deref for StatusCode {
    type Target = opentelemetry::trace::StatusCode;

    fn deref(&self) -> &Self::Target {
        match self {
            StatusCode::Unset => &opentelemetry::trace::StatusCode::Unset,
            StatusCode::Ok => &opentelemetry::trace::StatusCode::Ok,
            StatusCode::Error => &opentelemetry::trace::StatusCode::Error,
        }
    }
}
