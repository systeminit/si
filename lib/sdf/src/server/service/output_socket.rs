use axum::{
    body::{Bytes, Full},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use dal::socket::output::OutputSocketError as DalOutputSocketError;
use dal::{StandardModelError, TransactionsError};
use std::convert::Infallible;
use thiserror::Error;

pub mod list_output_sockets;

#[derive(Debug, Error)]
pub enum OutputSocketError {
    #[error(transparent)]
    ContextError(#[from] TransactionsError),
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data::PgPoolError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),

    #[error("output socket error: {0}")]
    OutputSocket(#[from] DalOutputSocketError),
}

pub type OutputSocketResult<T> = std::result::Result<T, OutputSocketError>;

impl IntoResponse for OutputSocketError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
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
        "/list_output_sockets",
        get(list_output_sockets::list_output_sockets),
    )
}
