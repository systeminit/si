use axum::{
    extract::rejection::JsonRejection,
    response::Json,
};
use dal::{
    WsEvent,
    change_set::ChangeSet,
};
use sdf_core::change_set_mvs::create_index_for_new_change_set_and_watch;
use sdf_extract::{
    EddaClient as EddaClientExtractor,
    FriggStore,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;
use utoipa::ToSchema;

use super::ChangeSetResult;
use crate::{
    api_types::change_sets::v1::ChangeSetViewV1,
    extract::{
        PosthogEventTracker,
        workspace::WorkspaceDalContext,
    },
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier")
    ),
    tag = "change_sets",
    summary = "Create a Change Set",
    request_body = CreateChangeSetV1Request,
    responses(
        (status = 200, description = "Change Set created successfully", body = CreateChangeSetV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn create_change_set(
    WorkspaceDalContext(ref ctx): WorkspaceDalContext,
    EddaClientExtractor(edda_client): EddaClientExtractor,
    FriggStore(frigg): FriggStore,
    tracker: PosthogEventTracker,
    payload: Result<Json<CreateChangeSetV1Request>, JsonRejection>,
) -> ChangeSetResult<Json<CreateChangeSetV1Response>> {
    let Json(CreateChangeSetV1Request { change_set_name }) = payload?;

    let change_set = ChangeSet::fork_head(ctx, &change_set_name).await?;
    let change_set_id = change_set.id;

    tracker.track(
        ctx,
        "api_create_change_set",
        serde_json::json!({
                    "change_set_name": change_set_name.clone(),
        }),
    );

    ctx.write_audit_log(AuditLogKind::CreateChangeSet, change_set_name.clone())
        .await?;

    WsEvent::change_set_created(ctx, change_set.id, change_set.workspace_snapshot_address)
        .await?
        .publish_on_commit(ctx)
        .await?;

    let change_set = ChangeSetViewV1 {
        id: change_set.id,
        name: change_set.clone().name,
        status: change_set.status,
        is_head: change_set.is_head(ctx).await?,
    };
    ctx.commit_no_rebase().await?;

    create_index_for_new_change_set_and_watch(
        &frigg,
        &edda_client,
        ctx.workspace_pk()?,
        change_set_id,
        ctx.change_set_id(), // note DalCtx is built for Head here to create a change set!
        ctx.workspace_snapshot()?.address().await,
    )
    .await?;

    Ok(Json(CreateChangeSetV1Response { change_set }))
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetV1Request {
    #[schema(example = "My new feature", required = true)]
    pub change_set_name: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetV1Response {
    #[schema(example = json!({
        "id": "01FXNV4P306V3KGZ73YSVN8A60",
        "name": "My new feature",
        "status": "Open",
        "isHead": "false"
    }))]
    pub change_set: ChangeSetViewV1,
}
