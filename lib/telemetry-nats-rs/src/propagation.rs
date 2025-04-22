//! Telemetry propagation via NATS headers.

use si_data_nats::HeaderMap;
use telemetry::{
    opentelemetry::{Context, global, trace::TraceContextExt},
    tracing::Span,
};

/// Injects propagation telemetry into a [`HeaderMap`].
pub fn inject_headers(headers: &mut HeaderMap) {
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let ctx = Span::current().context();
    inject_opentelemetry_context(&ctx, headers)
}

/// Creates a [`HeaderMap`] containing propagation telemetry.
pub fn empty_injected_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    inject_headers(&mut headers);

    headers
}

/// Associates the current [`Span`] with propagation telemetry in an optional [`HeaderMap`].
pub fn associate_current_span_from_headers<'a>(headers: impl Into<Option<&'a HeaderMap>>) {
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    if let Some(headers) = headers.into() {
        let span_ctx = extract_opentelemetry_context(headers)
            .span()
            .span_context()
            .clone();
        Span::current().add_link(span_ctx);
    }
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

mod headers {
    use std::str::FromStr;

    use si_data_nats::{HeaderMap, HeaderName, HeaderValue};
    use telemetry::opentelemetry::propagation::{Extractor, Injector};

    pub struct HeaderInjector<'a>(pub &'a mut HeaderMap);

    impl Injector for HeaderInjector<'_> {
        /// Set a key and value in the HeaderMap.  Does nothing if the key or value are not valid inputs.
        fn set(&mut self, key: &str, value: String) {
            if let Ok(name) = HeaderName::from_str(key) {
                if let Ok(val) = HeaderValue::from_str(&value) {
                    self.0.insert(name, val);
                }
            }
        }
    }

    pub struct HeaderExtractor<'a>(pub &'a HeaderMap);

    impl Extractor for HeaderExtractor<'_> {
        /// Get a value for a key from the HeaderMap.  If the value is not valid ASCII, returns None.
        fn get(&self, key: &str) -> Option<&str> {
            self.0.get(key).map(<HeaderValue as AsRef<str>>::as_ref)
        }

        /// Collect all the keys from the HeaderMap.
        fn keys(&self) -> Vec<&str> {
            self.0
                .iter()
                .map(|(name, _values)| <HeaderName as AsRef<str>>::as_ref(name))
                .collect::<Vec<_>>()
        }
    }
}
