use axum::body::{Bytes, Full};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use dal::{
    ComponentError as DalComponentError, ReadTenancyError, SchemaError, StandardModelError,
    WsEventError,
};
use std::convert::Infallible;
use thiserror::Error;

pub mod create_application;
pub mod get_application;
pub mod list_applications;

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
    #[error("schema not found")]
    SchemaNotFound,
    #[error("invalid request")]
    InvalidRequest,
    #[error("schema error: {0}")]
    SchemaError(#[from] SchemaError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
}

pub type ApplicationResult<T> = std::result::Result<T, ApplicationError>;

impl IntoResponse for ApplicationError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
        let (status, error_message) = match self {
            ApplicationError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApplicationError::SchemaNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router {
    Router::new()
        .route(
            "/create_application",
            post(create_application::create_application),
        )
        .route(
            "/list_applications",
            get(list_applications::list_applications),
        )
        .route("/get_application", get(get_application::get_application))
}
