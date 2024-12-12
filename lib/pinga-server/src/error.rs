use naxum::response::{IntoResponse, Response};
use telemetry::prelude::*;

// use std::io;

// use dal::{DedicatedExecutorInitializeError, InitializationError, TransactionsError};
// use si_data_nats::{async_nats, NatsError};
// use si_data_pg::PgPoolError;
// use thiserror::Error;

// #[remain::sorted]
// #[derive(Debug, Error)]
// pub enum ServerError {
//     #[error("dal jetstream streams error: {0}")]
//     DalJetstreamStreams(#[from] dal::JetstreamStreamsError),
//     #[error("compute executor initialization error: {0}")]
//     DedicatedExecutorInitialize(#[from] DedicatedExecutorInitializeError),
//     #[error("initialization error: {0}")]
//     Initialization(#[from] InitializationError),
//     #[error("stream consumer error: {0}")]
//     JsConsumer(#[from] async_nats::jetstream::stream::ConsumerError),
//     #[error("consumer stream error: {0}")]
//     JsConsumerStream(#[from] async_nats::jetstream::consumer::StreamError),
//     #[error("stream create error: {0}")]
//     JsCreateStreamError(#[from] async_nats::jetstream::context::CreateStreamError),
//     #[error("layer cache error: {0}")]
//     LayerCache(#[from] si_layer_cache::LayerDbError),
//     #[error("failed to initialize a nats client: {0}")]
//     NatsClient(#[source] NatsError),
//     #[error("naxum error: {0}")]
//     Naxum(#[source] io::Error),
//     #[error("pg pool error: {0}")]
//     PgPool(#[from] Box<PgPoolError>),
//     #[error("rebaser client error: {0}")]
//     Rebaser(#[from] rebaser_client::ClientError),
//     #[error("symmetric crypto error: {0}")]
//     SymmetricCryptoService(#[from] si_crypto::SymmetricCryptoError),
//     #[error("transactions error: {0}")]
//     Transactions(#[from] TransactionsError),
//     #[error("error when loading cyclone encryption key: {0}")]
//     VeritechEncryptionKey(#[from] si_crypto::VeritechEncryptionKeyError),
// }

#[derive(Debug)]
pub struct ServerError(anyhow::Error);

// impl From<PgPoolError> for ServerError {
//     fn from(e: PgPoolError) -> Self {
//         Self::PgPool(Box::new(e))
//     }
// }

impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        error!(si.error.message = ?self.0, "failed to process message");
        Response::default_internal_server_error()
    }
}
