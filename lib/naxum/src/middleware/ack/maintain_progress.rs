use std::{sync::Arc, time::Duration};

use async_nats::jetstream::{self, message::Acker};
use tokio::time::{self, Instant, Interval};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, trace, warn};

pub struct MaintainProgressTask {
    acker: Arc<Acker>,
    interval: Interval,
    shutdown_token: CancellationToken,
}

impl MaintainProgressTask {
    const NAME: &'static str = "Naxum::Ack::MaintainProgressTask";

    pub fn new(
        acker: Arc<Acker>,
        progress_period: Duration,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            acker,
            interval: time::interval_at(Instant::now() + progress_period, progress_period),
            shutdown_token,
        }
    }

    pub async fn run(mut self) {
        trace!(si.naxum.task = Self::NAME, "running task");
        debug!(si.naxum.task = Self::NAME, "first ack message");
        if let Err(err) = self.acker.ack_with(jetstream::AckKind::Progress).await {
            warn!(si.error.message = ?err, "failed initial ack");
        }

        loop {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    info!(si.naxum.task = Self::NAME, "received cancellation");
                    break;
                }
                _ = self.interval.tick() => {
                    debug!(task = Self::NAME, "acking message with progress");
                    if let Err(err) = self.acker.ack_with(jetstream::AckKind::Progress).await {
                        warn!(si.error.message = ?err, "failed to ack with progress");
                    }
                }
            }
        }

        info!(si.naxum.task = Self::NAME, "naxum shutdown complete");
    }
}
