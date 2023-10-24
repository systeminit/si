use axum::response::Response;
use axum::routing::post;
use axum::Json;
use axum::Router;
use dal::{TransactionsError, UserError};
use thiserror::Error;

use crate::server::{impl_default_error_into_response, state::AppState};

pub mod invite;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("no workspace in context")]
    NoWorkspace,
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("user error: {0}")]
    User(#[from] UserError),
}

pub type WorkspaceResult<T> = std::result::Result<T, WorkspaceError>;

impl_default_error_into_response!(WorkspaceError);

pub fn routes() -> Router<AppState> {
    Router::new().route("/invite", post(invite::invite))
}
