use si_data_pg::PgError;
use si_layer_cache::LayerDbError;
use thiserror::Error;

use crate::init::InitError;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum BackfillError {
    #[error("cache type not configured for backfill: {cache_type}")]
    CacheTypeNotBackfillable { cache_type: String },
    #[error("failed to initialize services context: {0}")]
    Init(#[from] InitError),
    #[error("invalid timestamp format: {timestamp}")]
    InvalidTimestampFormat { timestamp: String },
    #[error("layer cache error: {0}")]
    LayerCache(#[from] LayerDbError),
    #[error("missing required cutoff timestamp")]
    MissingCutoffTimestamp,
    #[error("no cache types specified for backfill")]
    NoCacheTypesSpecified,
    #[error("PostgreSQL error: {0}")]
    Pg(#[from] PgError),
    #[error("tokio join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
}

pub type BackfillResult<T> = std::result::Result<T, BackfillError>;
