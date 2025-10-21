use thiserror::Error;

#[derive(Debug, Error)]
pub enum SnapshotEvictionError {
    #[error("database error: {0}")]
    Database(#[from] si_data_pg::PgError),

    #[error("database pool error: {0}")]
    PgPool(#[from] si_data_pg::PgPoolError),

    #[error("layer cache error: {0}")]
    LayerCache(#[from] si_layer_cache::LayerDbError),

    #[error("NATS error: {0}")]
    Nats(#[from] si_data_nats::NatsError),
}

pub type SnapshotEvictionResult<T> = Result<T, SnapshotEvictionError>;
