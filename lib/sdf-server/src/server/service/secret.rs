use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use dal::{
    KeyPairError, StandardModelError, TransactionsError, UserError, WorkspacePk, WsEventError,
};
use thiserror::Error;

use crate::server::state::AppState;

pub mod create_secret;
pub mod get_public_key;
pub mod list_secrets;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SecretError {
    #[error(transparent)]
    ContextTransactions(#[from] TransactionsError),
    #[error(transparent)]
    KeyPairError(#[from] KeyPairError),
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    Secret(#[from] dal::SecretError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    User(#[from] UserError),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(WorkspacePk),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SecretResult<T> = std::result::Result<T, SecretError>;

impl IntoResponse for SecretError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());
        //SecretError::SecretNotFound => (StatusCode::NOT_FOUND, self.to_string()),

        let body = Json(serde_json::json!({
            "error": {
                "message": error_message,
                "code": 42,
                "statusCode": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/get_public_key", get(get_public_key::get_public_key))
        .route("/create_secret", post(create_secret::create_secret))
        .route("/list_secrets", get(list_secrets::list_secrets))
}
