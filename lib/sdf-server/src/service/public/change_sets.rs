use axum::{
    extract::Host,
    middleware,
    routing::{get, post},
    Json, Router,
};
use dal::{
    action::{
        prototype::{ActionKind, ActionPrototype},
        Action, ActionState,
    },
    change_set::ChangeSet,
    ChangeSetId, ComponentId, DalContext, WsEvent,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use si_events::{audit_log::AuditLogKind, ActionId, ChangeSetStatus};
use thiserror::Error;

use crate::{
    extract::{
        change_set::{ChangeSetDalContext, TargetChangeSetIdFromPath},
        workspace::{WorkspaceAuthorization, WorkspaceDalContext},
        PosthogEventTracker,
    },
    routes::AppError,
    service::v2::change_set::post_to_webhook,
};
use crate::{middleware::WorkspacePermissionLayer, AppState};

// /api/public/workspaces/:workspace_id/change-sets
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new().route("/", post(create_change_set)).nest(
        "/:change_set_id",
        Router::new()
            .nest("/components", super::components::routes())
            .nest("/management", super::management::routes())
            .route("/request_approval", post(request_approval))
            .route(
                "/force_apply",
                post(force_apply).route_layer(WorkspacePermissionLayer::new(
                    state,
                    permissions::Permission::Approve,
                )),
            )
            .route("/merge_status", get(merge_status))
            .route_layer(middleware::from_extractor::<TargetChangeSetIdFromPath>()),
    )
}

async fn create_change_set(
    WorkspaceDalContext(ref ctx): WorkspaceDalContext,
    tracker: PosthogEventTracker,
    Json(payload): Json<CreateChangeSetRequest>,
) -> Result<Json<CreateChangeSetResponse>, AppError> {
    let change_set = ChangeSet::fork_head(ctx, &payload.change_set_name).await?;

    tracker.track(ctx, "fs_create_change_set", json!(payload));

    ctx.write_audit_log(AuditLogKind::CreateChangeSet, payload.change_set_name)
        .await?;

    WsEvent::change_set_created(ctx, change_set.id)
        .await?
        .publish_on_commit(ctx)
        .await?;

    ctx.commit_no_rebase().await?;

    Ok(Json(CreateChangeSetResponse { change_set }))
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateChangeSetRequest {
    change_set_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateChangeSetResponse {
    change_set: ChangeSet,
}

// Get status of a change set and its actions
async fn merge_status(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
) -> Result<Json<MergeStatusResponse>, AppError> {
    let change_set = ctx.change_set()?.into_frontend_type(ctx).await?;

    let actions = match change_set.status {
        // Grab action status from HEAD since we don't get updates anymore after being applied
        ChangeSetStatus::Applied => {
            get_action_statuses(&ctx.clone_with_base().await?, change_set.id).await?
        }
        _ => get_action_statuses(ctx, change_set.id).await?,
    };

    Ok(Json(MergeStatusResponse {
        change_set,
        actions,
    }))
}

async fn get_action_statuses(
    ctx: &DalContext,
    change_set_id: ChangeSetId,
) -> Result<Vec<MergeStatusResponseAction>, AppError> {
    let mut actions = Vec::new();

    for action_id in Action::all_ids(ctx).await? {
        let action = Action::get_by_id(ctx, action_id).await?;
        let ActionPrototype { kind, name, .. } = Action::prototype(ctx, action_id).await?;
        let component = match Action::component(ctx, action_id).await? {
            Some(component) => Some(MergeStatusResponseActionComponent {
                id: component.id(),
                name: component.name(ctx).await?,
            }),
            None => None,
        };

        if action.originating_changeset_id() == change_set_id {
            actions.push(MergeStatusResponseAction {
                id: action_id,
                component,
                state: action.state(),
                kind,
                name,
            })
        }
    }

    Ok(actions)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MergeStatusResponse {
    change_set: si_frontend_types::ChangeSet,
    actions: Vec<MergeStatusResponseAction>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MergeStatusResponseAction {
    id: ActionId,
    component: Option<MergeStatusResponseActionComponent>,
    state: ActionState,
    kind: ActionKind,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MergeStatusResponseActionComponent {
    id: ComponentId,
    name: String,
}

async fn force_apply(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
) -> Result<(), AppError> {
    let change_set_id = ctx.change_set_id();
    let old_status = ctx.change_set()?.status;
    ChangeSet::prepare_for_force_apply(ctx).await?;
    ctx.write_audit_log(
        AuditLogKind::ApproveChangeSetApply {
            from_status: old_status.into(),
        },
        ctx.change_set()?.name.clone(),
    )
    .await?;
    // We need to run a commit before apply so changes get saved
    ctx.commit().await?;

    ChangeSet::apply_to_base_change_set(ctx).await?;

    tracker.track(
        ctx,
        "apply_change_set",
        serde_json::json!({
            "merged_change_set": change_set_id,
        }),
    );

    // TODO isn't this the same change_set_id() as before? If so, the name hasn't changed ...
    let change_set = ChangeSet::get_by_id(ctx, ctx.change_set_id()).await?;

    ctx.write_audit_log(AuditLogKind::ApplyChangeSet, change_set.name)
        .await?;
    // Ws Event fires from the dal

    ctx.commit().await?;

    Ok(())
}

async fn request_approval(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    WorkspaceAuthorization { user, .. }: WorkspaceAuthorization,
    tracker: PosthogEventTracker,
    Host(host_name): Host,
) -> Result<(), AppError> {
    let workspace_pk = ctx.workspace_pk()?;
    let mut change_set = ctx.change_set()?.clone();
    let change_set_id = change_set.id;
    let old_status = change_set.status;

    change_set.request_change_set_approval(ctx).await?;

    tracker.track(
        ctx,
        "request_change_set_approval",
        serde_json::json!({
            "change_set": change_set.id,
        }),
    );
    // TODO change to get_by_id when https://github.com/systeminit/si/pull/5261 lands
    let change_set_view = ChangeSet::get_by_id(ctx, change_set_id)
        .await?
        .into_frontend_type(ctx)
        .await?;

    let change_set_url = format!(
        "https://{}/w/{}/{}",
        host_name,
        ctx.workspace_pk()?,
        change_set_id
    );
    let message = format!(
        "{} requested an approval of change set {}: {}",
        user.email(),
        change_set_view.name.clone(),
        change_set_url
    );
    post_to_webhook(ctx, workspace_pk, message.as_str()).await?;

    ctx.write_audit_log(
        AuditLogKind::RequestChangeSetApproval {
            from_status: old_status.into(),
        },
        change_set_view.name.clone(),
    )
    .await?;

    WsEvent::change_set_status_changed(ctx, old_status, change_set_view)
        .await?
        .publish_on_commit(ctx)
        .await?;

    ctx.commit().await?;

    Ok(())
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetsError {
    #[error("action error: {0}")]
    Action(#[from] dal::action::ActionError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("change set apply error: {0}")]
    ChangeSetApply(#[from] dal::ChangeSetApplyError),
    #[error("change set service error: {0}")]
    ChangeSetService(#[from] crate::service::v2::change_set::Error),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("func error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}
