use std::{
    sync::Arc,
    task::{
        Context,
        Poll,
    },
};

use async_nats::jetstream;
use tower::Service;

use super::{
    DefaultOnFailure,
    DefaultOnSuccess,
    Info,
    JetstreamPostProcessLayer,
    OnFailure,
    OnSuccess,
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
pub struct JetstreamPostProcess<S, OnSuccess = DefaultOnSuccess, OnFailure = DefaultOnFailure> {
    pub(crate) inner: S,
    pub(crate) on_success: OnSuccess,
    pub(crate) on_failure: OnFailure,
}

impl<S> JetstreamPostProcess<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            on_success: DefaultOnSuccess::default(),
            on_failure: DefaultOnFailure::default(),
        }
    }

    pub fn layer() -> JetstreamPostProcessLayer {
        JetstreamPostProcessLayer::new()
    }
}

impl<S, OnSuccessT, OnFailureT> Service<Message<jetstream::Message>>
    for JetstreamPostProcess<S, OnSuccessT, OnFailureT>
where
    S: Service<Message<jetstream::Message>, Response = Response>,
    OnSuccessT: OnSuccess,
    OnFailureT: OnFailure,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S, OnSuccessT, OnFailureT>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Message<jetstream::Message>) -> Self::Future {
        // Deconstruct the message into head and payload and save a copy of head
        let parts = req.into_parts();
        let head = Arc::new(parts.0.clone());

        // Reconstruct a jetstream message from head and payload
        let (jetstream_message, extensions) =
            match <jetstream::Message as MessageHead>::from_head_and_payload(parts.0, parts.1) {
                Ok(message) => message,
                Err(err) => unreachable!(
                    "NATS Jetstream message from parts should succeed, this is a bug!; error={:?}",
                    err
                ),
            };

        // Create an info from the jetstream message
        let info = Arc::new(Info::from(
            // TODO(fnichol): the middleware here is infallible, but this call could, in theory
            // error. There's probably a better alternative here...
            jetstream_message
                .info()
                .expect("failed to parse message info"),
        ));

        // Create final message from jetstream message and remaining extensions
        let message = Message::new_with_extensions(jetstream_message, extensions);

        let response = self.inner.call(message);

        ResponseFuture {
            inner: response,
            on_success: self.on_success.clone(),
            on_failure: self.on_failure.clone(),
            state: super::future::State::Initial(Some((head, info))),
        }
    }
}
