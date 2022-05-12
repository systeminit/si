use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::{
    edit_field::EditFieldObjectKind, schema::variant::SchemaVariantError, socket::SocketError,
    ComponentError, PropError, QualificationCheckError, ReadTenancyError, SchemaError,
    StandardModelError, TransactionsError,
};
use thiserror::Error;

pub mod get_edit_fields;
pub mod insert_from_edit_field;
pub mod remove_from_edit_field;
pub mod update_from_edit_field;

#[derive(Debug, Error)]
pub enum EditFieldError {
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error(transparent)]
    ContextError(#[from] TransactionsError),
    #[error("Invalid object kind: {0}")]
    InvalidObjectKind(EditFieldObjectKind),
    #[error("invalid visibility")]
    InvalidVisibility,
    #[error("missing required attribute context for edit field request; bug")]
    MissingAttributeContext,
    #[error("missing required baggage for edit field request; bug")]
    MissingBaggage,
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data::PgPoolError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("qualification check error: {0}")]
    QualificationChec(#[from] QualificationCheckError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
}

pub type EditFieldResult<T> = std::result::Result<T, EditFieldError>;

impl IntoResponse for EditFieldError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            EditFieldError::InvalidVisibility => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
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
    Router::new()
        .route("/get_edit_fields", get(get_edit_fields::get_edit_fields))
        .route(
            "/insert_from_edit_field",
            post(insert_from_edit_field::insert_from_edit_field),
        )
        .route(
            "/remove_from_edit_field",
            post(remove_from_edit_field::remove_from_edit_field),
        )
        .route(
            "/update_from_edit_field",
            post(update_from_edit_field::update_from_edit_field),
        )
}
