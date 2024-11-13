use crate::{
    service::{force_change_set_response::ForceChangeSetResponse, ApiError},
    AppState,
};
use axum::{
    extract::{Host, OriginalUri, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use dal::{
    management::{
        prototype::{ManagementPrototype, ManagementPrototypeError, ManagementPrototypeId},
        ManagementError, ManagementFuncReturn, ManagementOperator,
    },
    ChangeSet, ChangeSetError, ChangeSetId, ComponentId, TransactionsError, WorkspacePk,
};
use serde::{Deserialize, Serialize};
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::ManagementFuncStatus;

use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

use super::func::FuncAPIError;

mod history;
mod latest;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunPrototypeResponse {
    status: ManagementFuncStatus,
    message: Option<String>,
}

pub type ManagementApiResult<T> = Result<T, ManagementApiError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ManagementApiError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("func api error: {0}")]
    FuncAPI(#[from] FuncAPIError),
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
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

impl IntoResponse for ManagementApiError {
    fn into_response(self) -> Response {
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

pub async fn run_prototype(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, prototype_id, component_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ManagementPrototypeId,
        ComponentId,
    )>,
) -> ManagementApiResult<ForceChangeSetResponse<RunPrototypeResponse>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // TODO check that this is a valid prototypeId
    let mut execution_result =
        ManagementPrototype::execute_by_id(&ctx, prototype_id, component_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "run_prototype",
        serde_json::json!({
            "how": "/management/run_prototype",
            "prototype_id": prototype_id.clone(),
            "component_id": component_id.clone(),
        }),
    );

    if let Some(result) = execution_result.result.take() {
        let result: ManagementFuncReturn = result.try_into()?;
        if result.status == ManagementFuncStatus::Ok {
            if let Some(operations) = result.operations {
                ManagementOperator::new(&ctx, component_id, operations, execution_result, None)
                    .await?
                    .operate()
                    .await?;
            }
        }

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
        .route("/prototype/:prototypeId/:componentId", post(run_prototype))
        .route(
            "/prototype/:prototypeId/:componentId/latest",
            get(latest::latest),
        )
        .route("/history", get(history::history))
}
