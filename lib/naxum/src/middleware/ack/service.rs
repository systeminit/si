use std::{
    sync::Arc,
    task::{
        Context,
        Poll,
    },
    time::Duration,
};

use async_nats::jetstream;
use tokio_util::sync::CancellationToken;
use tower::Service;
use tracing::warn;

use super::{
    future::ResponseFuture,
    info::Info,
    layer::AckLayer,
    maintain_progress::MaintainProgressTask,
    on_failure::{
        DefaultOnFailure,
        OnFailure,
    },
    on_success::{
        DefaultOnSuccess,
        OnSuccess,
    },
};
use crate::{
    message::{
        Message,
        MessageHead,
    },
    response::Response,
};

#[derive(Clone, Debug)]
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

impl<S, OnSuccessT, OnFailureT> Service<Message<jetstream::Message>>
    for Ack<S, OnSuccessT, OnFailureT>
where
    S: Service<Message<async_nats::Message>, Response = Response>,
    OnSuccessT: OnSuccess,
    OnFailureT: OnFailure,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Message<jetstream::Message>) -> Self::Future {
        // Split into jetstream message & extensions
        let (jetstream_message, extensions) = req.split();

        // Extract message info BEFORE splitting (contains delivery count for backoff)
        let info = Arc::new(match jetstream_message.info() {
            Ok(info) => Info::from(info),
            Err(err) => {
                // This shouldn't happen for valid JetStream messages, but handle gracefully
                warn!(
                    si.error.message = ?err,
                    "failed to parse jetstream message info, using defaults"
                );
                Info {
                    domain: None,
                    acc_hash: None,
                    stream: String::new(),
                    consumer: String::new(),
                    stream_sequence: 0,
                    consumer_sequence: 0,
                    delivered: 1, // Assume first delivery
                    pending: 0,
                    published: time::OffsetDateTime::now_utc(),
                    token: None,
                }
            }
        });

        // Split off acker from jetstream message which is now a core message
        let (core_message, acker) = jetstream_message.split();
        let acker = Arc::new(acker);

        // Decompose the core message into head and payload
        let mut parts = core_message.into_head_and_payload();

        // Append remaining extensions into head and save copy of head
        parts.0.extensions.extend(extensions);
        let head = Arc::new(parts.0.clone());

        // Reconstruct a core message from head and payload
        let (core_message, extensions) =
            match <async_nats::Message as MessageHead>::from_head_and_payload(parts.0, parts.1) {
                Ok(msg_and_exts) => msg_and_exts,
                Err(err) => unreachable!(
                    "NATS core message from parts is infallible, this is a bug!; error={:?}",
                    err
                ),
            };

        // Create final message from core message and remaining extensions
        let message = Message::new_with_extensions(core_message, extensions);

        let task_shutdown = CancellationToken::new();

        let task =
            MaintainProgressTask::new(acker.clone(), self.progress_period, task_shutdown.clone());
        tokio::spawn(task.run());
        // The drop guard will trigger a `cancel` on the token to ensure the task is shutdown even
        // if the response future has issues
        let shutdown_guard = task_shutdown.drop_guard();

        let response = self.inner.call(message);

        let on_success_fut = self.on_success.call(head.clone(), acker.clone());
        let on_failure_fut = self
            .on_failure
            .call(head.clone(), acker.clone(), info.clone());

        ResponseFuture {
            inner: response,
            on_success_fut,
            on_failure_fut,
            state: super::future::State::default(),
            shutdown_guard: Some(shutdown_guard),
        }
    }
}
