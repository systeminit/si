use si_data_pg::{PgError, PgPoolError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LayerCacheError {
    #[error("pg error: {0}]")]
    Pg(#[from] PgError),
    #[error("pg pool error: {0}]")]
    PgPool(#[from] PgPoolError),
    #[error("postcard error: {0}")]
    Postcard(#[from] postcard::Error),
    #[error("sled error: {0}")]
    SledError(#[from] sled::Error),
}

pub type LayerCacheResult<T> = Result<T, LayerCacheError>;
