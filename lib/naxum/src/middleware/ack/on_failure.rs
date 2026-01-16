use std::{
    sync::Arc,
    time::Duration,
};

use async_nats::jetstream::{
    self,
    message::Acker,
};
use futures::future::BoxFuture;
use telemetry_utils::metric;
use tracing::{
    debug,
    info,
    trace,
    warn,
};

use super::Info;
use crate::Head;

pub trait OnFailure {
    fn call(
        &mut self,
        head: Arc<Head>,
        acker: Arc<Acker>,
        info: Arc<Info>,
    ) -> BoxFuture<'static, ()>;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultOnFailure {}

impl DefaultOnFailure {
    pub fn new() -> Self {
        Self::default()
    }
}

impl OnFailure for DefaultOnFailure {
    fn call(
        &mut self,
        head: Arc<Head>,
        acker: Arc<Acker>,
        _info: Arc<Info>,
    ) -> BoxFuture<'static, ()> {
        Box::pin(async move {
            trace!("nacking message");
            if let Err(err) = acker.ack_with(jetstream::AckKind::Nak(None)).await {
                warn!(
                    si.error.message = ?err,
                    subject = head.subject.as_str(),
                    "failed to nack the message",
                );
            }
        })
    }
}

/// Failure handler with exponential backoff and max delivery termination.
/// Backoff: base_delay * 2^(attempt - 1), capped at max_delay.
/// On final attempt (delivered >= max_deliver), terminates instead of NAKing.
#[derive(Clone, Debug)]
pub struct BackoffOnFailure {
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub max_deliver: i64,
}

impl Default for BackoffOnFailure {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(120),
            max_deliver: 5,
        }
    }
}

impl BackoffOnFailure {
    pub fn new(max_deliver: i64) -> Self {
        Self {
            max_deliver,
            ..Default::default()
        }
    }

    pub fn with_params(base_delay: Duration, max_delay: Duration, max_deliver: i64) -> Self {
        Self {
            base_delay,
            max_delay,
            max_deliver,
        }
    }

    fn calculate_backoff(&self, delivered: i64) -> Duration {
        let attempt = (delivered - 1).max(0) as u32;
        let exponent = attempt.min(6);
        let delay = self.base_delay.saturating_mul(2_u32.pow(exponent));
        delay.min(self.max_delay)
    }

    fn is_final_attempt(&self, delivered: i64) -> bool {
        delivered >= self.max_deliver
    }
}

impl OnFailure for BackoffOnFailure {
    fn call(
        &mut self,
        head: Arc<Head>,
        acker: Arc<Acker>,
        info: Arc<Info>,
    ) -> BoxFuture<'static, ()> {
        let backoff = self.calculate_backoff(info.delivered);
        let is_final = self.is_final_attempt(info.delivered);
        let max_deliver = self.max_deliver;

        Box::pin(async move {
            if is_final {
                metric!(counter.naxum.max_delivery_terminated = 1);
                warn!(
                    delivered = info.delivered,
                    max_deliver = max_deliver,
                    subject = head.subject.as_str(),
                    "max delivery attempts reached, terminating message"
                );
                if let Err(err) = acker.ack_with(jetstream::AckKind::Term).await {
                    warn!(
                        si.error.message = ?err,
                        subject = head.subject.as_str(),
                        "failed to term the message",
                    );
                }
            } else {
                metric!(counter.naxum.nak_with_backoff = 1);
                debug!(
                    delivered = info.delivered,
                    backoff_ms = %backoff.as_millis(),
                    "nacking message with backoff"
                );
                if let Err(err) = acker.ack_with(jetstream::AckKind::Nak(Some(backoff))).await {
                    warn!(
                        si.error.message = ?err,
                        subject = head.subject.as_str(),
                        delivered = info.delivered,
                        "failed to nack the message",
                    );
                }
            }
        })
    }
}
