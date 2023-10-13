use serde::de::DeserializeOwned;
use serde::Serialize;
use si_cbor::CborError;
use si_data_pg::{PgError, PgPoolError};
use thiserror::Error;

use crate::hash::ContentHash;
use crate::pair::ContentPairError;

pub(crate) mod local;
pub(crate) mod pg;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("cbor error: {0}")]
    Cbor(#[from] CborError),
    #[error("content pair error: {0}")]
    ContentPair(#[from] ContentPairError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

type StoreResult<T> = Result<T, StoreError>;

/// This trait provides the minimum methods needed to create a content store.
#[async_trait::async_trait]
pub trait Store {
    /// Indicates whether or not the store is empty.
    fn is_empty(&self) -> bool;

    /// Indicates the number of keys in the store.
    fn len(&self) -> usize;

    /// Adds an item to the store.
    fn add<T>(&mut self, object: &T) -> StoreResult<ContentHash>
    where
        T: Serialize + ?Sized;

    /// Gets an item from the store.
    ///
    /// Implementers of this trait may want to consider a "pull-through cache" implementation for
    /// this method.
    async fn get<T>(&mut self, key: &ContentHash) -> StoreResult<Option<T>>
    where
        T: DeserializeOwned;

    /// Writes out content in the store to a persistent storage layer, if applicable.
    async fn write(&mut self) -> StoreResult<()>;
}
