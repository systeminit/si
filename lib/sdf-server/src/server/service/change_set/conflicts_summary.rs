use std::collections::HashSet;

use axum::Json;
use serde::{Deserialize, Serialize};

use dal::{
    workspace_snapshot::{conflict::Conflict, graph::NodeIndex},
    ChangeSet, ComponentId, FuncId, Visibility, WorkspaceSnapshot, WorkspaceSnapshotError,
};

use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConflictsSummaryRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConflictsSummaryResponse {
    pub components: Vec<ComponentId>,
    pub functions: Vec<FuncId>,
}

pub async fn conflicts_summary(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<ConflictsSummaryRequest>,
) -> ChangeSetResult<Json<ConflictsSummaryResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // Get change set snapshot
    let change_set = ChangeSet::find(&ctx, request.visibility.change_set_id)
        .await?
        .ok_or(dal::ChangeSetError::ChangeSetNotFound(
            request.visibility.change_set_id,
        ))?;
    let cs_workspace_snapshot = WorkspaceSnapshot::find_for_change_set(&ctx, change_set.id).await?;
    let cs_vector_clock_id = cs_workspace_snapshot
        .max_recently_seen_clock_id(Some(change_set.id))
        .await?
        .ok_or(WorkspaceSnapshotError::RecentlySeenClocksMissing(
            change_set.id,
        ))?;

    // Get base snapshot
    let base_change_set = if let Some(base_change_set_id) = change_set.base_change_set_id {
        ChangeSet::find(&ctx, base_change_set_id)
            .await?
            .ok_or(dal::ChangeSetError::ChangeSetNotFound(base_change_set_id))?
    } else {
        return Err(dal::ChangeSetError::NoBaseChangeSet(request.visibility.change_set_id).into());
    };
    let base_snapshot = WorkspaceSnapshot::find_for_change_set(&ctx, base_change_set.id).await?;
    let base_vector_clock_id = base_snapshot
        .max_recently_seen_clock_id(Some(base_change_set.id))
        .await?
        .ok_or(WorkspaceSnapshotError::RecentlySeenClocksMissing(
            base_change_set.id,
        ))?;

    let conflicts_and_updates_change_set_into_base = base_snapshot
        .detect_conflicts_and_updates(
            base_vector_clock_id,
            &cs_workspace_snapshot,
            cs_vector_clock_id,
        )
        .await?;

    let mut components = HashSet::new();
    for conflict in conflicts_and_updates_change_set_into_base.conflicts {
        if let Some(node_index) = change_set_node_index(conflict) {
            let node_weight = cs_workspace_snapshot.get_node_weight(node_index).await?;
            if let Some(component_id) = cs_workspace_snapshot
                .associated_component_id(&ctx, node_weight)
                .await?
            {
                components.insert(component_id);
            }
        }
    }

    Ok(Json(ConflictsSummaryResponse {
        components: components.iter().copied().collect(),
        functions: vec![],
    }))
}

fn change_set_node_index(conflict: Conflict) -> Option<NodeIndex> {
    match conflict {
        Conflict::ChildOrder { onto, .. } => Some(onto.index),
        Conflict::ExclusiveEdgeMismatch { destination, .. } => Some(destination.index),
        Conflict::ModifyRemovedItem(node) => Some(node.index),
        Conflict::NodeContent { onto, .. } => Some(onto.index),
        Conflict::RemoveModifiedItem { .. } => None,
    }
}
