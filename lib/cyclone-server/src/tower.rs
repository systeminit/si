pub use web_socket_trace::WebSocketTraceLayer;

mod limit_requests {
    use std::{
        future::Future,
        pin::Pin,
        sync::{
            atomic::{AtomicU32, Ordering},
            Arc,
        },
        task::{Context, Poll},
    };

    use pin_project_lite::pin_project;
    use telemetry::prelude::*;
    use tokio::sync::mpsc;
    use tower::{Layer, Service};

    use crate::ShutdownSource;

    #[derive(Clone, Debug)]
    pub struct LimitRequestLayer {
        remaining: Arc<Option<AtomicU32>>,
        shutdown_tx: mpsc::Sender<ShutdownSource>,
    }

    impl LimitRequestLayer {
        #[allow(dead_code)]
        pub fn new(
            remaining: Arc<Option<AtomicU32>>,
            shutdown_tx: mpsc::Sender<ShutdownSource>,
        ) -> Self {
            Self {
                remaining,
                shutdown_tx,
            }
        }
    }

    impl<S> Layer<S> for LimitRequestLayer {
        type Service = LimitRequest<S>;

        fn layer(&self, inner: S) -> Self::Service {
            LimitRequest::new(inner, self.remaining.clone(), self.shutdown_tx.clone())
        }
    }

    #[derive(Clone, Debug)]
    pub struct LimitRequest<T> {
        inner: T,
        remaining: Arc<Option<AtomicU32>>,
        shutdown_tx: mpsc::Sender<ShutdownSource>,
    }

    impl<T> LimitRequest<T> {
        pub fn new(
            inner: T,
            remaining: Arc<Option<AtomicU32>>,
            shutdown_tx: mpsc::Sender<ShutdownSource>,
        ) -> Self {
            Self {
                inner,
                remaining,
                shutdown_tx,
            }
        }
    }

    impl<S, Request> Service<Request> for LimitRequest<S>
    where
        S: Service<Request>,
        S::Error: Sync,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = ResponseFuture<S::Future>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, request: Request) -> Self::Future {
            let shutdown_tx = match (*self.remaining).as_ref() {
                // If we're limiting requests, then decrement by one and if we're at zero, pass
                // along a shutdown tx handle to the future. Otherwise, the future doesn't get a
                // shutdown handle.
                Some(remaining) => {
                    let mut updated = remaining.load(Ordering::Relaxed);
                    updated = updated.saturating_sub(1);
                    remaining.store(updated, Ordering::Relaxed);
                    debug!("requests remaining: {}", updated);

                    if updated > 0 {
                        None
                    } else {
                        Some(self.shutdown_tx.clone())
                    }
                }
                // If we're not limiting requests, then pass `None`
                None => None,
            };
            let response = self.inner.call(request);

            ResponseFuture::new(response, shutdown_tx)
        }
    }

    pin_project! {
        #[derive(Debug)]
        pub struct ResponseFuture<T> {
            #[pin]
            response: T,
            shutdown_tx: Option<mpsc::Sender<ShutdownSource>>,
        }
    }

    impl<T> ResponseFuture<T> {
        fn new(response: T, shutdown_tx: Option<mpsc::Sender<ShutdownSource>>) -> Self {
            Self {
                response,
                shutdown_tx,
            }
        }
    }

    impl<F, T, E> Future for ResponseFuture<F>
    where
        F: Future<Output = Result<T, E>>,
    {
        type Output = Result<T, E>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.project();

            match this.response.poll(cx) {
                Poll::Ready(value) => {
                    if let Some(tx) = this.shutdown_tx {
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            trace!("sending shutdown to limit request shutdown receiver");
                            if tx.send(ShutdownSource::LimitRequest).await.is_err() {
                                warn!(
                                    "the limit request shutdown receiver has already been dropped"
                                );
                            }
                        });
                    }

                    Poll::Ready(value)
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

mod web_socket_trace {
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    };

    use axum::{body::Body, extract::MatchedPath, http::Request, response::Response};
    use pin_project_lite::pin_project;
    use telemetry::prelude::*;
    use telemetry_http::{propagation, ParentSpan};
    use tower::{Layer, Service};

    // Vendored and modified from futures_core::task::poll::ready
    macro_rules! ready {
        ($e:expr $(,)?) => {
            match $e {
                ::std::task::Poll::Ready(t) => t,
                ::std::task::Poll::Pending => return ::std::task::Poll::Pending,
            }
        };
    }

    #[derive(Clone, Debug)]
    pub struct WebSocketTraceLayer;

    impl WebSocketTraceLayer {
        pub fn new() -> Self {
            Self
        }
    }

    impl<S> Layer<S> for WebSocketTraceLayer {
        type Service = WebSocketTrace<S>;

        fn layer(&self, inner: S) -> Self::Service {
            WebSocketTrace::new(inner)
        }
    }

    #[derive(Clone, Debug)]
    pub struct WebSocketTrace<S> {
        inner: S,
    }

    impl<S> WebSocketTrace<S> {
        pub fn new(inner: S) -> Self {
            Self { inner }
        }
    }

    impl<S> Service<Request<Body>> for WebSocketTrace<S>
    where
        S: Service<Request<Body>, Response = Response> + Send + 'static,
        S::Future: Send + 'static,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = ResponseFuture<S::Future>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, mut req: Request<Body>) -> Self::Future {
            let span = span_from_request(&req);

            req.extensions_mut().insert(ParentSpan::new(span.clone()));

            let response = {
                let _guard = span.enter();
                self.inner.call(req)
            };

            ResponseFuture { inner: response }
        }
    }

    pin_project! {
        #[derive(Debug)]
        pub struct ResponseFuture<S> {
            #[pin]
            inner: S,
        }
    }

    impl<F, T, E> Future for ResponseFuture<F>
    where
        F: Future<Output = Result<T, E>>,
    {
        type Output = Result<T, E>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.project();
            let result = ready!(this.inner.poll(cx));

            Poll::Ready(result)
        }
    }

    fn span_from_request(req: &Request<Body>) -> Span {
        let span = span!(
            Level::INFO,
            "GET websocket",
            otel.kind = SpanKind::Server.as_str(),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        );

        let method = req.method().as_str();
        let matched_path = req.extensions().get::<MatchedPath>().map(|mp| mp.as_str());

        span.record(
            "otel.name",
            match matched_path {
                Some(path) => format!("WS {method} {path}"),
                None => format!("WS {method}"),
            },
        );

        // Extract OpenTelemetry parent span metadata from the request headers (if it exists) and
        // associate it with this request span
        propagation::parent_span_from_headers(&span, req.headers());

        span
    }
}
