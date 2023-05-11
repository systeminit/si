//! This module contains [`StatusReceiverClient`], which is used to talk to the
//! [`StatusReceiver`](crate::tasks::StatusReceiver).

use nats_subscriber::SubscriberError;
use serde::Serialize;
use si_data_nats::{NatsClient, NatsError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::tasks::status_receiver::{StatusReceiverRequest, STATUS_RECEIVER_REQUEST_SUBJECT};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum StatusReceiverClientError {
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("nats error")]
    Nats(#[from] NatsError),
    #[error("result error")]
    Result(#[from] SubscriberError),
}

pub type StatusReceiverClientResult<T> = Result<T, StatusReceiverClientError>;

/// The client used to talk to the [`StatusReceiver`](crate::tasks::StatusReceiver)
/// over [NATS](https://nats.io).
#[derive(Clone, Debug)]
pub struct StatusReceiverClient {
    nats_client: NatsClient,
}

impl StatusReceiverClient {
    /// Create a new [`client`](Self).
    pub async fn new(nats_client: NatsClient) -> Self {
        Self { nats_client }
    }

    /// Publishes a [`request`](crate::tasks::StatusReceiverRequest) to
    /// [NATS](https://nats.io) with the appropriate subject.
    ///
    /// The request is performed in a UDP-like manner without a reply mailbox.
    #[instrument(name = "status_receiver_client.publish", skip_all)]
    pub async fn publish(&self, request: &StatusReceiverRequest) -> StatusReceiverClientResult<()> {
        self.execute_request(STATUS_RECEIVER_REQUEST_SUBJECT, request)
            .await
    }

    /// A generic UDP-style publisher to [NATS](https://nats.io) with a given subject and
    /// serializable request.
    async fn execute_request<R>(
        &self,
        subject: impl Into<String>,
        request: &R,
    ) -> StatusReceiverClientResult<()>
    where
        R: Serialize,
    {
        let msg = serde_json::to_vec(request).map_err(StatusReceiverClientError::JSONSerialize)?;
        let subject = subject.into();
        trace!(
            messaging.destination = &subject.as_str(),
            "publishing message"
        );
        self.nats_client.publish(subject, msg).await?;
        Ok(())
    }
}
