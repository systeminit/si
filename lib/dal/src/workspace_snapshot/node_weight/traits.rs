use thiserror::Error;

use crate::{workspace_snapshot::graph::ConflictsAndUpdates, WorkspaceSnapshotGraphV1};

use super::NodeWeight;

#[derive(Debug, Error)]
pub enum UpdateConflictsAndUpdatesError {}

pub type UpdateConflictsAndUpdatesResult<T> = Result<T, UpdateConflictsAndUpdatesError>;

pub trait UpdateConflictsAndUpdates {
    fn update_conflicts_and_updates(
        &self,
        _to_rebase_workspace_snapshot: &WorkspaceSnapshotGraphV1,
        _other_workspace_snapshot: &WorkspaceSnapshotGraphV1,
        conflicts_and_updates: ConflictsAndUpdates,
    ) -> UpdateConflictsAndUpdatesResult<ConflictsAndUpdates> {
        Ok(conflicts_and_updates)
    }
}

impl UpdateConflictsAndUpdates for NodeWeight {
    fn update_conflicts_and_updates(
        &self,
        to_rebase_workspace_snapshot: &WorkspaceSnapshotGraphV1,
        other_workspace_snapshot: &WorkspaceSnapshotGraphV1,
        conflicts_and_updates: ConflictsAndUpdates,
    ) -> UpdateConflictsAndUpdatesResult<ConflictsAndUpdates> {
        match self {
            NodeWeight::Action(action_weight) => action_weight.update_conflicts_and_updates(
                to_rebase_workspace_snapshot,
                other_workspace_snapshot,
                conflicts_and_updates,
            ),
            NodeWeight::ActionPrototype(action_prototype_weight) => action_prototype_weight
                .update_conflicts_and_updates(
                    to_rebase_workspace_snapshot,
                    other_workspace_snapshot,
                    conflicts_and_updates,
                ),
            NodeWeight::AttributePrototypeArgument(attribute_prototype_argument_weight) => {
                attribute_prototype_argument_weight.update_conflicts_and_updates(
                    to_rebase_workspace_snapshot,
                    other_workspace_snapshot,
                    conflicts_and_updates,
                )
            }
            NodeWeight::AttributeValue(attribute_value_weight) => attribute_value_weight
                .update_conflicts_and_updates(
                    to_rebase_workspace_snapshot,
                    other_workspace_snapshot,
                    conflicts_and_updates,
                ),
            NodeWeight::Category(category_node_weight) => category_node_weight
                .update_conflicts_and_updates(
                    to_rebase_workspace_snapshot,
                    other_workspace_snapshot,
                    conflicts_and_updates,
                ),
            NodeWeight::Component(component_weight) => component_weight
                .update_conflicts_and_updates(
                    to_rebase_workspace_snapshot,
                    other_workspace_snapshot,
                    conflicts_and_updates,
                ),
            NodeWeight::Content(content_weight) => content_weight.update_conflicts_and_updates(
                to_rebase_workspace_snapshot,
                other_workspace_snapshot,
                conflicts_and_updates,
            ),
            NodeWeight::DependentValueRoot(dv_root_weight) => dv_root_weight
                .update_conflicts_and_updates(
                    to_rebase_workspace_snapshot,
                    other_workspace_snapshot,
                    conflicts_and_updates,
                ),
            NodeWeight::Func(func_weight) => func_weight.update_conflicts_and_updates(
                to_rebase_workspace_snapshot,
                other_workspace_snapshot,
                conflicts_and_updates,
            ),
            NodeWeight::FuncArgument(func_argument_weight) => func_argument_weight
                .update_conflicts_and_updates(
                    to_rebase_workspace_snapshot,
                    other_workspace_snapshot,
                    conflicts_and_updates,
                ),
            NodeWeight::Ordering(ordering_weight) => ordering_weight.update_conflicts_and_updates(
                to_rebase_workspace_snapshot,
                other_workspace_snapshot,
                conflicts_and_updates,
            ),
            NodeWeight::Prop(prop_weight) => prop_weight.update_conflicts_and_updates(
                to_rebase_workspace_snapshot,
                other_workspace_snapshot,
                conflicts_and_updates,
            ),
            NodeWeight::Secret(secret_weight) => secret_weight.update_conflicts_and_updates(
                to_rebase_workspace_snapshot,
                other_workspace_snapshot,
                conflicts_and_updates,
            ),
        }
    }
}
