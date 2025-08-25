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
        put,
    },
};
use dal::{
    ActionPrototypeId,
    ChangeSet,
    ChangeSetError,
    ChangeSetId,
    ComponentError,
    ComponentId,
    Func,
    FuncError,
    SchemaError,
    SchemaVariantError,
    TransactionsError,
    UserPk,
    WorkspacePk,
    WorkspaceSnapshotError,
    action::{
        Action,
        prototype::{
            ActionKind,
            ActionPrototype,
            ActionPrototypeError,
        },
    },
    slow_rt::SlowRuntimeError,
};
use sdf_core::{
    api_error::ApiError,
    force_change_set_response::ForceChangeSetResponse,
};
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_events::{
    ActionState,
    audit_log::AuditLogKind,
};
use si_id::{
    ActionId,
    FuncRunId,
};
use si_layer_cache::LayerDbError;
use thiserror::Error;

use crate::{
    app_state::AppState,
    extract::HandlerContext,
    service::v2::AccessBuilder,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ActionRequestError {
    #[error(transparent)]
    Action(#[from] dal::action::ActionError),
    #[error("action already enqueued: {0}")]
    ActionAlreadyEnqueued(ActionPrototypeId),
    #[error("action history is missing a field - this is a bug!: {0}")]
    ActionHistoryFieldMissing(String),
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("changeset error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error("component {0} not found")]
    ComponentNotFound(ComponentId),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error("Cannot cancel Running or Dispatched actions. ActionId {0}")]
    InvalidActionCancellation(ActionId),
    #[error("Cannot update action state that's not Queued to On Hold. Action with Id {0}")]
    InvalidOnHoldTransition(ActionId),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("invalid user {0}")]
    InvalidUser(UserPk),
    #[error("invalid user system init")]
    InvalidUserSystemInit,
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("no schema found for component {0}")]
    NoSchemaForComponent(ComponentId),
    #[error("no schema variant found for component {0}")]
    NoSchemaVariantForComponent(ComponentId),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("serrde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type ActionResult<T> = Result<T, ActionRequestError>;

impl IntoResponse for ActionRequestError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            ActionRequestError::InvalidOnHoldTransition(_) => {
                (StatusCode::NOT_MODIFIED, self.to_string())
            }
            ActionRequestError::Action(dal::action::ActionError::WorkspaceSnapshot(err))
                if err.is_node_with_id_not_found() =>
            {
                (StatusCode::GONE, err.to_string())
            }
            ActionRequestError::ActionAlreadyEnqueued(_)
            | ActionRequestError::Action(dal::action::ActionError::ActionAlreadyEnqueued(_)) => {
                (StatusCode::CONFLICT, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        ApiError::new(status_code, error_message).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/add", post(add))
        .route("/refresh/:component_id", put(refresh))
        .route("/:action_id/cancel", put(cancel))
        .route("/:action_id/put_on_hold", put(hold))
        .route("/:action_id/retry", put(retry))
        .route("/:action_id/func_run_id", get(get_func_run_id))
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddActionRequest {
    pub component_id: ComponentId,
    pub prototype_id: ActionPrototypeId,
}

pub async fn add(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    Path((_workspace_pk, _change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(AddActionRequest {
        component_id,
        prototype_id,
    }): Json<AddActionRequest>,
) -> ActionResult<ForceChangeSetResponse<()>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;
    let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;

    match prototype.kind {
        ActionKind::Create | ActionKind::Destroy | ActionKind::Update | ActionKind::Refresh => {
            let maybe_duplicate_action =
                Action::find_for_kind_and_component_id(ctx, component_id, prototype.kind).await?;
            if !maybe_duplicate_action.is_empty() {
                return Err(ActionRequestError::ActionAlreadyEnqueued(prototype.id));
            }
        }

        dal::action::prototype::ActionKind::Manual => {}
    }

    let func_id = ActionPrototype::func_id(ctx, prototype.id).await?;
    let func = Func::get_by_id(ctx, func_id).await?;
    Action::new(ctx, prototype_id, Some(component_id)).await?;
    ctx.write_audit_log(
        AuditLogKind::AddAction {
            prototype_id: prototype.id(),
            action_kind: prototype.kind.into(),
            func_id,
            func_display_name: func.display_name,
            func_name: func.name.clone(),
            component_id: Some(component_id),
        },
        func.name,
    )
    .await?;

    ctx.commit().await?;
    Ok(ForceChangeSetResponse::new(force_change_set_id, ()))
}

pub async fn refresh(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_pk, change_set_id, component_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ComponentId,
    )>,
) -> ActionResult<()> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    Action::enqueue_refresh_in_correct_change_set_and_commit(&ctx, component_id).await?;

    tracker.track(
        &ctx,
        "refresh_component",
        json!({
            "how": "/actions/refresh",
            "component_id": component_id,
            "change_set_id": ctx.change_set_id(),
        }),
    );
    Ok(())
}

pub async fn cancel(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, action_id)): Path<(WorkspacePk, ChangeSetId, ActionId)>,
) -> ActionResult<()> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let prototype_id = Action::prototype_id(&ctx, action_id).await?;
    let component_id = Action::component_id(&ctx, action_id).await?;
    let prototype = ActionPrototype::get_by_id(&ctx, prototype_id).await?;
    let func_id = ActionPrototype::func_id(&ctx, prototype_id).await?;
    let func = Func::get_by_id(&ctx, func_id).await?;
    ctx.write_audit_log(
        AuditLogKind::CancelAction {
            prototype_id,
            action_kind: prototype.kind.into(),
            func_id,
            func_display_name: func.display_name,
            func_name: func.name.clone(),
            component_id,
        },
        func.name,
    )
    .await?;

    Action::remove_by_id(&ctx, action_id).await?;
    ctx.commit().await?;
    Ok(())
}

