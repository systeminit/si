use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use dal::{
    ChangeSetError, DiagramError, KeyPairError, SecretId, StandardModelError, TransactionsError,
    UserError, WorkspacePk, WsEventError,
};
use thiserror::Error;

use crate::server::state::AppState;

pub mod create_secret;
pub mod get_public_key;
pub mod list_secrets;
pub mod update_secret;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SecretError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error(transparent)]
    ContextTransactions(#[from] TransactionsError),
    #[error(transparent)]
    Diagram(#[from] DiagramError),
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error(transparent)]
    KeyPairError(#[from] KeyPairError),
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    Secret(#[from] dal::SecretError),
    #[error("definition not found for secret: {0}")]
    SecretWithInvalidDefinition(SecretId),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    User(#[from] UserError),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(WorkspacePk),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SecretResult<T> = Result<T, SecretError>;

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
        .route("/", post(create_secret::create_secret))
        .route("/", get(list_secrets::list_secrets))
        .route("/", patch(update_secret::update_secret))
}
