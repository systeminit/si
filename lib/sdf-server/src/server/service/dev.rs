mod author_single_schema_with_default_variant;
mod get_current_git_sha;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use dal::{StandardModelError, TransactionsError, UserError, WsEventError};
use thiserror::Error;

pub use author_single_schema_with_default_variant::CREATE_CONFIRMATION_NAME;
pub use author_single_schema_with_default_variant::DELETE_CONFIRMATION_NAME;
pub use author_single_schema_with_default_variant::{
    AuthorSingleSchemaRequest, AuthorSingleSchemaResponse,
};

use crate::server::state::AppState;
use crate::service::dev::author_single_schema_with_default_variant::author_single_schema_with_default_variant;
use crate::service::func;

#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
pub enum DevError {
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error("Function not found")]
    FuncNotFound,
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
    #[error(transparent)]
    Func(#[from] dal::FuncError),
    #[error(transparent)]
    SdfFunc(#[from] func::FuncError),
    #[error(transparent)]
    Builtin(#[from] dal::BuiltinsError),
}

pub type DevResult<T> = Result<T, DevError>;

impl IntoResponse for DevError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(serde_json::json!({
            "error": {
                "message": error_message,
                "code": 42,
                "statusCode": status.as_u16(),
            },
        }));

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/get_current_git_sha",
            get(get_current_git_sha::get_current_git_sha),
        )
        .route(
            "/author_single_schema_with_default_variant",
            post(author_single_schema_with_default_variant),
        )
}
