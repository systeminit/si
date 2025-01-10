use std::collections::HashMap;

use axum::{extract::Path, Json};
use dal::{ChangeSet, ChangeSetId, Tenancy, WorkspacePk};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::service::v2::admin::{AdminAPIResult, AdminChangeSet, AdminUserContext};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListChangesetsResponse {
    change_sets: HashMap<ChangeSetId, AdminChangeSet>,
}

#[instrument(name = "admin.list_change_sets", skip_all)]
pub async fn list_change_sets(
    AdminUserContext(mut ctx): AdminUserContext,
    Path(workspace_id): Path<WorkspacePk>,
) -> AdminAPIResult<Json<ListChangesetsResponse>> {
    ctx.update_tenancy(Tenancy::new(workspace_id));

    let change_sets = ChangeSet::list_all_for_workspace(&ctx, workspace_id)
        .await?
        .into_iter()
        .map(|change_set| (change_set.id, change_set.into()))
        .collect();

    Ok(Json(ListChangesetsResponse { change_sets }))
}
