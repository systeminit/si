use std::{
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use async_nats::jetstream;
use tokio_util::sync::CancellationToken;
use tower::Service;

use crate::{response::Response, MessageHead};

use super::{
    future::ResponseFuture,
    layer::AckLayer,
    maintain_progress::MaintainProgressTask,
    on_failure::{DefaultOnFailure, OnFailure},
    on_success::{DefaultOnSuccess, OnSuccess},
};

#[derive(Clone, Copy, Debug)]
pub struct Ack<S, OnSuccess = DefaultOnSuccess, OnFailure = DefaultOnFailure> {
    pub(crate) inner: S,
    pub(crate) on_success: OnSuccess,
    pub(crate) on_failure: OnFailure,
    pub(crate) progress_period: Duration,
}

impl<S> Ack<S> {
    pub fn new(inner: S, progress_period: Duration) -> Self {
        Self {
            inner,
            on_success: DefaultOnSuccess::default(),
            on_failure: DefaultOnFailure::default(),
            progress_period,
        }
    }

    pub fn layer() -> AckLayer {
        AckLayer::new()
    }
}

impl<S, OnSuccessT, OnFailureT> Service<jetstream::Message> for Ack<S, OnSuccessT, OnFailureT>
where
    S: Service<async_nats::Message, Response = Response>,
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
        let (message, acker) = req.split();
        let acker = Arc::new(acker);
        let parts = message.into_parts();
        let head = Arc::new(parts.0.clone());
        let message = match <async_nats::Message as MessageHead>::from_parts(parts.0, parts.1) {
            Ok(message) => message,
            Err(err) => unreachable!(
                "NATS core message from parts is infallible, this is a bug!; error={:?}",
                err
            ),
        };

        let task_shutdown = CancellationToken::new();

        let task =
            MaintainProgressTask::new(acker.clone(), self.progress_period, task_shutdown.clone());
        tokio::spawn(task.run());
        // The drop guard will trigger a `cancel` on the token to ensure the task is shutdown even
        // if the response future has issues
        let shutdown_guard = task_shutdown.drop_guard();

        let response = self.inner.call(message);

        let on_success_fut = self.on_success.call(head.clone(), acker.clone());
        let on_failure_fut = self.on_failure.call(head.clone(), acker.clone());

        ResponseFuture {
            inner: response,
            on_success_fut,
            on_failure_fut,
            state: super::future::State::default(),
            shutdown_guard: Some(shutdown_guard),
        }
    }
}
