use rabbitmq_stream_client::error::{
    ClientError, ConsumerCloseError, ConsumerCreateError, ConsumerDeliveryError,
    ProducerCloseError, ProducerCreateError, ProducerPublishError, StreamCreateError,
    StreamDeleteError,
};
use std::string::FromUtf8Error;
use thiserror::Error;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum RabbitError {
    #[error("client error: {0}")]
    Client(#[from] ClientError),
    #[error("consumer close error: {0}")]
    ConsumerClose(#[from] ConsumerCloseError),
    #[error("consumer create error: {0}")]
    ConsumerCreate(#[from] ConsumerCreateError),
    #[error("consumer delivery error: {0}")]
    ConsumerDelivery(#[from] ConsumerDeliveryError),
    #[error("from utf-8 error: {0}")]
    FromUtf8(#[from] FromUtf8Error),
    #[error("producer close error: {0}")]
    ProducerClose(#[from] ProducerCloseError),
    #[error("can no longer use producer because it has been closed")]
    ProducerClosed,
    #[error("producer create error: {0}")]
    ProducerCreate(#[from] ProducerCreateError),
    #[error("producer publish error: {0}")]
    ProducerPublish(#[from] ProducerPublishError),
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("stream create error: {0}")]
    StreamCreate(#[from] StreamCreateError),
    #[error("stream delete error: {0}")]
    StreamDelete(#[from] StreamDeleteError),
}

#[allow(missing_docs)]
pub type RabbitResult<T> = Result<T, RabbitError>;
