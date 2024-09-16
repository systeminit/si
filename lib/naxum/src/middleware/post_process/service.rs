use std::{
    sync::Arc,
    task::{Context, Poll},
};

use async_nats::jetstream;
use tower::Service;

use crate::{response::Response, MessageHead};

use super::{
    future::ResponseFuture, DefaultOnFailure, DefaultOnSuccess, Info, OnFailure, OnSuccess,
    PostProcessLayer,
};

#[derive(Clone, Copy, Debug)]
pub struct PostProcess<S, OnSuccess = DefaultOnSuccess, OnFailure = DefaultOnFailure> {
    pub(crate) inner: S,
    pub(crate) on_success: OnSuccess,
    pub(crate) on_failure: OnFailure,
}

impl<S> PostProcess<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            on_success: DefaultOnSuccess::default(),
            on_failure: DefaultOnFailure::default(),
        }
    }

    pub fn layer() -> PostProcessLayer {
        PostProcessLayer::new()
    }
}

impl<S, OnSuccessT, OnFailureT> Service<jetstream::Message>
    for PostProcess<S, OnSuccessT, OnFailureT>
where
    S: Service<jetstream::Message, Response = Response>,
    OnSuccessT: OnSuccess,
    OnFailureT: OnFailure,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: jetstream::Message) -> Self::Future {
        let parts = req.into_parts();
        let head = Arc::new(parts.0.clone());
        let message = match <jetstream::Message as MessageHead>::from_parts(parts.0, parts.1) {
            Ok(message) => message,
            Err(err) => unreachable!(
                "NATS Jetstream message from parts should succeed, this is a bug!; error={:?}",
                err
            ),
        };

        let info = Arc::new(Info::from(
            // TODO(fnichol): the middleware here is infallible, but this call could, in theory
            // error. There's probably a better alternative here...
            message.info().expect("failed to parse message info"),
        ));

        let response = self.inner.call(message);

        let on_success_fut = self.on_success.call(head.clone(), info.clone());
        let on_failure_fut = self.on_failure.call(head.clone(), info.clone());

        ResponseFuture {
            inner: response,
            on_success_fut,
            on_failure_fut,
            state: super::future::State::default(),
        }
    }
}
