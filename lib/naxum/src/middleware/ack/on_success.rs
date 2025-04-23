use std::sync::Arc;

use async_nats::jetstream::message::Acker;
use futures::future::BoxFuture;
use telemetry_utils::metric;
use tracing::{
    error,
    info,
    trace,
};

use crate::Head;

pub trait OnSuccess {
    fn call(&mut self, head: Arc<Head>, acker: Arc<Acker>) -> BoxFuture<'static, ()>;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultOnSuccess {}

impl DefaultOnSuccess {
    pub fn new() -> Self {
        Self::default()
    }
}

impl OnSuccess for DefaultOnSuccess {
    fn call(&mut self, head: Arc<Head>, acker: Arc<Acker>) -> BoxFuture<'static, ()> {
        Box::pin(async move {
            trace!("double acking message");
            if let Err(err) = acker.double_ack().await {
                error!(
                    si.error.message = ?err,
                    subject = head.subject.as_str(),
                    "failed to double ack the message",
                );
                metric!(counter.naxum.ack_layer.on_success.double_ack_failed = 1);
            }
        })
    }
}
