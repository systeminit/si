use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::{pkg::PkgError, FuncError};
use dal::{
    schema::variant::definition::{
        SchemaVariantDefinitionError as DalSchemaVariantDefinitionError, SchemaVariantDefinitionId,
    },
    SchemaVariantError, StandardModelError, TenancyError, TransactionsError, WsEventError,
};
use si_pkg::{SiPkgError, SpecError};
use thiserror::Error;

use crate::server::state::AppState;
use crate::service::func::FuncError as SdfFuncError;

pub mod clone_variant_def;
pub mod create_variant_def;
pub mod exec_variant_def;
pub mod get_variant_def;
pub mod list_variant_defs;
pub mod save_variant_def;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaVariantDefinitionError {
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("error creating schema variant from definition: {0}")]
    CouldNotCreateSchemaVariantFromDefinition(String),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error(transparent)]
    Pkg(#[from] PkgError),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error(transparent)]
    SchemaVariantDefinition(#[from] DalSchemaVariantDefinitionError),
    #[error(transparent)]
    SdfFunc(#[from] SdfFuncError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SiPkg(#[from] SiPkgError),
    #[error(transparent)]
    Spec(#[from] SpecError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("tenancy error: {0}")]
    Tenancy(#[from] TenancyError),
    #[error("Schema Variant Definition {0} not found")]
    VariantDefinitionNotFound(SchemaVariantDefinitionId),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SchemaVariantDefinitionResult<T> = Result<T, SchemaVariantDefinitionError>;

impl IntoResponse for SchemaVariantDefinitionError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/list_variant_defs",
            get(list_variant_defs::list_variant_defs),
        )
        .route("/get_variant_def", get(get_variant_def::get_variant_def))
        .route(
            "/save_variant_def",
            post(save_variant_def::save_variant_def),
        )
        .route(
            "/create_variant_def",
            post(create_variant_def::create_variant_def),
        )
        .route(
            "/exec_variant_def",
            post(exec_variant_def::exec_variant_def),
        )
        .route(
            "/clone_variant_def",
            post(clone_variant_def::create_variant_def),
        )
}
