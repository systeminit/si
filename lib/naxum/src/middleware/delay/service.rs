use std::{
    convert::Infallible,
    task::{Context, Poll},
    time::Duration,
};

use tokio::time;
use tower::Service;

use super::future::ResponseFuture;

pub struct Delay<S> {
    pub(crate) inner: S,
    pub(crate) wait: Duration,
}

impl<S> Delay<S> {
    pub fn new(inner: S, wait: Duration) -> Self {
        Self { inner, wait }
    }
}

impl<S, Request> Service<Request> for Delay<S>
where
    S: Service<Request, Error = Infallible>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.inner.poll_ready(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(r) => Poll::Ready(r.map_err(Into::into)),
        }
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let response = self.inner.call(request);
        let sleep = time::sleep(self.wait);

        ResponseFuture { response, sleep }
    }
}
