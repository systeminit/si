use std::{
    convert::Infallible,
    error,
    fmt,
    future::{
        Future,
        IntoFuture,
        poll_fn,
    },
    io,
    marker::PhantomData,
    ops,
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use futures::{
    Stream,
    TryStreamExt,
};
use telemetry_utils::metric;
use tokio::{
    sync::{
        Notify,
        OwnedSemaphorePermit,
        Semaphore,
    },
    time,
};
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};
use tower::{
    Service,
    ServiceExt,
};
use tracing::{
    debug,
    error,
    info,
    trace,
};

use crate::{
    message::{
        Message,
        MessageHead,
    },
    response::Response,
};

#[derive(Debug, Clone)]
enum SemaphoreMode {
    /// Permits are released when the request completes (standard behavior).
    Internal(Arc<Semaphore>),
    /// Permits are forgotten after acquisition; owner restores via `add_permits()`.
    External(Arc<Semaphore>),
}

pub fn serve<M, S, T, E, R>(stream: T, make_service: M) -> Serve<M, S, T, E, R>
where
    M: for<'a> Service<IncomingMessage<'a, R>, Error = Infallible, Response = S>,
    S: Service<Message<R>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send,
    T: Stream<Item = Result<R, E>>,
    E: error::Error,
    R: MessageHead,
{
    Serve {
        stream,
        make_service,
        semaphore: None,
        _service_marker: PhantomData,
        _stream_error_marker: PhantomData,
        _request_marker: PhantomData,
    }
}

pub fn serve_with_incoming_limit<M, S, T, E, R>(
    stream: T,
    make_service: M,
    limit: impl Into<Option<usize>>,
) -> Serve<M, S, T, E, R>
where
    M: for<'a> Service<IncomingMessage<'a, R>, Error = Infallible, Response = S>,
    S: Service<Message<R>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send,
    T: Stream<Item = Result<R, E>>,
    E: error::Error,
    R: MessageHead,
{
    Serve {
        stream,
        make_service,
        semaphore: limit
            .into()
            .map(|l| SemaphoreMode::Internal(Arc::new(Semaphore::new(l)))),
        _service_marker: PhantomData,
        _stream_error_marker: PhantomData,
        _request_marker: PhantomData,
    }
}

/// Like [`serve_with_incoming_limit`], but permits are forgotten after acquisition
/// and must be restored by the semaphore owner via `add_permits()`.
pub fn serve_with_external_semaphore<M, S, T, E, R>(
    stream: T,
    make_service: M,
    semaphore: Arc<Semaphore>,
) -> Serve<M, S, T, E, R>
where
    M: for<'a> Service<IncomingMessage<'a, R>, Error = Infallible, Response = S>,
    S: Service<Message<R>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send,
    T: Stream<Item = Result<R, E>>,
    E: error::Error,
    R: MessageHead,
{
    Serve {
        stream,
        make_service,
        semaphore: Some(SemaphoreMode::External(semaphore)),
        _service_marker: PhantomData,
        _stream_error_marker: PhantomData,
        _request_marker: PhantomData,
    }
}

#[must_use = "futures must be awaited or polled"]
pub struct Serve<M, S, T, E, R> {
    stream: T,
    make_service: M,
    semaphore: Option<SemaphoreMode>,
    _service_marker: PhantomData<S>,
    _stream_error_marker: PhantomData<E>,
    _request_marker: PhantomData<R>,
}

impl<M, S, T, E, R> fmt::Debug for Serve<M, S, T, E, R>
where
    M: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Serve")
            .field("make_service", &self.make_service)
            .finish_non_exhaustive()
    }
}

impl<M, S, T, E, R> Serve<M, S, T, E, R> {
    pub fn with_graceful_shutdown<F>(self, signal: F) -> WithGracefulShutdown<M, S, T, E, R, F>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        WithGracefulShutdown {
            stream: self.stream,
            make_service: self.make_service,
            semaphore: self.semaphore,
            signal,
            _service_marker: PhantomData,
            _stream_error_marker: PhantomData,
            _request_marker: PhantomData,
        }
    }
}

/// Serve future with supporting a  graceful shutdown.
#[must_use = "futures must be awaited or polled"]
pub struct WithGracefulShutdown<M, S, T, E, R, F> {
    stream: T,
    make_service: M,
    semaphore: Option<SemaphoreMode>,
    signal: F,
    _service_marker: PhantomData<S>,
    _stream_error_marker: PhantomData<E>,
    _request_marker: PhantomData<R>,
}

impl<M, S, T, E, R, F> fmt::Debug for WithGracefulShutdown<M, S, T, E, R, F>
where
    M: fmt::Debug,
    S: fmt::Debug,
    F: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WithGracefulShutdown")
            .field("make_service", &self.make_service)
            .field("signal", &self.signal)
            .finish_non_exhaustive()
    }
}

