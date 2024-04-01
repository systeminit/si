use axum::routing::{get, patch};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use dal::{
    ChangeSetError, KeyPairError, SecretId, StandardModelError, TransactionsError, UserError,
    WorkspacePk, WsEventError,
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
    #[error("context transactions error: {0}")]
    ContextTransactions(#[from] TransactionsError),
    #[error("dal secret error: {0}")]
    DalSecret(#[from] dal::SecretError),
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error("key pair error: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("nats error: {0}")]
    Nats(#[from] si_data_nats::NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("secret definition view error: {0}")]
    SecretDefinitionView(#[from] dal::SecretDefinitionViewError),
    #[error("secret view error: {0}")]
    SecretView(#[from] dal::SecretViewError),
    #[error("definition not found for secret: {0}")]
    SecretWithInvalidDefinition(SecretId),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("user error: {0}")]
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
