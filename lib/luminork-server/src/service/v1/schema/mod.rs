use axum::{
    Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use thiserror::Error;

use crate::AppState;

pub mod list_schema;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("cached module error: {0}")]
    CachedModule(#[from] dal::cached_module::CachedModuleError),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("validation error: {0}")]
    Validation(String),
}

impl IntoResponse for SchemaError {
    fn into_response(self) -> axum::response::Response {
        use crate::service::v1::common::ErrorIntoResponse;
        self.to_api_response()
    }
}

impl crate::service::v1::common::ErrorIntoResponse for SchemaError {
    fn status_and_message(&self) -> (StatusCode, String) {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
    }
}

impl From<JsonRejection> for SchemaError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::JsonDataError(_) => {
                SchemaError::Validation(format!("Invalid JSON data format: {}", rejection))
            }
            JsonRejection::JsonSyntaxError(_) => {
                SchemaError::Validation(format!("Invalid JSON syntax: {}", rejection))
            }
            JsonRejection::MissingJsonContentType(_) => SchemaError::Validation(
                "Request must have Content-Type: application/json header".to_string(),
            ),
            _ => SchemaError::Validation(format!("JSON validation error: {}", rejection)),
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(list_schema::list_schemas))
}
