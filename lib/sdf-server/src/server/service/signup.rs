use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use thiserror::Error;

use dal::{
    ComponentError, NodeError, SchemaError, StandardModelError, TransactionsError, WorkspaceError,
};

pub mod create_account;

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum SignupError {
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("invalid signup secret")]
    InvalidSignupSecret,
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error("Node error: {0}")]
    Node(#[from] NodeError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error("Schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("StandardModel error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
}

pub type SignupResult<T> = std::result::Result<T, SignupError>;

impl IntoResponse for SignupError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            SignupError::InvalidSignupSecret => {
                (StatusCode::BAD_REQUEST, "signup failed".to_string())
            }
            err => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

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

pub fn routes() -> Router {
    Router::new().route("/create_account", post(create_account::create_account))
}
