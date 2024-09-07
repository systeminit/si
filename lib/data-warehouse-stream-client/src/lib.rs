//! This crate provides a client for streaming data directly or eventually to a data warehouse.

use aws_sdk_firehose::{operation::put_record::PutRecordError, primitives::Blob, types::Record};
use base64::{engine::general_purpose, Engine};
use telemetry::prelude::*;
use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DataWarehouseStreamClientError {
    #[error("firehose error: {0}")]
    Firehose(#[from] aws_sdk_firehose::Error),
    #[error("firehose build error: {0}")]
    FirehoseBuild(#[from] aws_sdk_firehose::error::BuildError),
    #[error("firehose put record error: {0}")]
    FirehosePutRecord(#[from] aws_sdk_firehose::error::SdkError<PutRecordError>),
}

type DataWarehouseStreamClientResult<T> = Result<T, DataWarehouseStreamClientError>;

#[derive(Debug, Clone)]
pub struct DataWarehouseStreamClient {
    delivery_stream_name: String,
    inner: Box<aws_sdk_firehose::Client>,
}

impl DataWarehouseStreamClient {
    #[instrument(
        name = "data_warehouse_stream_client.new",
        level = "info",
        skip(delivery_stream_name)
    )]
    pub async fn new(delivery_stream_name: impl Into<String>) -> Self {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_firehose::Client::new(&config);
        Self {
            inner: Box::new(client),
            delivery_stream_name: delivery_stream_name.into(),
        }
    }

    #[instrument(
        name = "data_warehouse_stream_client.publish",
        level = "debug",
        skip(raw_data)
    )]
    pub async fn publish(&self, raw_data: impl AsRef<[u8]>) -> DataWarehouseStreamClientResult<()> {
        let record = Record::builder()
            .data(Blob::new(Self::base64_encode_data(raw_data)))
            .build()?;
        let output = self
            .inner
            .put_record()
            .delivery_stream_name(&self.delivery_stream_name)
            .record(record)
            .send()
            .await?;

        // TODO(nick): remove this is replace with formal logging. We will keep this until we actually
        // use the client.
        dbg!(&output);

        Ok(())
    }

    fn base64_encode_data(input: impl AsRef<[u8]>) -> String {
        general_purpose::STANDARD_NO_PAD.encode(input)
    }
}
