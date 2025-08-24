use axum::{
    Router,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::post,
};
use dal::{
    ChangeSetError,
    FuncError,
    FuncId,
    SchemaError,
    SchemaId,
    SchemaVariantId,
    TransactionsError,
    WsEventError,
    pkg::PkgError,
    schema::variant::authoring::VariantAuthoringError,
};
use sdf_core::api_error::ApiError;
use si_pkg::{
    SiPkgError,
    SpecError,
};
use telemetry::prelude::*;
use thiserror::Error;

pub mod clone_variant;
pub mod create_variant;
pub mod regenerate_variant;
pub mod save_variant;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaVariantError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("trying to create unlocked copy for schema variant that's not the default: {0}")]
    CreatingUnlockedCopyForNonDefault(SchemaVariantId),
    #[error("dal schema variant error: {0}")]
    DalSchemaVariant(#[from] dal::schema::variant::SchemaVariantError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func execution error: {0}")]
    FuncExecution(FuncId),
    #[error("func execution failure error: {0}")]
    FuncExecutionFailure(String),
    #[error("func is empty: {0}")]
    FuncIsEmpty(FuncId),
    #[error("func not found: {0}")]
    FuncNotFound(FuncId),
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error("no new asset was created")]
    NoAssetCreated,
    #[error("no default schema variant found for schema: {0}")]
    NoDefaultSchemaVariantFoundForSchema(SchemaId),
    #[error("pkg error: {0}")]
    Pkg(#[from] PkgError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("Schema name {0} already taken")]
    SchemaNameAlreadyTaken(String),
    #[error("schema variant asset func not found: {0}")]
    SchemaVariantAssetNotFound(SchemaVariantId),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si pkg error: {0}")]
    SiPkg(#[from] SiPkgError),
    #[error("spec error: {0}")]
    Spec(#[from] SpecError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("schema variant already is unlocked: {0}")]
    VariantAlreadyUnlocked(SchemaVariantId),
    #[error("variant authoring: {0}")]
    VariantAuthoring(#[from] VariantAuthoringError),
    #[error("schema variant not found: {0}")]
    VariantNotFound(SchemaVariantId),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SchemaVariantResult<T> = Result<T, SchemaVariantError>;

impl IntoResponse for SchemaVariantError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            SchemaVariantError::FuncNotFound(_)
            | SchemaVariantError::NoDefaultSchemaVariantFoundForSchema(_)
            | SchemaVariantError::SchemaVariantAssetNotFound(_)
            | SchemaVariantError::VariantNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            SchemaVariantError::VariantAuthoring(VariantAuthoringError::DuplicatedSchemaName(
                _,
            )) => (StatusCode::CONFLICT, self.to_string()),
            SchemaVariantError::VariantAuthoring(
                VariantAuthoringError::AssetTypeNotReturnedForAssetFunc(_, _),
            ) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "unexpected return type, expected 'Asset' return type".to_string(),
            ),
            SchemaVariantError::VariantAuthoring(VariantAuthoringError::FuncExecutionFailure(
                message,
            )) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            SchemaVariantError::SchemaNameAlreadyTaken(name) => (
                StatusCode::CONFLICT,
                format!("Schema name {name} already in use"),
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        ApiError::new(status_code, error_message).into_response()
    }
}

pub fn routes() -> Router<sdf_core::app_state::AppState> {
    Router::new()
        .route("/create_variant", post(create_variant::create_variant)) // USED IN CUSTOMIZE UI AND API TESTS
        .route(
            "/regenerate_variant", // USED IN CUSTOMIZE UI
            post(regenerate_variant::regenerate_variant),
        )
        .route("/clone_variant", post(clone_variant::clone_variant)) // USED IN CUSTOMIZE UI
        .route("/save_variant", post(save_variant::save_variant)) // USED IN CUSTOMIZE UI
}
