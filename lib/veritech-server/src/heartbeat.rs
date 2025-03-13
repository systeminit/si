//! This module contains the [`HeartbeatApp`] used for assessing the health of veritech as well as
//! performing force reconnects as needed.

use std::sync::atomic::Ordering;
use std::time::Duration;

use si_data_nats::{NatsClient, State, Subject};
use telemetry::prelude::*;
use telemetry_utils::metric;
use tokio_util::sync::CancellationToken;

const HEARTBEAT_SUBJECT_PREFIX: &str = "veritech.heartbeat";

/// An app for assessing the health of veritech as well as performing force reconnects as needed.
#[derive(Debug)]
pub struct HeartbeatApp {
    nats: NatsClient,
    token: CancellationToken,
    auto_force_reconnect_logic_enabled: bool,
    sleep_duration: Duration,
    publish_timeout_duration: Duration,
    force_reconnect_timeout_duration: Duration,
    heartbeat_subject: Subject,
    heartbeat_payload: Vec<u8>,
    needs_force_reconnect: bool,
}

impl HeartbeatApp {
    /// Creates a new [`HeartbeatApp`].
    pub fn new(
        nats: NatsClient,
        token: CancellationToken,
        instance_id: &str,
        enable_auto_force_reconnect_logic: bool,
        sleep_duration: Duration,
        publish_timeout_duration: Duration,
        force_reconnect_timeout_duration: Duration,
    ) -> Self {
        let heartbeat_subject = Subject::from(match nats.metadata().subject_prefix() {
            Some(prefix) => format!("{prefix}.{HEARTBEAT_SUBJECT_PREFIX}.{instance_id}"),
            None => format!("{HEARTBEAT_SUBJECT_PREFIX}.{instance_id}"),
        });

        Self {
            nats,
            token,
            auto_force_reconnect_logic_enabled: enable_auto_force_reconnect_logic,
            sleep_duration,
            publish_timeout_duration,
            force_reconnect_timeout_duration,
            heartbeat_subject,
            heartbeat_payload: Vec::new(),
            needs_force_reconnect: false,
        }
    }

    /// Runs the [`HeartbeatApp`]. This method explicitly does not return anything.
    pub async fn run(&mut self) {
        info!(
            veritech.heartbeat.auto_force_reconnect_logic.enabled = self.auto_force_reconnect_logic_enabled,
            veritech.heartbeat.sleep_duration = ?self.sleep_duration,
            veritech.heartbeat.publish_timeout_duration = ?self.publish_timeout_duration,
            veritech.heartbeat.force_reconnect_timeout_duration = ?self.force_reconnect_timeout_duration,
            "running heartbeat app"
        );

        // Reset metrics before running the core loop.
        metric!(counter.veritech.heartbeat.loop_iteration = 0);
        metric!(counter.veritech.heartbeat.publish.success = 0);
        metric!(counter.veritech.heartbeat.publish.error = 0);
        metric!(counter.veritech.heartbeat.publish.timeout = 0);
        metric!(counter.veritech.heartbeat.force_reconnect.success = 0);
        metric!(counter.veritech.heartbeat.force_reconnect.error = 0);
        metric!(counter.veritech.heartbeat.force_reconnect.timeout = 0);
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
        // Either perform the reconnect or publish the heartbeat, but do not do both. We only do
        // force reconnection if the app has the feature enabled (which it likely is, by default).
        if self.auto_force_reconnect_logic_enabled && self.needs_force_reconnect {
            self.perform_force_reconnect().await
        } else {
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
                        self.needs_force_reconnect = true;
                    }
                },
                Err(_) => {
                    metric!(counter.veritech.heartbeat.publish.timeout = 1);
                    self.needs_force_reconnect = true;
                }
            }
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

    async fn perform_force_reconnect(&mut self) {
        match tokio::time::timeout(
            self.force_reconnect_timeout_duration,
            self.nats.force_reconnect(),
        )
        .await
        {
            Ok(force_reconnect_result) => match force_reconnect_result {
                Ok(()) => {
                    metric!(counter.veritech.heartbeat.force_reconnect.success = 1);
                    self.needs_force_reconnect = false;
                }
                Err(err) => {
                    error!(si.error.message = ?err, "heartbeat: force reconnect error");
                    metric!(counter.veritech.heartbeat.force_reconnect.error = 1);
                    self.needs_force_reconnect = true;
                }
            },
            Err(_) => {
                metric!(counter.veritech.heartbeat.force_reconnect.timeout = 1);
                self.needs_force_reconnect = true;
            }
        }
    }
}
