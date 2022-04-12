use axum::{
    body::{Bytes, Full},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dal::{
    schema::variant::SchemaVariantError, socket::SocketError, ComponentError, PropError,
    QualificationCheckError, ReadTenancyError, SchemaError, StandardModelError, TransactionsError,
};
use std::convert::Infallible;
use thiserror::Error;

pub mod get_edit_fields;
pub mod remove_from_edit_field;
pub mod update_from_edit_field;

#[derive(Debug, Error)]
pub enum EditFieldError {
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("qualification check error: {0}")]
    QualificationChec(#[from] QualificationCheckError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    ContextError(#[from] TransactionsError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("missing required baggage for edit field request; bug")]
    MissingBaggage,
    #[error("missing required attribute context for edit field request; bug")]
    MissingAttributeContext,
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
}

pub type EditFieldResult<T> = std::result::Result<T, EditFieldError>;

impl IntoResponse for EditFieldError {
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
    Router::new()
        .route("/get_edit_fields", get(get_edit_fields::get_edit_fields))
        .route(
            "/remove_from_edit_field",
            post(remove_from_edit_field::remove_from_edit_field),
        )
        .route(
            "/update_from_edit_field",
            post(update_from_edit_field::update_from_edit_field),
        )
}
