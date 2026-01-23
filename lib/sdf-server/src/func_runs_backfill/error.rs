use dal::TransactionsError;
use si_data_pg::{
    PgError,
    PgPoolError,
};
use si_db::SiDbError;
use si_layer_cache::LayerDbError;
use thiserror::Error;
use tokio::task::JoinError;

use crate::init::InitError;

pub type FuncRunsBackfillResult<T> = Result<T, FuncRunsBackfillError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum FuncRunsBackfillError {
    #[error("dal initialization error: {0}")]
    DalInit(#[from] crate::ServerError),
    #[error("init error: {0}")]
    Init(#[from] InitError),
    #[error("tokio join error: {0}")]
    Join(#[from] JoinError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("postgres error: {0}")]
    Pg(#[from] PgError),
    #[error("postgres pool error: {0}")]
    PgPool(#[from] PgPoolError),
    #[error("si-db error: {0}")]
    SiDb(#[from] SiDbError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}
