use axum::body::{Bytes, Full};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{post};
use axum::Json;
use axum::Router;
use dal::{ComponentError as DalComponentError, NodeError, StandardModelError};
use std::convert::Infallible;
use thiserror::Error;

pub mod create_application;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("entity error: {0}")]
    Component(#[from] DalComponentError),
    #[error("not found")]
    NotFound,
    #[error("invalid request")]
    InvalidRequest,
    #[error("node error")]
    Node(#[from] NodeError),
}

pub type ApplicationResult<T> = std::result::Result<T, ApplicationError>;

impl IntoResponse for ApplicationError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
        let (status, error_message) = match self {
            ApplicationError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router {
    Router::new().route(
        "/create_application",
        post(create_application::create_application),
    )
}