impl<M, S, T, E, R, F> IntoFuture for WithGracefulShutdown<M, S, T, E, R, F>
where
    M: for<'a> Service<IncomingMessage<'a, R>, Error = Infallible, Response = S> + Send + 'static,
    for<'a> <M as Service<IncomingMessage<'a, R>>>::Future: Send,
    S: Service<Message<R>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send,
    T: Stream<Item = Result<R, E>> + Send + 'static,
    E: error::Error,
    R: MessageHead + Send + 'static,
    F: Future<Output = ()> + Send + 'static,
{
    type Output = io::Result<()>;
    type IntoFuture = private::ServeFuture;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            stream,
            mut make_service,
            semaphore,
            signal,
            ..
        } = self;

        let tracker = TaskTracker::new();
        let graceful_token = CancellationToken::new();

        let notify_graceful_shutdown = Arc::new(Notify::new());
        let terminate_graceful_shutdown = notify_graceful_shutdown.clone();

        let token = graceful_token.clone();
        tracker.spawn(async move {
            tokio::select! {
                _ = signal => {
                    debug!("received graceful shutdown signal, telling tasks to shutdown");
                    token.cancel();
                }
                _ = terminate_graceful_shutdown.notified() => {
                    trace!("graceful shutdown task terminating");
                }
            }
        });

        private::ServeFuture(Box::pin(async move {
            tokio::pin!(stream);

            metric!(counter.naxum.next_message.processing = 0);
            metric!(counter.naxum.next_message.failed = 0);

            loop {
                let (msg, permit) = tokio::select! {
                    biased;

                    _ = graceful_token.cancelled() => {
                        debug!("signal received, not accepting new messages");
                        tracker.close();
                        break;
                    }
                    (msg, permit) = next_message(&mut stream, &semaphore) => {
                        match msg {
                            Some(Ok(msg)) => {
                                metric!(counter.naxum.next_message.failed = 0);
                                (msg, permit)
                            },
                            Some(Err(err)) => {
                                error!(si.error.message = ?err, "failed to read next message from stream");
                                metric!(counter.naxum.next_message.failed = 1);
                                continue;
                            },
                            None => {
                                info!("stream is closed, breaking out of loop");
                                tracker.close();
                                break;
                            },
                        }
                    }
                };

                trace!(subject = msg.subject().as_str(), "message received");
                metric!(counter.naxum.next_message.processing = 1);

                poll_fn(|cx| make_service.poll_ready(cx))
                    .await
                    .unwrap_or_else(|err| match err {});

                let tower_svc = make_service
                    .call(IncomingMessage { msg: &msg })
                    .await
                    .unwrap_or_else(|err| match err {});

                tracker.spawn(async move {
                    let _result = tower_svc.oneshot(msg).await;
                    metric!(counter.naxum.next_message.processing = -1);
                    trace!("message processed");

                    drop(permit);
                });
            }

            notify_graceful_shutdown.notify_one();

            trace!("waiting for {} task(s) to finish", tracker.len());
            let mut progress_interval = time::interval_at(
                time::Instant::now() + Duration::from_secs(5),
                Duration::from_secs(5),
            );
            loop {
                tokio::select! {
                    _ = tracker.wait() => {
                        break;
                    }
                    _ = progress_interval.tick() => {
                        debug!("waiting for {} task(s) to finish", tracker.len());
                    }
                }
            }

            Ok(())
        }))
    }
}

async fn next_message<T, E, R>(
    stream: &mut Pin<&mut T>,
    semaphore: &Option<SemaphoreMode>,
) -> (Option<Result<Message<R>, E>>, Option<OwnedSemaphorePermit>)
where
    T: Stream<Item = Result<R, E>> + Send + 'static,
    E: error::Error,
    R: MessageHead + Send + 'static,
{
    // Acquire permit before awaiting next message on stream, thereby limiting the number of
    // spawned processing requests
    let permit = match semaphore {
        Some(SemaphoreMode::Internal(semaphore)) => {
            #[allow(clippy::expect_used)]
            // errors only if semaphore is closed (we never close)
            Some(
                semaphore
                    .clone()
                    .acquire_owned()
                    .await
                    .expect("semaphore will not be closed"),
            )
        }
        Some(SemaphoreMode::External(semaphore)) => {
            #[allow(clippy::expect_used)]
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .expect("semaphore will not be closed");
            permit.forget();
            None
        }
        None => None,
    };

    match stream.try_next().await {
        Ok(maybe) => (
            Ok(maybe.map(|inner| Message::new(inner))).transpose(),
            permit,
        ),
        Err(err) => (Some(Err(err)), permit),
    }
}

mod private {
    use std::{
        fmt,
        future::Future,
        io,
        pin::Pin,
        task::{
            Context,
            Poll,
        },
    };

    pub struct ServeFuture(pub(super) futures::future::BoxFuture<'static, io::Result<()>>);

    impl Future for ServeFuture {
        type Output = io::Result<()>;

        #[inline]
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.0.as_mut().poll(cx)
        }
    }

    impl fmt::Debug for ServeFuture {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("ServeFuture").finish_non_exhaustive()
        }
    }
}

pub struct IncomingMessage<'a, R> {
    msg: &'a R,
}

impl<R> ops::Deref for IncomingMessage<'_, R>
where
    R: MessageHead,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.msg
    }
}
