mod get_current_git_sha;

use axum::{
    Router,
    routing::get,
};
use dal::{
    TransactionsError,
    WsEventError,
};
use thiserror::Error;

use super::impl_default_error_into_response;
use crate::AppState;

#[remain::sorted]
#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
pub enum DevError {
    #[error("builtin error: {0}")]
    Builtin(#[from] dal::BuiltinsError),
    #[error("func error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("Function not found")]
    FuncNotFound,
    #[error("nats error: {0}")]
    Nats(#[from] si_data_nats::NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type DevResult<T> = Result<T, DevError>;

impl_default_error_into_response!(DevError);

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/get_current_git_sha",
        get(get_current_git_sha::get_current_git_sha),
    )
}
