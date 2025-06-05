use axum::{
    Json,
    Router,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        get,
        post,
    },
};
use chrono::{
    Duration,
    Utc,
};
use dal::{
    ChangeSet,
    ChangeSetError,
    ChangeSetId,
    ComponentId,
    FuncError,
    FuncId,
    SchemaVariantError,
    TransactionsError,
    WorkspacePk,
    WsEventError,
    diagram::view::ViewId,
    func::authoring::FuncAuthoringError,
    management::{
        ManagementError,
        prototype::{
            ManagementPrototypeError,
            ManagementPrototypeId,
        },
    },
    schema::variant::authoring::VariantAuthoringError,
};
use sdf_core::api_error::ApiError;
use serde::{
    Deserialize,
    Serialize,
};
use si_db::{
    ManagementFuncExecutionError,
    ManagementFuncJobState,
    ManagementState,
};
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::ManagementFuncStatus;

use super::func::FuncAPIError;
use crate::{
    AppState,
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::AccessBuilder,
    },
    track,
};

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
    #[error("management func job status error: {0}")]
    ManagementFuncJobStatus(#[from] ManagementFuncExecutionError),
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

const MAX_PENDING_DURATION: Duration = Duration::minutes(7);

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
    let state_ctx = ctx.clone();

    // Is there already a pending run? if so fail it if it's too old, so that we can start a new one
    if let Some(pending) =
        ManagementFuncJobState::get_pending(&state_ctx, component_id, prototype_id).await?
    {
        if pending.timestamp().updated_at <= Utc::now() - MAX_PENDING_DURATION {
            ManagementFuncJobState::transition_state(
                &state_ctx,
                pending.id(),
                ManagementState::Failure,
                None,
            )
            .await?;
            state_ctx.commit().await?;
        }
    }

    if let Err(err) = ManagementFuncJobState::new_pending(&ctx, component_id, prototype_id).await {
        match err {
            ManagementFuncExecutionError::CreationFailed => {
                return Ok(ForceChangeSetResponse::new(
                    force_change_set_id,
                    RunPrototypeResponse {
                        status: ManagementFuncStatus::Error,
                        message: "This management func is already running".to_string().into(),
                    },
                ));
            }
            other => Err(other)?,
        }
    }

    ctx.enqueue_management_func(prototype_id, component_id, view_id, request.request_ulid)
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

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        RunPrototypeResponse {
            status: ManagementFuncStatus::Ok,
            message: "enqueued".to_string().into(),
        },
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
