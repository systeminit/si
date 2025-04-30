use std::collections::{
    HashMap,
    HashSet,
};

use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    Component,
    ComponentId,
    component::delete::{
        self,
        ComponentDeletionStatus,
    },
};
use serde::Serialize;
use utoipa::{
    self,
    ToSchema,
};

use super::ComponentV1RequestPath;
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::ComponentsError,
};

#[utoipa::path(
    delete,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier"),
        ("component_id", description = "Component identifier")
    ),
    tag = "components",
    responses(
        (status = 200, description = "Component deleted successfully", body = DeleteComponentV1Response),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn delete_component(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    _tracker: PosthogEventTracker,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
) -> Result<Json<DeleteComponentV1Response>, ComponentsError> {
    let head_components: HashSet<ComponentId> =
        Component::exists_on_head(ctx, &[component_id]).await?;

    let mut socket_map = HashMap::new();
    let mut socket_map_head = HashMap::new();
    let base_change_set_ctx = ctx.clone_with_base().await?;

    let status = delete::delete_and_process(
        ctx,
        false,
        &head_components,
        &mut socket_map,
        &mut socket_map_head,
        &base_change_set_ctx,
        component_id,
    )
    .await?;

    ctx.commit().await?;

    Ok(Json(DeleteComponentV1Response {
        status: match status {
            ComponentDeletionStatus::MarkedForDeletion => "marked_for_deletion".to_string(),
            ComponentDeletionStatus::StillExistsOnHead => "still_exists_on_head".to_string(),
            ComponentDeletionStatus::Deleted => "deleted".to_string(),
        },
    }))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComponentV1Response {
    #[schema(example = "MarkedForDeletion")]
    pub status: String,
}
