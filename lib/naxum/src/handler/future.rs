//! Handler future types.

#![allow(clippy::type_complexity)] // For `Map<F, fn....>` type

use std::{
    convert::Infallible,
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::future::Map;
use pin_project_lite::pin_project;
use tower::{util::Oneshot, Service};

use crate::{response::Response, MessageHead};

pin_project! {
    pub struct IntoServiceFuture<F> {
        #[pin]
        future: Map<
            F,
            fn(Response) -> Result<Response, Infallible>,
        >,
    }
}

impl<F> IntoServiceFuture<F> {
    pub(crate) fn new(future: Map<F, fn(Response) -> Result<Response, Infallible>>) -> Self {
        Self { future }
    }
}

impl<F> fmt::Debug for IntoServiceFuture<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoServiceFuture").finish_non_exhaustive()
    }
}

impl<F> Future for IntoServiceFuture<F>
where
    Map<F, fn(Response) -> Result<Response, Infallible>>: Future,
{
    type Output = <Map<F, fn(Response) -> Result<Response, Infallible>> as Future>::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().future.poll(cx)
    }
}

pin_project! {
    pub struct LayeredFuture<S, R>
    where
        S: Service<R>,
        R: MessageHead,
    {
        #[pin]
        inner: Map<Oneshot<S, R>, fn(Result<S::Response, S::Error>) -> Response>,
    }
}

impl<S, R> LayeredFuture<S, R>
where
    S: Service<R>,
    R: MessageHead,
{
    pub(super) fn new(
        inner: Map<Oneshot<S, R>, fn(Result<S::Response, S::Error>) -> Response>,
    ) -> Self {
        Self { inner }
    }
}

impl<S, R> Future for LayeredFuture<S, R>
where
    S: Service<R>,
    R: MessageHead,
{
    type Output = Response;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}
