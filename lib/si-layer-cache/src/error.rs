use thiserror::Error;

#[derive(Error, Debug)]
pub enum LayerCacheError {
    #[error(transparent)]
    SledError(#[from] sled::Error),
}

pub type LayerCacheResult<T> = Result<T, LayerCacheError>;
