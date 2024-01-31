use std::{fmt, time::Duration};

use telemetry::{prelude::*, OtelStatusCode};
use tower_http::{trace::OnResponse, LatencyUnit};

/// An implementation of [`OnResponse`] to update span fields for HTTP responses.
#[derive(Clone, Debug)]
pub struct HttpOnResponse {
    level: Level,
    latency_unit: LatencyUnit,
}

impl Default for HttpOnResponse {
    #[inline]
    fn default() -> Self {
        Self {
            level: Level::DEBUG,
            latency_unit: LatencyUnit::Millis,
        }
    }
}

impl HttpOnResponse {
    /// Creates a new `HttpOnResponse`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`Level`] used for the tracing [`Span`].
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }
}

impl<B> OnResponse<B> for HttpOnResponse {
    fn on_response(self, response: &hyper::Response<B>, latency: Duration, span: &Span) {
        let status = response.status();
        span.record("http.response.status_code", status.as_u16());

        // TODO(fnichol): set response headers if useful?

        // In OpenTelemetry HTTP spans, only HTTP/5xx errors should set an `ERROR` value for
        // `otel.status_code` (except for other errors such as transient/network errors).
        //
        // > Span Status MUST be left unset if HTTP status code was in the 1xx, 2xx or 3xx ranges,
        // > unless there was another error (e.g., network error receiving the response body; or
        // > 3xx codes with max redirects exceeded), in which case status MUST be set to Error.
        //
        // > For HTTP status codes in the 4xx range span status MUST be left unset in case of
        // > SpanKind.SERVER and MUST be set to Error in case of SpanKind.CLIENT.
        //
        // See: <https://opentelemetry.io/docs/specs/semconv/http/http-spans/#status>
        if status.is_server_error() {
            span.record("otel.status_code", OtelStatusCode::Error.as_str());
        }

        macro_rules! inner_event {
            ($level:expr, $($tt:tt)*) => {
                match $level {
                    ::telemetry::tracing::Level::ERROR => {
                        ::telemetry::tracing::error!($($tt)*);
                    }
                    ::telemetry::tracing::Level::WARN => {
                        ::telemetry::tracing::warn!($($tt)*);
                    }
                    ::telemetry::tracing::Level::INFO => {
                        ::telemetry::tracing::info!($($tt)*);
                    }
                    ::telemetry::tracing::Level::DEBUG => {
                        ::telemetry::tracing::debug!($($tt)*);
                    }
                    ::telemetry::tracing::Level::TRACE => {
                        ::telemetry::tracing::trace!($($tt)*);
                    }
                }
            };
        }

        let latency = Latency {
            unit: self.latency_unit,
            duration: latency,
        };

        inner_event!(
            self.level,
            %latency,
            status = status.as_u16(),
            "finished processing request",
        );
    }
}

// From `tower_http::trace`
struct Latency {
    unit: LatencyUnit,
    duration: Duration,
}

impl fmt::Display for Latency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.unit {
            LatencyUnit::Seconds => write!(f, "{} s", self.duration.as_secs_f64()),
            LatencyUnit::Millis => write!(f, "{} ms", self.duration.as_millis()),
            LatencyUnit::Micros => write!(f, "{} μs", self.duration.as_micros()),
            LatencyUnit::Nanos => write!(f, "{} ns", self.duration.as_nanos()),
            _ => write!(f, "{:?} (unknown)", self.duration),
        }
    }
}
