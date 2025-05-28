use std::{
    sync::Arc,
    task::{
        Context,
        Poll,
    },
};

use tower::Service;

use super::{
    DefaultOnFailure,
    DefaultOnSuccess,
    OnFailure,
    OnSuccess,
    PostProcessLayer,
    future::ResponseFuture,
};
use crate::{
    message::{
        Message,
        MessageHead,
    },
    response::Response,
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

impl<S, OnSuccessT, OnFailureT, R> Service<Message<R>> for PostProcess<S, OnSuccessT, OnFailureT>
where
    S: Service<Message<R>, Response = Response>,
    OnSuccessT: OnSuccess,
    OnFailureT: OnFailure,
    R: MessageHead,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S, R>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Message<R>) -> Self::Future {
        // Decontruct the message into head and payload and save a copy of head
        let parts = req.into_parts();
        let head = Arc::new(parts.0.clone());

        // Reconstruct a message from head and payload
        let (inner, extensions) = match <R as MessageHead>::from_head_and_payload(parts.0, parts.1)
        {
            Ok(message) => message,
            Err(err) => unreachable!(
                "message from parts should succeed, this is a bug!; error={:?}",
                err
            ),
        };

        // Create final message from inner message and remaining extensions
        let message = Message::new_with_extensions(inner, extensions);

        let response = self.inner.call(message);

        let on_success_fut = self.on_success.call(head.clone());
        let on_failure_fut = self.on_failure.call(head.clone());

        ResponseFuture {
            inner: response,
            on_success_fut,
            on_failure_fut,
            state: super::future::State::default(),
        }
    }
}
