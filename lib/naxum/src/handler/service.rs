use std::{
    convert::Infallible,
    fmt,
    marker::PhantomData,
    task::{Context, Poll},
};

use futures::FutureExt;
use tower::Service;

use crate::{make_service::IntoMakeService, response::Response, MessageHead};

use super::Handler;

pub struct HandlerService<H, T, S, R> {
    handler: H,
    state: S,
    _marker: PhantomData<fn() -> T>,
    _request_marker: PhantomData<R>,
}

impl<H, T, S, R> HandlerService<H, T, S, R> {
    pub(super) fn new(handler: H, state: S) -> Self {
        Self {
            handler,
            state,
            _marker: PhantomData,
            _request_marker: PhantomData,
        }
    }

    pub fn into_make_service(self) -> IntoMakeService<HandlerService<H, T, S, R>> {
        IntoMakeService::new(self)
    }

    pub fn state(&self) -> &S {
        &self.state
    }
}

impl<H, T, S, R> fmt::Debug for HandlerService<H, T, S, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandlerService").finish_non_exhaustive()
    }
}

impl<H, T, S, R> Clone for HandlerService<H, T, S, R>
where
    H: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            state: self.state.clone(),
            _marker: PhantomData,
            _request_marker: PhantomData,
        }
    }
}

impl<H, T, S, R> Service<R> for HandlerService<H, T, S, R>
where
    H: Handler<T, S, R> + Clone + Send + 'static,
    S: Clone + Send + Sync,
    R: MessageHead,
{
    type Response = Response;
    type Error = Infallible;
    type Future = super::future::IntoServiceFuture<H::Future>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // `IntoService` can only be constructed from async functions which are always ready, or
        // from `Layered` which buffers in `<Layered as Handler>::call` and is therefore
        // also always ready.
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: R) -> Self::Future {
        let handler = self.handler.clone();
        let future = Handler::call(handler, req, self.state.clone());
        let future = future.map(Ok as _);

        super::future::IntoServiceFuture::new(future)
    }
}
