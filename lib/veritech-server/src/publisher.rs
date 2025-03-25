use serde::Serialize;
use si_data_nats::{NatsClient, Subject};
use si_pool_noodle::{FunctionResult, OutputStream};
use telemetry::prelude::*;
use telemetry_nats::propagation;
use thiserror::Error;
use veritech_core::{reply_mailbox_for_output, reply_mailbox_for_result, FINAL_MESSAGE_HEADER_KEY};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PublisherError {
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("failed to publish message to nats subject: {1}")]
    NatsPublish(#[source] si_data_nats::NatsError, String),
}

type Result<T> = std::result::Result<T, PublisherError>;

#[derive(Debug)]
pub struct Publisher<'a> {
    nats: &'a NatsClient,
    reply_mailbox_output: Subject,
    reply_mailbox_result: Subject,
}

impl<'a> Publisher<'a> {
    pub fn new(nats: &'a NatsClient, reply_mailbox: &str) -> Self {
        Self {
            nats,
            reply_mailbox_output: reply_mailbox_for_output(reply_mailbox).into(),
            reply_mailbox_result: reply_mailbox_for_result(reply_mailbox).into(),
        }
    }

    #[instrument(
        name = "veritech.publisher.publish_output",
        level = "info",
        skip_all,
        fields(
            veritech.publisher.publish_output.duration_ms = Empty,
            veritech.publisher.publish_output.size = Empty,
            veritech.publisher.publish_output.reply_mailbox_output = Empty,
            veritech.publisher.publish_output.execution_id = Empty,
            veritech.publisher.publish_output.execution_kind = Empty,
            veritech.publisher.publish_output.count = Empty,
        )
    )]
    pub async fn publish_output(
        &self,
        output: &OutputStream,
        execution_id: String,
        execution_kind: String,
        count: usize,
    ) -> Result<()> {
        let span = current_span_for_instrument_at!("info");

        let nats_msg = serde_json::to_string(output).map_err(PublisherError::JSONSerialize)?;

        span.record("veritech.publisher.publish_output.size", nats_msg.len());
        span.record(
            "veritech.publisher.publish_output.result_mailbox_output",
            self.reply_mailbox_output.to_string(),
        );
        span.record(
            "veritech.publisher.publish_output.execution_id",
            execution_id,
        );
        span.record(
            "veritech.publisher.publish_output.execution_kind",
            execution_kind,
        );
        span.record("veritech.publisher.publish_output.count", count);

        let start = tokio::time::Instant::now();
        let result = self
            .nats
            .publish_with_headers(
                self.reply_mailbox_output.clone(),
                propagation::empty_injected_headers(),
                nats_msg.into(),
            )
            .await
            .map_err(|err| PublisherError::NatsPublish(err, self.reply_mailbox_output.to_string()));
        span.record(
            "veritech.publisher.publish_output.duration_ms",
            start.elapsed().as_millis(),
        );
        result
    }

    #[instrument(
        name = "veritech.publisher.finalize_output",
        level = "info",
        skip_all,
        fields(
            veritech.publisher.finalize_output.duration_ms = Empty,
            veritech.publisher.finalize_output.result_mailbox_output= Empty,
            veritech.publisher.finalize_output.execution_id = Empty,
            veritech.publisher.finalize_output.execution_kind = Empty,
        )
    )]
    pub async fn finalize_output(
        &self,
        execution_id: String,
        execution_kind: String,
    ) -> Result<()> {
        let span = current_span_for_instrument_at!("info");

        let mut headers = si_data_nats::HeaderMap::new();
        headers.insert(FINAL_MESSAGE_HEADER_KEY, "true");
        propagation::inject_headers(&mut headers);

        span.record(
            "veritech.publisher.finalize_output.result_mailbox_output",
            self.reply_mailbox_output.to_string(),
        );
        span.record(
            "veritech.publisher.finalize_output.execution_id",
            execution_id,
        );
        span.record(
            "veritech.publisher.finalize_output.execution_kind",
            execution_kind,
        );

        let start = tokio::time::Instant::now();
        let result = self
            .nats
            .publish_with_headers(self.reply_mailbox_output.clone(), headers, vec![].into())
            .await
            .map_err(|err| PublisherError::NatsPublish(err, self.reply_mailbox_output.to_string()));
        span.record(
            "veritech.publisher.finalize_output.duration_ms",
            start.elapsed().as_millis(),
        );
        result
    }

    #[instrument(
        name = "veritech.publisher.publish_result",
        level = "info",
        skip_all,
        fields(
            veritech.publisher.publish_result.duration_ms = Empty,
            veritech.publisher.publish_result.size = Empty,
            veritech.publisher.publish_result.reply_mailbox_result = Empty,
            veritech.publisher.publish_result.execution_id = Empty,
            veritech.publisher.publish_result.execution_kind = Empty,
        )
    )]
    pub async fn publish_result<R>(
        &self,
        result: &FunctionResult<R>,
        execution_id: String,
        execution_kind: String,
    ) -> Result<()>
    where
        R: Serialize,
    {
        let span = current_span_for_instrument_at!("info");

        let nats_msg = serde_json::to_string(result).map_err(PublisherError::JSONSerialize)?;

        span.record("veritech.publisher.publish_result.size", nats_msg.len());
        span.record(
            "veritech.publisher.publish_result.reply_mailbox_result",
            self.reply_mailbox_result.to_string(),
        );
        span.record(
            "veritech.publisher.publish_result.execution_id",
            execution_id,
        );
        span.record(
            "veritech.publisher.publish_result.execution_kind",
            execution_kind,
        );

        let start = tokio::time::Instant::now();
        let result = self
            .nats
            .publish_with_headers(
                self.reply_mailbox_result.clone(),
                propagation::empty_injected_headers(),
                nats_msg.into(),
            )
            .await
            .map_err(|err| PublisherError::NatsPublish(err, self.reply_mailbox_result.to_string()));
        span.record(
            "veritech.publisher.publish_result.duration_ms",
            start.elapsed().as_millis(),
        );
        result
    }
}
