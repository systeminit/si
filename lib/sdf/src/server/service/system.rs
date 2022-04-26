use axum::{
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::{StandardModelError, SystemError as DalSystemError, TransactionsError};
use hyper::StatusCode;
use thiserror::Error;

pub mod create_system;
pub mod get_system;
pub mod list_systems;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    System(#[from] DalSystemError),
    #[error("system not found")]
    SystemNotFound,
    #[error(transparent)]
    Transaction(#[from] TransactionsError),
}

pub type SystemResult<T> = std::result::Result<T, SystemError>;

impl IntoResponse for SystemError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            SystemError::SystemNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

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

pub fn routes() -> Router {
    Router::new()
        .route("/create_system", post(create_system::create_system))
        .route("/get_system", get(get_system::get_system))
        .route("/list_systems", get(list_systems::list_systems))
}
