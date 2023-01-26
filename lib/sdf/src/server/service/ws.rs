use axum::{
    http::StatusCode, response::IntoResponse, response::Response, routing::get, Json, Router,
};
use dal::TransactionsError;
use si_data_pg::{PgError, PgPoolError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WsError {
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
}

pub mod billing_account_updates;

impl IntoResponse for WsError {
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

pub fn routes() -> Router {
    Router::new().route(
        "/billing_account_updates",
        get(billing_account_updates::billing_account_updates),
    )
}
