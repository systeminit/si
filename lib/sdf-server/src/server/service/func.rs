use axum::{
    response::Response,
    routing::{get, post},
    Json, Router,
};
use dal::func::argument::FuncArgumentError;
use dal::func::authoring::FuncAuthoringError;
use dal::func::summary::FuncSummaryError;
use dal::func::view::FuncViewError;
use dal::schema::variant::SchemaVariantError;
use dal::{workspace_snapshot::WorkspaceSnapshotError, FuncId, TransactionsError};
use dal::{ChangeSetError, WsEventError};
use thiserror::Error;

use crate::server::{impl_default_error_into_response, state::AppState};

pub mod create_func;
pub mod get_func;
pub mod list_funcs;
pub mod save_func;

// pub mod delete_func;
// pub mod execute;
// pub mod list_input_sources;
// pub mod save_and_exec;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("context transaction error: {0}")]
    ContextTransaction(#[from] TransactionsError),
    #[error("dal func error: {0}")]
    Func(#[from] dal::func::FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] FuncAuthoringError),
    #[error("func {0} cannot be converted to frontend variant")]
    FuncCannotBeTurnedIntoVariant(FuncId),
    #[error("The function name \"{0}\" is reserved")]
    FuncNameReserved(String),
    #[error("func summary error: {0}")]
    FuncSummary(#[from] FuncSummaryError),
    #[error("func view error: {0}")]
    FuncView(#[from] FuncViewError),
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error("func is read-only")]
    NotWritable,
    #[error("prop for value not found")]
    PropNotFound,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type FuncResult<T> = Result<T, FuncError>;

impl_default_error_into_response!(FuncError);

// async fn action_prototypes_into_schema_variants_and_components(
//     ctx: &DalContext,
//     func_id: FuncId,
// ) -> FuncResult<(Option<ActionKind>, Vec<SchemaVariantId>)> {
//     let mut variant_ids = vec![];
//     let mut action_kind: Option<ActionKind> = None;

//     for proto in ActionPrototype::find_for_func(ctx, func_id).await? {
//         if let Some(action_kind) = &action_kind {
//             if action_kind != proto.kind() {
//                 return Err(FuncError::ActionFuncMultipleKinds(func_id));
//             }
//         } else {
//             action_kind = Some(*proto.kind());
//         }

//         if proto.schema_variant_id().is_some() {
//             variant_ids.push(proto.schema_variant_id());
//         }
//     }

//     if !variant_ids.is_empty() && action_kind.is_none() {
//         return Err(FuncError::ActionKindMissing(func_id));
//     }

//     Ok((action_kind, variant_ids))
// }

// async fn attribute_prototypes_into_schema_variants_and_components(
//     ctx: &DalContext,
//     func_id: FuncId,
// ) -> FuncResult<(Vec<SchemaVariantId>, Vec<ComponentId>)> {
//     let schema_variants_components =
//         AttributePrototype::find_for_func_as_variant_and_component(ctx, func_id).await?;

//     let mut schema_variant_ids = vec![];
//     let mut component_ids = vec![];

//     for (schema_variant_id, component_id) in schema_variants_components {
//         if component_id == ComponentId::NONE {
//             schema_variant_ids.push(schema_variant_id);
//         } else {
//             component_ids.push(component_id);
//         }
//     }

//     Ok((schema_variant_ids, component_ids))
// }

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list_funcs", get(list_funcs::list_funcs))
        .route("/get_func", get(get_func::get_func))
        //         .route(
        //             "/get_func_last_execution",
        //             get(get_func::get_latest_func_execution),
        //         )
        .route("/create_func", post(create_func::create_func))
        .route("/save_func", post(save_func::save_func))
    //         .route("/delete_func", post(delete_func::delete_func))
    //         .route("/save_and_exec", post(save_and_exec::save_and_exec))
    //         .route("/execute", post(execute::execute))
    //         .route(
    //             "/list_input_sources",
    //             get(list_input_sources::list_input_sources),
    //         )
}
