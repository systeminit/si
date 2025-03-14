use axum::{
    extract::{Host, OriginalUri, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::{
    diagram::view::ViewId,
    func::authoring::FuncAuthoringError,
    management::{
        prototype::{ManagementPrototype, ManagementPrototypeError, ManagementPrototypeId},
        ManagementError, ManagementFuncReturn, ManagementOperator,
    },
    schema::variant::authoring::VariantAuthoringError,
    ChangeSet, ChangeSetError, ChangeSetId, ComponentId, Func, FuncError, FuncId,
    SchemaVariantError, TransactionsError, WorkspacePk, WsEvent, WsEventError,
};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::ManagementFuncStatus;

use crate::{
    extract::{HandlerContext, PosthogClient},
    service::{force_change_set_response::ForceChangeSetResponse, v2::AccessBuilder, ApiError},
    track, AppState,
};

use super::func::FuncAPIError;

mod generate_template;
mod history;
mod latest;

pub type ManagementApiResult<T> = Result<T, ManagementApiError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ManagementApiError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func api error: {0}")]
    FuncAPI(#[from] FuncAPIError),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] FuncAuthoringError),
    #[error("generated mgmt func {0} has no prototype")]
    FuncMissingPrototype(FuncId),
    #[error("hyper error: {0}")]
    Http(#[from] axum::http::Error),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("management error: {0}")]
    Management(#[from] ManagementError),
    #[error("management history missing a field - this is a bug!: {0}")]
    ManagementHistoryFieldMissing(String),
    #[error("management prototype error: {0}")]
    ManagementPrototype(#[from] ManagementPrototypeError),
    #[error("management prototype execution failure: {0}")]
    ManagementPrototypeExecutionFailure(ManagementPrototypeId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("variant authoring error: {0}")]
    VariantAuthoring(#[from] VariantAuthoringError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

impl IntoResponse for ManagementApiError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            ManagementApiError::ManagementPrototype(
                dal::management::prototype::ManagementPrototypeError::FuncExecutionFailure(message),
            ) => (StatusCode::BAD_REQUEST, message),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        ApiError::new(status_code, error_message).into_response()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunPrototypeRequest {
    request_ulid: Option<ulid::Ulid>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunPrototypeResponse {
    status: ManagementFuncStatus,
    message: Option<String>,
}

pub async fn run_prototype(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, prototype_id, component_id, view_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ManagementPrototypeId,
        ComponentId,
        ViewId,
    )>,
    Json(request): Json<RunPrototypeRequest>,
) -> ManagementApiResult<ForceChangeSetResponse<RunPrototypeResponse>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // TODO check that this is a valid prototypeId
    let mut execution_result =
        ManagementPrototype::execute_by_id(&ctx, prototype_id, component_id, view_id.into())
            .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "run_prototype",
        serde_json::json!({
            "how": "/management/run_prototype",
            "view_id": view_id,
            "prototype_id": prototype_id,
            "component_id": component_id,
        }),
    );

    if let Some(result) = execution_result.result.take() {
        let result: ManagementFuncReturn = result.try_into()?;
        let mut created_component_ids = None;
        if result.status == ManagementFuncStatus::Ok {
            if let Some(operations) = result.operations {
                created_component_ids = ManagementOperator::new(
                    &ctx,
                    component_id,
                    operations,
                    execution_result,
                    Some(view_id),
                )
                .await?
                .operate()
                .await?;
            }
        }

        let func_id = ManagementPrototype::func_id(&ctx, prototype_id).await?;
        let func = Func::get_by_id(&ctx, func_id).await?;

        WsEvent::management_operations_complete(
            &ctx,
            request.request_ulid,
            func.name.clone(),
            result.message.clone(),
            result.status,
            created_component_ids,
        )
        .await?
        .publish_on_commit(&ctx)
        .await?;

        ctx.write_audit_log(
            AuditLogKind::ManagementOperationsComplete {
                component_id,
                prototype_id,
                func_id,
                func_name: func.name.clone(),
                status: match result.status {
                    ManagementFuncStatus::Ok => "ok",
                    ManagementFuncStatus::Error => "error",
                }
                .to_string(),
                message: result.message.clone(),
            },
            func.name,
        )
        .await?;

        ctx.commit().await?;

        return Ok(ForceChangeSetResponse::new(
            force_change_set_id,
            RunPrototypeResponse {
                status: result.status,
                message: result.message,
            },
        ));
    }

    Err(ManagementApiError::ManagementPrototypeExecutionFailure(
        prototype_id,
    ))
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/prototype/:prototypeId/:componentId/:viewId",
            post(run_prototype),
        )
        .route(
            "/prototype/:prototypeId/:componentId/latest",
            get(latest::latest),
        )
        .route("/history", get(history::history))
        .route(
            "/generate_template/:viewId",
            post(generate_template::generate_template),
        )
}
