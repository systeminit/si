use thiserror::Error;

#[derive(Error, Debug)]
pub enum LayerCacheError {
    #[error("postcard error: {0}")]
    Postcard(#[from] postcard::Error),
    #[error("sled error: {0}")]
    SledError(#[from] sled::Error),
}

pub type LayerCacheResult<T> = Result<T, LayerCacheError>;
