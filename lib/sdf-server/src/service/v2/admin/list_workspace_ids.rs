use axum::Json;
use dal::{
    Workspace,
    WorkspacePk,
};
use serde::{
    Deserialize,
    Serialize,
};
use telemetry::prelude::*;

use crate::service::v2::admin::{
    AdminAPIResult,
    AdminUserContext,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ListWorkspaceIdsResponse {
    workspaces: Vec<WorkspacePk>,
}

#[instrument(name = "admin.list_workspace_ids", skip_all)]
pub async fn list_workspace_ids(
    AdminUserContext(ctx): AdminUserContext,
) -> AdminAPIResult<Json<ListWorkspaceIdsResponse>> {
    let workspaces = Workspace::list_all(&ctx)
        .await?
        .into_iter()
        .map(|workspace| *workspace.pk())
        .collect();

    Ok(Json(ListWorkspaceIdsResponse { workspaces }))
}
