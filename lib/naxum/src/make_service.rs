use std::{
    convert::Infallible,
    fmt,
    future::{ready, Future, Ready},
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;
use tower::Service;

#[derive(Clone, Debug)]
pub struct IntoMakeService<S> {
    svc: S,
}

impl<S> IntoMakeService<S> {
    pub(crate) fn new(svc: S) -> Self {
        Self { svc }
    }
}

impl<S, T> Service<T> for IntoMakeService<S>
where
    S: Clone,
{
    type Response = S;
    type Error = Infallible;
    type Future = IntoMakeServiceFuture<S>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _target: T) -> Self::Future {
        IntoMakeServiceFuture::new(ready(Ok(self.svc.clone())))
    }
}

pin_project! {
    pub struct IntoMakeServiceFuture<S> {
        #[pin]
        future: Ready<Result<S, Infallible>>,
    }
}

impl<S> IntoMakeServiceFuture<S> {
    pub(crate) fn new(future: Ready<Result<S, Infallible>>) -> Self {
        Self { future }
    }
}

impl<S> fmt::Debug for IntoMakeServiceFuture<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoMakeServiceFuture")
            .finish_non_exhaustive()
    }
}

impl<S> Future for IntoMakeServiceFuture<S>
where
    Ready<Result<S, Infallible>>: Future,
{
    type Output = <Ready<Result<S, Infallible>> as Future>::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().future.poll(cx)
    }
}
