use axum::response::Response;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use dal::provider::external::ExternalProviderError;
use dal::provider::internal::InternalProviderError;
use dal::{StandardModelError, TransactionsError};

use thiserror::Error;

pub mod list_all_providers;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error(transparent)]
    ContextError(#[from] TransactionsError),
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),

    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
}

pub type ProviderResult<T> = std::result::Result<T, ProviderError>;

impl IntoResponse for ProviderError {
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

pub fn routes() -> Router {
    Router::new().route(
        "/list_all_providers",
        get(list_all_providers::list_all_providers),
    )
}
