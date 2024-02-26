use std::{
    convert::Infallible,
    error, fmt,
    future::{poll_fn, Future, IntoFuture},
    io,
    marker::PhantomData,
    ops,
};

use futures::{Stream, TryStreamExt};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tower::{Service, ServiceExt};
use tracing::{trace, warn};

use crate::{message::MessageHead, response::Response};

pub fn serve<M, S, T, E, R>(stream: T, make_service: M) -> Serve<M, S, T, E, R>
where
    M: for<'a> Service<IncomingMessage<'a, R>, Error = Infallible, Response = S>,
    S: Service<R, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send,
    T: Stream<Item = Result<R, E>>,
    E: error::Error,
    R: MessageHead,
{
    Serve {
        stream,
        make_service,
        _service_marker: PhantomData,
        _stream_error_marker: PhantomData,
        _request_marker: PhantomData,
    }
}

#[must_use = "futures must be awaited or polled"]
pub struct Serve<M, S, T, E, R> {
    stream: T,
    make_service: M,
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
    S: Service<R, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send,
    T: Stream<Item = Result<R, E>> + Unpin + Send + 'static,
    E: error::Error,
    R: MessageHead + Send + 'static,
    F: Future<Output = ()> + Send + 'static,
{
    type Output = io::Result<()>;
    type IntoFuture = private::ServeFuture;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            mut stream,
            mut make_service,
            signal,
            ..
        } = self;

        let tracker = TaskTracker::new();
        let graceful_token = CancellationToken::new();

        let token = graceful_token.clone();
        tracker.spawn(async move {
            signal.await;
            trace!("received graceful shutdown signal, telling tasks to shutdown");
            token.cancel();
        });

        private::ServeFuture(Box::pin(async move {
            loop {
                let msg = tokio::select! {
                    msg = next_message(&mut stream) => {
                        match msg {
                            Some(msg) => msg,
                            None => continue,
                        }
                    }
                    _ = graceful_token.cancelled() => {
                        trace!("signal received, not accepting new messages");
                        tracker.close();
                        break;
                    }
                };

                trace!(subject = msg.subject().as_str(), "message received");

                poll_fn(|cx| make_service.poll_ready(cx))
                    .await
                    .unwrap_or_else(|err| match err {});

                let tower_svc = make_service
                    .call(IncomingMessage { msg: &msg })
                    .await
                    .unwrap_or_else(|err| match err {});

                let graceful_token = graceful_token.clone();
                tracker.spawn(async move {
                    tokio::select! {
                        _result = tower_svc.oneshot(msg) => {
                            // huh
                        }
                        _ = graceful_token.cancelled() => {
                            trace!("signal received in task, starting graceful shutdown");
                        }
                    }
                });
            }

            tracker.wait().await;

            Ok(())
        }))
    }
}

async fn next_message<T, E, R>(stream: &mut T) -> Option<R>
where
    T: Stream<Item = Result<R, E>> + Unpin + Send + 'static,
    E: error::Error,
    R: MessageHead + Send + 'static,
{
    match stream.try_next().await {
        Ok(maybe) => maybe,
        Err(err) => {
            // TODO(fnichol): this level might need to be `trace!()`, just unclear at the moment
            warn!(error = ?err, "failed to resolve next message in stream");
            None
        }
    }
}

mod private {
    use std::{
        fmt,
        future::Future,
        io,
        pin::Pin,
        task::{Context, Poll},
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
