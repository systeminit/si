pub use limit_requests::{LimitRequest, LimitRequestLayer};

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
    use tokio::sync::mpsc;
    use tower::{Layer, Service};
    use tracing::{debug, trace, warn};

    #[derive(Clone, Debug)]
    pub struct LimitRequestLayer {
        remaining: Arc<Option<AtomicU32>>,
        shutdown_tx: mpsc::Sender<()>,
    }

    impl LimitRequestLayer {
        pub fn new(remaining: Option<impl Into<u32>>, shutdown_tx: mpsc::Sender<()>) -> Self {
            Self {
                remaining: Arc::new(remaining.map(|r| r.into().into())),
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
        shutdown_tx: mpsc::Sender<()>,
    }

    impl<T> LimitRequest<T> {
        pub fn new(
            inner: T,
            remaining: Arc<Option<AtomicU32>>,
            shutdown_tx: mpsc::Sender<()>,
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
                    updated = updated.checked_sub(1).unwrap_or(0);
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
            shutdown_tx: Option<mpsc::Sender<()>>,
        }
    }

    impl<T> ResponseFuture<T> {
        fn new(response: T, shutdown_tx: Option<mpsc::Sender<()>>) -> Self {
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
                            if let Err(_) = tx.send(()).await {
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
