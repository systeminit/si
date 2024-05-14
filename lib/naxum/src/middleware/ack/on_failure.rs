use std::sync::Arc;

use async_nats::jetstream::{self, message::Acker};
use futures::future::BoxFuture;
use tracing::{trace, warn};

use crate::Head;

pub trait OnFailure {
    fn call(&mut self, head: Arc<Head>, acker: Arc<Acker>) -> BoxFuture<'static, ()>;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultOnFailure {}

impl DefaultOnFailure {
    pub fn new() -> Self {
        Self::default()
    }
}

impl OnFailure for DefaultOnFailure {
    fn call(&mut self, head: Arc<Head>, acker: Arc<Acker>) -> BoxFuture<'static, ()> {
        Box::pin(async move {
            trace!("nacking message");
            if let Err(err) = acker.ack_with(jetstream::AckKind::Nak(None)).await {
                warn!(
                    error = ?err,
                    subject = head.subject.as_str(),
                    "failed to nack the message",
                );
            }
        })
    }
}
