use axum::{
    body::{Bytes, Full},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use dal::socket::input::InputSocketError as DalInputSocketError;
use dal::{StandardModelError, TransactionsError};
use std::convert::Infallible;
use thiserror::Error;

pub mod list_input_sockets;

#[derive(Debug, Error)]
pub enum InputSocketError {
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

    #[error("input socket error: {0}")]
    InputSocket(#[from] DalInputSocketError),
}

pub type InputSocketResult<T> = std::result::Result<T, InputSocketError>;

impl IntoResponse for InputSocketError {
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
        "/list_input_sockets",
        get(list_input_sockets::list_input_sockets),
    )
}
