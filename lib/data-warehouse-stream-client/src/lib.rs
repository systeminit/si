//! This crate provides a client for streaming data directly or eventually to a data warehouse.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use aws_sdk_firehose::{
    operation::put_record::PutRecordError,
    primitives::Blob,
    types::Record,
};
use si_aws_config::{
    AwsConfig,
    AwsConfigError,
};
use telemetry::prelude::*;
use thiserror::Error;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum DataWarehouseStreamClientError {
    #[error("AWS Config Error error: {0}")]
    AwsConfig(#[from] AwsConfigError),
    #[error("firehose error: {0}")]
    Firehose(#[from] aws_sdk_firehose::Error),
    #[error("firehose build error: {0}")]
    FirehoseBuild(#[from] Box<aws_sdk_firehose::error::BuildError>),
    #[error("firehose put record error: {0}")]
    FirehosePutRecord(#[from] Box<aws_sdk_firehose::error::SdkError<PutRecordError>>),
}

impl From<aws_sdk_firehose::error::BuildError> for DataWarehouseStreamClientError {
    fn from(value: aws_sdk_firehose::error::BuildError) -> Self {
        Box::new(value).into()
    }
}

impl From<aws_sdk_firehose::error::SdkError<PutRecordError>> for DataWarehouseStreamClientError {
    fn from(value: aws_sdk_firehose::error::SdkError<PutRecordError>) -> Self {
        Box::new(value).into()
    }
}

type DataWarehouseStreamClientResult<T> = Result<T, DataWarehouseStreamClientError>;

/// A client for communicating with a stream to a data warehouse.
#[derive(Debug, Clone)]
pub struct DataWarehouseStreamClient {
    delivery_stream_name: String,
    inner: Box<aws_sdk_firehose::Client>,
}

impl DataWarehouseStreamClient {
    /// Creates a new [client for communicating with a stream to a data warehouse](DataWarehouseStreamClient).
    #[instrument(
        name = "data_warehouse_stream_client.new",
        level = "info",
        skip(delivery_stream_name)
    )]
    pub async fn new(
        delivery_stream_name: impl Into<String>,
    ) -> DataWarehouseStreamClientResult<Self> {
        let config = AwsConfig::from_env().await?;
        let client = aws_sdk_firehose::Client::new(&config);
        Ok(Self {
            inner: Box::new(client),
            delivery_stream_name: delivery_stream_name.into(),
        })
    }

    /// Publishes a message to a stream to a data warehouse.
    #[instrument(
        name = "data_warehouse_stream_client.publish",
        level = "debug",
        skip(raw_data)
    )]
    pub async fn publish(&self, raw_data: impl AsRef<[u8]>) -> DataWarehouseStreamClientResult<()> {
        let record = Record::builder()
            .data(Blob::new(raw_data.as_ref()))
            .build()?;
        let output = self
            .inner
            .put_record()
            .delivery_stream_name(&self.delivery_stream_name)
            .record(record)
            .send()
            .await?;
        debug!(
            ?output,
            "output from sending put record request to kinesis firehose stream"
        );
        Ok(())
    }
}
