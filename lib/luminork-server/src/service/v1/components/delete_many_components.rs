use std::collections::{
    HashMap,
    HashSet,
};

use axum::response::Json;
use dal::{
    Component,
    ComponentId,
    component::delete::{
        self,
        ComponentDeletionStatus,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use utoipa::{
    self,
    ToSchema,
};

use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::ComponentsError,
};

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteManyComponentsV1Request {
    #[schema(value_type = Vec<String>)]
    pub component_ids: Vec<ComponentId>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteManyComponentsV1Response {
    pub results: Vec<ComponentDeletionResult>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDeletionResult {
    #[schema(value_type = String)]
    pub component_id: ComponentId,
    pub status: String, // "marked_for_deletion" | "still_exists_on_head" | "deleted"
}

#[utoipa::path(
    delete,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/delete_many",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "components",
    request_body = DeleteManyComponentsV1Request,
    summary = "Delete multiple components",
    responses(
        (status = 200, description = "Components deleted successfully", body = DeleteManyComponentsV1Response),
        (status = 400, description = "Bad Request - Not permitted on HEAD"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn delete_many_components(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    posthog: PosthogEventTracker,
    payload: Result<Json<DeleteManyComponentsV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<DeleteManyComponentsV1Response>, ComponentsError> {
    let Json(payload) = payload?;

    // Validate not on HEAD change set
    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(ComponentsError::NotPermittedOnHead);
    }

    // Fetch shared data once upfront for all deletions
    let head_components: HashSet<ComponentId> =
        Component::exists_on_head_by_ids(ctx, &payload.component_ids).await?;
    let base_change_set_ctx = ctx.clone_with_base().await?;
    let mut socket_map = HashMap::new();
    let mut socket_map_head = HashMap::new();

    let mut results = Vec::with_capacity(payload.component_ids.len());

    // Process each deletion in order, stop on first error
    for (index, component_id) in payload.component_ids.iter().enumerate() {
        let status = delete::delete_and_process(
            ctx,
            false, // force_erase = false for soft delete
            &head_components,
            &mut socket_map,
            &mut socket_map_head,
            &base_change_set_ctx,
            *component_id,
        )
        .await
        .map_err(|e| ComponentsError::BulkOperationFailed {
            index,
            source: Box::new(e.into()),
        })?;

        results.push(ComponentDeletionResult {
            component_id: *component_id,
            status: match status {
                ComponentDeletionStatus::MarkedForDeletion => "marked_for_deletion".to_string(),
                ComponentDeletionStatus::StillExistsOnHead => "still_exists_on_head".to_string(),
                ComponentDeletionStatus::Deleted => "deleted".to_string(),
            },
        });
    }

    // Track bulk deletion (non-transactional analytics)
    posthog.track(
        ctx,
        "api_delete_many_components",
        json!({
            "count": results.len(),
        }),
    );

    // Commit (publishes queued operations transactionally)
    ctx.commit().await?;

    Ok(Json(DeleteManyComponentsV1Response { results }))
}
