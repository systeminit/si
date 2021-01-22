use reqwest;
use thiserror::Error;

pub mod event_log_fs;
pub use event_log_fs::{EventLogFS, EventLogFSError};
pub mod pg;
pub use pg::{PgError, PgPool, PgTxn};
pub mod nats;
pub use self::nats::{NatsConn, NatsTxn, NatsTxnError};

lazy_static::lazy_static! {
    pub static ref REQWEST: reqwest::Client = reqwest::Client::new();
}

#[derive(Error, Debug)]
pub enum DataError {}

pub type DataResult<T> = Result<T, DataError>;