pub async fn hold(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, action_id)): Path<(WorkspacePk, ChangeSetId, ActionId)>,
) -> ActionResult<()> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let action = Action::get_by_id(&ctx, action_id).await?;

    match action.state() {
        ActionState::Running | ActionState::Dispatched | ActionState::OnHold => {
            return Err(ActionRequestError::InvalidOnHoldTransition(action_id));
        }
        ActionState::Queued | ActionState::Failed => {}
    }

    let prototype_id = Action::prototype_id(&ctx, action_id).await?;
    let component_id = Action::component_id(&ctx, action_id).await?;
    let prototype = ActionPrototype::get_by_id(&ctx, prototype_id).await?;
    let func_id = ActionPrototype::func_id(&ctx, prototype_id).await?;
    let func = Func::get_by_id(&ctx, func_id).await?;

    Action::set_state(&ctx, action.id(), ActionState::OnHold).await?;

    ctx.write_audit_log(
        AuditLogKind::HoldAction {
            prototype_id,
            action_kind: prototype.kind.into(),
            func_id,
            func_display_name: func.display_name,
            func_name: func.name.clone(),
            component_id,
        },
        func.name,
    )
    .await?;

    ctx.commit().await?;
    Ok(())
}

pub async fn retry(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, action_id)): Path<(WorkspacePk, ChangeSetId, ActionId)>,
) -> ActionResult<()> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let action = Action::get_by_id(&ctx, action_id).await?;

    let prototype_id = Action::prototype_id(&ctx, action_id).await?;
    let component_id = Action::component_id(&ctx, action_id).await?;
    let prototype = ActionPrototype::get_by_id(&ctx, prototype_id).await?;
    let func_id = ActionPrototype::func_id(&ctx, prototype_id).await?;
    let func = Func::get_by_id(&ctx, func_id).await?;
    ctx.write_audit_log(
        AuditLogKind::RetryAction {
            prototype_id,
            action_kind: prototype.kind.into(),
            func_id,
            func_display_name: func.display_name,
            func_name: func.name.clone(),
            component_id,
        },
        func.name,
    )
    .await?;

    match action.state() {
        ActionState::Running | ActionState::Dispatched => {
            return Err(ActionRequestError::InvalidOnHoldTransition(action_id));
        }
        ActionState::Queued | ActionState::Failed | ActionState::OnHold => {}
    }
    Action::set_state(&ctx, action.id(), ActionState::Queued).await?;
    ctx.commit().await?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestFuncRunId {
    pub func_run_id: Option<FuncRunId>,
}

pub async fn get_func_run_id(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, action_id)): Path<(WorkspacePk, ChangeSetId, ActionId)>,
) -> ActionResult<Json<LatestFuncRunId>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let func_run_id = Action::last_func_run_id_for_id_opt(&ctx, action_id).await?;
    Ok(Json(LatestFuncRunId { func_run_id }))
}
