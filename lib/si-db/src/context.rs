use async_trait::async_trait;
use si_data_nats::NatsTxn;
use si_data_pg::{
    PgError,
    PgTxn,
};
use thiserror::Error;
use tokio::sync::MappedMutexGuard;

use crate::{
    history_event::HistoryActor,
    tenancy::Tenancy,
    visibility::Visibility,
};

#[async_trait]
pub trait SiDbContext {
    type Transactions: SiDbTransactions;
    fn history_actor(&self) -> &HistoryActor;
    async fn txns(&self)
    -> Result<MappedMutexGuard<'_, Self::Transactions>, BaseTransactionsError>;
    fn tenancy(&self) -> &Tenancy;
    fn visibility(&self) -> &Visibility;
}

pub trait SiDbTransactions {
    fn pg(&self) -> &PgTxn;
    fn nats(&self) -> &NatsTxn;
}

// TODO TransactionsError really needs to be moved to a common place accessible here and in dal
#[remain::sorted]
#[derive(Debug, Error, strum::EnumDiscriminants)]
pub enum BaseTransactionsError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("cannot start transactions without connections; state={0}")]
    TxnStart(&'static str),
}
