//! This module contains the [`HeartbeatApp`] used for assessing the health of veritech and its NATS client.

use std::{
    sync::atomic::Ordering,
    time::Duration,
};

use si_data_nats::{
    NatsClient,
    State,
    Subject,
};
use telemetry::prelude::*;
use telemetry_utils::metric;
use tokio_util::sync::CancellationToken;

const HEARTBEAT_SUBJECT_PREFIX: &str = "veritech.heartbeat";

/// An app for assessing the health of veritech and its NATS client.
#[derive(Debug)]
pub struct HeartbeatApp {
    nats: NatsClient,
    token: CancellationToken,
    sleep_duration: Duration,
    publish_timeout_duration: Duration,
    heartbeat_subject: Subject,
    heartbeat_payload: Vec<u8>,
}

impl HeartbeatApp {
    /// Creates a new [`HeartbeatApp`].
    pub fn new(
        nats: NatsClient,
        token: CancellationToken,
        instance_id: &str,
        sleep_duration: Duration,
        publish_timeout_duration: Duration,
    ) -> Self {
        let heartbeat_subject = Subject::from(match nats.metadata().subject_prefix() {
            Some(prefix) => format!("{prefix}.{HEARTBEAT_SUBJECT_PREFIX}.{instance_id}"),
            None => format!("{HEARTBEAT_SUBJECT_PREFIX}.{instance_id}"),
        });

        Self {
            nats,
            token,
            sleep_duration,
            publish_timeout_duration,
            heartbeat_subject,
            heartbeat_payload: Vec::new(),
        }
    }

    /// Runs the [`HeartbeatApp`]. This method explicitly does not return anything.
    pub async fn run(&mut self) {
        info!(
            veritech.heartbeat.sleep_duration = ?self.sleep_duration,
            veritech.heartbeat.publish_timeout_duration = ?self.publish_timeout_duration,
            "running heartbeat app"
        );

        // Reset metrics before running the core loop.
        metric!(counter.veritech.heartbeat.loop_iteration = 0);
        metric!(counter.veritech.heartbeat.publish.success = 0);
        metric!(counter.veritech.heartbeat.publish.error = 0);
        metric!(counter.veritech.heartbeat.publish.timeout = 0);
        metric!(counter.veritech.heartbeat.connection_state.connected = 0);
        metric!(counter.veritech.heartbeat.connection_state.disconnected = 0);
        metric!(counter.veritech.heartbeat.connection_state.pending = 0);

        loop {
            metric!(counter.veritech.heartbeat.loop_iteration = 1);
            tokio::select! {
                _ = tokio::time::sleep(self.sleep_duration) => {
                    self.perform_heartbeat().await;
                }
                _ = self.token.cancelled() => {
                    info!("heartbeat: shutting down...");
                    break;
                }
            }
        }
    }

    async fn perform_heartbeat(&mut self) {
        match tokio::time::timeout(
            self.publish_timeout_duration,
            self.nats.publish(
                self.heartbeat_subject.to_owned(),
                self.heartbeat_payload.to_owned().into(),
            ),
        )
        .await
        {
            Ok(publish_result) => match publish_result {
                Ok(()) => {
                    metric!(counter.veritech.heartbeat.publish.success = 1);
                }
                Err(err) => {
                    error!(si.error.message = ?err, "heartbeat: publish error");
                    metric!(counter.veritech.heartbeat.publish.error = 1);
                }
            },
            // NOTE(nick): this is going to be an "Elapsed" error, which contains a single private
            // field: the unit type. As a result of this, we cannot match on the specific error. I
            // don't love the underscore here in case the underlying API changes, but here we are.
            Err(_) => metric!(counter.veritech.heartbeat.publish.timeout = 1),
        }

        // Track the connection state, which does not use internal channel(s).
        match self.nats.connection_state() {
            State::Connected => {
                metric!(counter.veritech.heartbeat.connection_state.connected = 1);
            }
            State::Disconnected => {
                metric!(counter.veritech.heartbeat.connection_state.disconnected = 1);
            }
            State::Pending => {
                metric!(counter.veritech.heartbeat.connection_state.pending = 1);
            }
        }

        // Gather statistics, which do not use internal channel(s).
        let statistics = self.nats.statistics();
        metric!(
            histogram.veritech.heartbeat.statistics.in_bytes =
                statistics.in_bytes.load(Ordering::Relaxed)
        );
        metric!(
            histogram.veritech.heartbeat.statistics.out_bytes =
                statistics.out_bytes.load(Ordering::Relaxed)
        );
        metric!(
            histogram.veritech.heartbeat.statistics.in_messages =
                statistics.in_messages.load(Ordering::Relaxed)
        );
        metric!(
            histogram.veritech.heartbeat.statistics.out_messages =
                statistics.out_messages.load(Ordering::Relaxed)
        );
        metric!(
            histogram.veritech.heartbeat.statistics.connects =
                statistics.connects.load(Ordering::Relaxed)
        );
    }
}
