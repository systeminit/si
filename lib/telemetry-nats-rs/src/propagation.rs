use si_data_nats::HeaderMap;
use telemetry::{
    opentelemetry::{global, Context},
    tracing::Span,
};

#[inline]
#[must_use]
pub fn find_current_context() -> Context {
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    dbg!(dbg!(Span::current()).context())
}

#[inline]
#[must_use]
pub fn find_trace_id(context: &Context) -> Option<String> {
    use telemetry::opentelemetry::trace::TraceContextExt;

    let span = context.span();
    let span_context = span.span_context();
    span_context
        .is_valid()
        .then(|| span_context.trace_id().to_string())
}

/// Injects propagation telemetry into a [`HeaderMap`].
#[inline]
pub fn inject_headers(mut headers: HeaderMap) -> HeaderMap {
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    let ctx = dbg!(dbg!(Span::current()).context());
    inject_opentelemetry_context(&ctx, &mut headers);

    headers
}

/// Creates a [`HeaderMap`] containing propagation telemetry.
pub fn inject_empty_headers() -> HeaderMap {
    inject_headers(HeaderMap::new())
}

/// Extracts an OpenTelemetry [`Context`] from a [`HeaderMap`].
pub fn extract_opentelemetry_context(headers: &HeaderMap) -> Context {
    let extractor = self::headers::HeaderExtractor(headers);
    global::get_text_map_propagator(|propagator| dbg!(propagator).extract(&extractor))
}

/// Injects an OpenTelemetry [`Context`] into a [`HeaderMap`].
pub fn inject_opentelemetry_context(ctx: &Context, headers: &mut HeaderMap) {
    dbg!(ctx);
    let mut injector = self::headers::HeaderInjector(headers);
    global::get_text_map_propagator(|propagator| {
        dbg!(propagator).inject_context(ctx, &mut injector)
    });
    dbg!(headers);
}

mod headers {
    use std::str::FromStr;

    use si_data_nats::{HeaderMap, HeaderName, HeaderValue};
    use telemetry::opentelemetry::propagation::{Extractor, Injector};

    pub struct HeaderInjector<'a>(pub &'a mut HeaderMap);

    impl<'a> Injector for HeaderInjector<'a> {
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

    impl<'a> Extractor for HeaderExtractor<'a> {
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
