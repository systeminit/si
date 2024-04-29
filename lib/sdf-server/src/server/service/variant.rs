use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{routing::get, Json, Router};
use dal::func::binding::FuncBindingError;
use dal::func::summary::FuncSummaryError;
use dal::pkg::PkgError;
use dal::schema::variant::authoring::VariantAuthoringError;
use dal::{
    ChangeSetError, FuncError, FuncId, SchemaError, SchemaVariantId, TransactionsError,
    WsEventError,
};
use si_pkg::{SiPkgError, SpecError};
use thiserror::Error;

use crate::server::state::AppState;

pub mod clone_variant;
pub mod create_variant;
pub mod get_variant;
pub mod list_variants;
pub mod update_variant;
// pub mod save_variant_def;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaVariantError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("dal schema variant error: {0}")]
    DalSchemaVariant(#[from] dal::schema::variant::SchemaVariantError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func execution error: {0}")]
    FuncExecution(FuncId),
    #[error("func execution failure error: {0}")]
    FuncExecutionFailure(String),
    #[error("func is empty: {0}")]
    FuncIsEmpty(FuncId),
    #[error("func not found: {0}")]
    FuncNotFound(FuncId),
    #[error("func summary error: {0}")]
    FuncSummary(#[from] FuncSummaryError),
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error("no new asset was created")]
    NoAssetCreated,
    #[error("no default schema variant found for schema")]
    NoDefaultSchemaVariantFoundForSchema,
    #[error("pkg error: {0}")]
    Pkg(#[from] PkgError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
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
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list_variants", get(list_variants::list_variants))
        .route("/get_variant", get(get_variant::get_variant))
        // .route(
        //     "/save_variant_def",
        //     post(save_variant_def::save_variant_def),
        // )
        .route("/create_variant", post(create_variant::create_variant))
        .route("/update_variant", post(update_variant::update_variant))
        .route("/clone_variant", post(clone_variant::clone_variant))
}
