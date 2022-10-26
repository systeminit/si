use std::string::FromUtf8Error;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};

use thiserror::Error;

use dal::{qualification::QualificationSummaryError, WsEventError};
use dal::{
    AttributeValueError, ComponentError, ComponentId, FuncError, FuncId,
    QualificationPrototypeError, QualificationPrototypeId, ReadTenancyError, SchemaError, SchemaId,
    StandardModelError, TransactionsError, WriteTenancyError,
};

pub mod get_summary;

// code endpoints here are deprecated, removing them from the module tree
// moved to the func service - this probably means we can pair down the
// QualificationError a bit
//pub mod create;
//pub mod get_code;
//pub mod set_code;

#[derive(Debug, Error)]
pub enum QualificationError {
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("utf8 error: {0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("entity error: {0}")]
    QualificationPrototype(#[from] QualificationPrototypeError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("write tenancy error: {0}")]
    WriteTenancy(#[from] WriteTenancyError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("func not found")]
    FuncNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("func code not found: {0}")]
    FuncCodeNotFound(FuncId),
    #[error("component not found: {0}")]
    ComponentNotFound(ComponentId),
    #[error("schema not found: {0}")]
    SchemaNotFound(SchemaId),
    #[error("qualification prototype not found: {0}")]
    PrototypeNotFound(QualificationPrototypeId),
    #[error("not writable")]
    NotWritable,
    #[error("qualification summary error: {0}")]
    QualificationSummaryError(#[from] QualificationSummaryError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type QualificationResult<T> = std::result::Result<T, QualificationError>;

impl IntoResponse for QualificationError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router {
    Router::new().route("/get_summary", get(get_summary::get_summary))
}
