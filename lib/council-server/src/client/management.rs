//! This module contains [`ManagementClient`], which is used to subscribe to management-type messages from a council
//! server (example: notify all running pinga instances that they should [`restart`](ManagementResponse::Restart)
//! actively running jobs).

use futures::StreamExt;
use si_data_nats::{NatsClient, Subject, Subscriber};
use std::time::Duration;
use telemetry::{prelude::*, tracing::field};
use telemetry_nats::propagation;

use crate::client::{ClientError, ClientResult};
use crate::{ManagementResponse, SubjectGenerator};

#[derive(Debug)]
pub struct ManagementClient {
    management_channel: Subject,
    management_subscriber: Subscriber,
}

impl ManagementClient {
    pub async fn new(nats: &NatsClient, subject_prefix: Option<String>) -> ClientResult<Self> {
        let management_channel = SubjectGenerator::for_management_client(subject_prefix);
        Ok(Self {
            management_subscriber: nats.subscribe(management_channel.clone()).await?,
            management_channel: management_channel.into(),
        })
    }

    // None means subscriber has been unsubscribed or that the connection has been closed
    #[instrument(
        name = "council_management_client.fetch_response",
        level = "info",
        skip_all,
        fields(
            response = Empty,
        )
    )]
    pub async fn fetch_response(&mut self) -> ClientResult<Option<ManagementResponse>> {
        // TODO: timeout so we don't get stuck here forever if council goes away
        // TODO: handle message.data() empty with Status header as 503:
        // https://github.com/nats-io/nats.go/pull/576
        let msg = loop {
            let res =
                tokio::time::timeout(Duration::from_secs(60), self.management_subscriber.next())
                    .await;

            match res {
                Ok(msg) => break msg,
                Err(_) => {
                    warn!(
                        management_channel = ?self.management_channel,
                        "Council client waiting for response on management channel for 60 seconds",
                    );
                }
            }
        };

        match msg {
            Some(msg) => {
                let span = Span::current();
                propagation::associate_current_span_from_headers(msg.headers());
                if msg.payload().is_empty() {
                    return Err(ClientError::NoListenerAvailable);
                }
                let response = serde_json::from_slice::<ManagementResponse>(msg.payload())?;
                span.record("response", field::debug(&response));
                Ok(Some(response))
            }
            None => Ok(None),
        }
    }
}
