pub(crate) mod local;
pub(crate) mod pg;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use si_data_pg::{PgError, PgPoolError};

use thiserror::Error;

use crate::hash::ContentHash;
use crate::pair::ContentPairError;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("content pair error: {0}")]
    ContentPair(#[from] ContentPairError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, Clone)]
struct StoreItem {
    value: Value,
    written: bool,
}

#[async_trait::async_trait]
pub trait Store {
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn add<T>(&mut self, value: T) -> StoreResult<(ContentHash, bool)>
    where
        T: Serialize + ToOwned;
    async fn get<T>(&self, key: &ContentHash) -> StoreResult<Option<T>>
    where
        T: DeserializeOwned;
    async fn write(&mut self) -> StoreResult<()>;
}
