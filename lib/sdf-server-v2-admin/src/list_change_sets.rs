use std::collections::HashMap;

use axum::{extract::Path, Json};
use dal::{ChangeSet, ChangeSetId, WorkspacePk};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use super::{AdminAPIResult, AdminChangeSet};
use axum_util::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListChangesetsResponse {
    change_sets: HashMap<ChangeSetId, AdminChangeSet>,
}

#[instrument(name = "admin.list_change_sets", skip_all)]
pub async fn list_change_sets(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path(workspace_pk): Path<WorkspacePk>,
) -> AdminAPIResult<Json<ListChangesetsResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let change_sets = ChangeSet::list_all_for_workspace(&ctx, workspace_pk)
        .await?
        .into_iter()
        .map(|change_set| (change_set.id, change_set.into()))
        .collect();

    Ok(Json(ListChangesetsResponse { change_sets }))
}
