use std::collections::HashMap;

use axum::Json;
use dal::{
    workspace_snapshot::conflict::Conflict, AttributeValue, AttributeValueId, ChangeSet,
    ComponentId, Visibility, WorkspaceSnapshot, WorkspaceSnapshotError,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::component::ComponentResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConflictsForComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type ConflictsForComponentResponse =
    HashMap<AttributeValueId, si_frontend_types::ConflictWithHead>;

// TODO get visibility and component id via path as the V2 endpoints do
pub async fn conflicts_for_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(ConflictsForComponentRequest {
        component_id,
        visibility,
    }): Json<ConflictsForComponentRequest>,
) -> ComponentResult<Json<ConflictsForComponentResponse>> {
    let ctx = builder.build(request_ctx.build(visibility)).await?;

    // Get change set snapshot
    let change_set = ChangeSet::find(&ctx, visibility.change_set_id)
        .await?
        .ok_or(dal::ChangeSetError::ChangeSetNotFound(
            visibility.change_set_id,
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
        return Err(dal::ChangeSetError::NoBaseChangeSet(visibility.change_set_id).into());
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

    // TODO move this to the dal for ease of testing and write tests
    let mut conflicts_for_av_id = ConflictsForComponentResponse::new();

    for conflict in conflicts_and_updates_change_set_into_base.conflicts {
        let node_index = cs_workspace_snapshot
            .get_node_index_by_id(node_ulid_for_conflict(conflict))
            .await?;

        let node_weight = cs_workspace_snapshot.get_node_weight(node_index).await?;

        let Some(this_av_id) = cs_workspace_snapshot
            .associated_attribute_value_id(node_weight)
            .await?
        else {
            continue;
        };

        if AttributeValue::component_id(&ctx, this_av_id).await? != component_id {
            continue;
        }

        let frontend_conflict = match conflict {
            Conflict::ChildOrder { .. }
            | Conflict::ExclusiveEdgeMismatch { .. }
            | Conflict::NodeContent { .. } => si_frontend_types::ConflictWithHead::Untreated {
                raw: serde_json::json!(conflict).to_string(),
            },

            Conflict::ModifyRemovedItem { .. } => {
                si_frontend_types::ConflictWithHead::RemovedWhatHeadModified {
                    container_av_id: this_av_id.into(),
                }
            }

            Conflict::RemoveModifiedItem { .. } => {
                si_frontend_types::ConflictWithHead::ModifiedWhatHeadRemoved {
                    modified_av_id: this_av_id.into(),
                }
            }
        };

        conflicts_for_av_id.insert(this_av_id, frontend_conflict);
    }

    Ok(Json(conflicts_for_av_id))
}

fn node_ulid_for_conflict(conflict: Conflict) -> Ulid {
    match conflict {
        Conflict::ChildOrder { onto, .. } => onto.id,
        Conflict::ExclusiveEdgeMismatch { destination, .. } => destination.id,
        Conflict::ModifyRemovedItem { container, .. } => container.id,
        Conflict::NodeContent { onto, .. } => onto.id,
        Conflict::RemoveModifiedItem { removed_item, .. } => removed_item.id,
    }
    .into()
}
