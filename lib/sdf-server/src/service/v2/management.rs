use axum::{
    Json,
    Router,
    extract::Path,
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
    ComponentError,
    ComponentId,
    DalContext,
    FuncError,
    FuncId,
    SchemaVariantError,
    TransactionsError,
    WorkspacePk,
    WsEventError,
    diagram::{
        DiagramError,
        view::{
            View,
            ViewId,
        },
    },
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
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
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
    service::force_change_set_response::ForceChangeSetResponse,
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
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("diagram error: {0}")]
    Diagram(#[from] DiagramError),
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
            ManagementApiError::Component(dal::ComponentError::NotFound(_)) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
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

pub async fn run_prototype_inner(
    ctx: &mut DalContext,
    prototype_id: ManagementPrototypeId,
    component_id: ComponentId,
    view_id: ViewId,
    request: RunPrototypeRequest,
    tracker: PosthogEventTracker,
) -> ManagementApiResult<ForceChangeSetResponse<RunPrototypeResponse>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;

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

    if let Err(err) = ManagementFuncJobState::new_pending(ctx, component_id, prototype_id).await {
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

    tracker.track(
        ctx,
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

pub async fn run_prototype(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path((_workspace_pk, _change_set_id, prototype_id, component_id, view_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ManagementPrototypeId,
        ComponentId,
        ViewId,
    )>,
    Json(request): Json<RunPrototypeRequest>,
) -> ManagementApiResult<ForceChangeSetResponse<RunPrototypeResponse>> {
    run_prototype_inner(ctx, prototype_id, component_id, view_id, request, tracker).await
}

pub async fn run_prototype_for_default_view(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path((_workspace_pk, _change_set_id, prototype_id, component_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ManagementPrototypeId,
        ComponentId,
    )>,
    Json(request): Json<RunPrototypeRequest>,
) -> ManagementApiResult<ForceChangeSetResponse<RunPrototypeResponse>> {
    run_prototype_inner(
        ctx,
        prototype_id,
        component_id,
        View::get_id_for_default(ctx).await?,
        request,
        tracker,
    )
    .await
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        // TODO(Victor): rewrite these to use a viewId resolver instead of two endpoints
        .route(
            "/prototype/:prototypeId/:componentId/DEFAULT",
            post(run_prototype_for_default_view),
        )
        .route(
            "/prototype/:prototypeId/:componentId/:viewId",
            post(run_prototype),
        )
        .route(
            "/prototype/:prototypeId/:componentId/latest",
            get(latest::latest),
        )
        .route(
            "/component/:componentId/latest",
            get(latest::all_latest_for_component),
        )
        .route("/history", get(history::history))
        .route(
            "/generate_template/:viewId",
            post(generate_template::generate_template),
        )
}
