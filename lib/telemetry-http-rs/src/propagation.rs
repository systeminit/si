//! Telemetry propagation via HTTP headers.

use http::HeaderMap;
use telemetry::{
    opentelemetry::{global, Context},
    tracing::Span,
};

/// Injects propagation telemetry into a [`HeaderMap`].
pub fn inject_headers(headers: &mut HeaderMap) {
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let ctx = Span::current().context();
    inject_opentelemetry_context(&ctx, headers)
}

/// Extracts an OpenTelemetry [`Context`] from a [`HeaderMap`].
pub fn extract_opentelemetry_context(headers: &HeaderMap) -> Context {
    let extractor = self::headers::HeaderExtractor(headers);
    global::get_text_map_propagator(|propagator| propagator.extract(&extractor))
}

/// Injects an OpenTelemetry [`Context`] into a [`HeaderMap`].
pub fn inject_opentelemetry_context(ctx: &Context, headers: &mut HeaderMap) {
    let mut injector = self::headers::HeaderInjector(headers);
    global::get_text_map_propagator(|propagator| propagator.inject_context(ctx, &mut injector));
}

// Implementation vendored from `opentelemetry-http` crate, released under the Apache 2.0 license
//
// https://github.com/open-telemetry/opentelemetry-rust/blob/47881b20a2b8e94d8e1cdbd4877852dd74cc07de/opentelemetry-http/src/lib.rs#L13-L41
mod headers {
    use telemetry::opentelemetry::propagation::{Extractor, Injector};

    pub struct HeaderInjector<'a>(pub &'a mut http::HeaderMap);

    impl<'a> Injector for HeaderInjector<'a> {
        /// Set a key and value in the HeaderMap.  Does nothing if the key or value are not valid inputs.
        fn set(&mut self, key: &str, value: String) {
            if let Ok(name) = http::header::HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(val) = http::header::HeaderValue::from_str(&value) {
                    self.0.insert(name, val);
                }
            }
        }
    }

    pub struct HeaderExtractor<'a>(pub &'a http::HeaderMap);

    impl<'a> Extractor for HeaderExtractor<'a> {
        /// Get a value for a key from the HeaderMap.  If the value is not valid ASCII, returns None.
        fn get(&self, key: &str) -> Option<&str> {
            self.0.get(key).and_then(|value| value.to_str().ok())
        }

        /// Collect all the keys from the HeaderMap.
        fn keys(&self) -> Vec<&str> {
            self.0
                .keys()
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        }
    }
}
