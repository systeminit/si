use std::string::FromUtf8Error;

use axum::{
    Router,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::get,
};
use dal::{
    ComponentError,
    ComponentId,
    FuncId,
    SchemaError,
    SchemaId,
    StandardModelError,
    TenancyError,
    TransactionsError,
    WsEventError,
    qualification::QualificationSummaryError,
};
use sdf_core::{
    api_error::ApiError,
    app_state::AppState,
};
use telemetry::prelude::*;
use thiserror::Error;

pub mod get_summary;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum QualificationError {
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component not found: {0}")]
    ComponentNotFound(ComponentId),
    #[error("func code not found: {0}")]
    FuncCodeNotFound(FuncId),
    #[error("func not found")]
    FuncNotFound,
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error("qualification summary error: {0}")]
    QualificationSummaryError(#[from] QualificationSummaryError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema not found: {0}")]
    SchemaNotFound(SchemaId),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("tenancy error: {0}")]
    Tenancy(#[from] TenancyError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("utf8 error: {0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type QualificationResult<T> = std::result::Result<T, QualificationError>;

impl IntoResponse for QualificationError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            QualificationError::ComponentNotFound(_)
            | QualificationError::FuncCodeNotFound(_)
            | QualificationError::FuncNotFound
            | QualificationError::SchemaNotFound(_)
            | QualificationError::SchemaVariantNotFound => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        ApiError::new(status_code, error_message).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/get_summary", get(get_summary::get_summary))
}
