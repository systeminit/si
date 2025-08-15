use si_data_nats::NatsTxn;
use si_data_pg::{
    PgError,
    PgTxn,
};
use thiserror::Error;

pub trait SiDbTransactions {
    fn pg(&self) -> &PgTxn;
    fn nats(&self) -> &NatsTxn;
}

// TODO TransactionsError really needs to be moved to a common place accessible here and in dal
#[remain::sorted]
#[derive(Debug, Error, strum::EnumDiscriminants)]
pub enum SiDbTransactionsError {
    #[error("cannot use transactions when connection state invalid")]
    ConnStateInvalid,
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("cannot start transactions without connections; state={0}")]
    TxnStart(&'static str),
}
