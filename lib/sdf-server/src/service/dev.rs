mod get_current_git_sha;

use axum::{routing::get, Router};
use dal::{StandardModelError, TransactionsError, UserError, WsEventError};
use thiserror::Error;

use super::impl_default_error_into_response;
use crate::AppState;

#[remain::sorted]
#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
pub enum DevError {
    #[error(transparent)]
    Builtin(#[from] dal::BuiltinsError),
    #[error(transparent)]
    Func(#[from] dal::FuncError),
    #[error("Function not found")]
    FuncNotFound,
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("user error: {0}")]
    User(#[from] UserError),
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
