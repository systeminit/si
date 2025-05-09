use axum::{
    Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use dal::{
    FuncId,
    SchemaId,
    SchemaVariantId,
};
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;
use utoipa::ToSchema;

use crate::AppState;

pub mod find_schema;
pub mod get_default_variant;
pub mod get_schema;
pub mod get_variant;
pub mod list_schema;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("cached module error: {0}")]
    CachedModule(#[from] dal::cached_module::CachedModuleError),
    #[error("decode error: {0}")]
    Decode(#[from] ulid::DecodeError),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema not found by name: {0}")]
    SchemaNameNotFound(String),
    #[error("schema not found error: {0}")]
    SchemaNotFound(SchemaId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("schema variant not found error: {0}")]
    SchemaVariantNotFound(SchemaVariantId),
    #[error("schema variant {0} not a variant for the schema {1} error")]
    SchemaVariantNotMemberOfSchema(SchemaId, SchemaVariantId),
    #[error("validation error: {0}")]
    Validation(String),
}

pub type SchemaResult<T> = Result<T, SchemaError>;

#[derive(Deserialize, ToSchema)]
pub struct SchemaV1RequestPath {
    #[schema(value_type = String)]
    pub schema_id: SchemaId,
}

#[derive(Deserialize, ToSchema)]
pub struct SchemaVariantV1RequestPath {
    #[schema(value_type = String)]
    pub schema_id: SchemaId,
    #[schema(value_type = String)]
    pub schema_variant_id: SchemaVariantId,
}

impl IntoResponse for SchemaError {
    fn into_response(self) -> axum::response::Response {
        use crate::service::v1::common::ErrorIntoResponse;
        self.to_api_response()
    }
}

impl crate::service::v1::common::ErrorIntoResponse for SchemaError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            SchemaError::SchemaNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            SchemaError::SchemaNameNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            SchemaError::SchemaVariantNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            SchemaError::SchemaVariantNotMemberOfSchema(_, _) => {
                (StatusCode::PRECONDITION_REQUIRED, self.to_string())
            }
            SchemaError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
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
    Router::new()
        .route("/", get(list_schema::list_schemas))
        .route("/find", get(find_schema::find_schema))
        .nest(
            "/:schema_id",
            Router::new().route("/", get(get_schema::get_schema)).nest(
                "/variant",
                Router::new()
                    .route("/default", get(get_default_variant::get_default_variant))
                    .nest(
                        "/:schema_variant_id",
                        Router::new().route("/", get(get_variant::get_variant)),
                    ),
            ),
        )
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaVariantV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub variant_id: SchemaVariantId,
    #[schema(value_type = String, example = "AWS EC2 Instance")]
    pub display_name: String,
    #[schema(value_type = String, example = "AWS::EC2")]
    pub category: String,
    #[schema(value_type = String, example = "#FF5733")]
    pub color: String,
    #[schema(value_type = bool, example = false)]
    pub is_locked: bool,
    #[schema(value_type = String, example = "Amazon EC2 Instance resource type")]
    pub description: Option<String>,
    #[schema(value_type = String, example = "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-instance.html")]
    pub link: Option<String>,
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q75XY")]
    pub asset_func_id: FuncId,
    #[schema(value_type = Vec<String>, example = json!(["01H9ZQD35JPMBGHH69BT0Q75AA", "01H9ZQD35JPMBGHH69BT0Q75BB"]))]
    pub variant_func_ids: Vec<FuncId>,
    #[schema(value_type = bool, example = true)]
    pub is_default_variant: bool,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaV1Response {
    #[schema(value_type = String, example = "AWS::EC2::Instance")]
    pub name: String,
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub default_variant_id: SchemaVariantId,
    #[schema(value_type = Vec<String>, example = json!(["01H9ZQD35JPMBGHH69BT0Q79VZ", "01H9ZQD35JPMBGHH69BT0Q79VY"]))]
    pub variant_ids: Vec<SchemaVariantId>,
}
