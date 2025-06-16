use std::collections::{
    HashMap,
    HashSet,
};

use petgraph::{
    Direction,
    prelude::*,
};
use si_events::{
    merkle_tree_hash::MerkleTreeHash,
    workspace_snapshot::{
        Change,
        EntityKind,
    },
};
use si_id::{
    ApprovalRequirementDefinitionId,
    EntityId,
    WorkspacePk,
};

use super::WorkspaceSnapshotGraphV4;
use crate::{
    EdgeWeightKindDiscriminants,
    NodeWeightDiscriminants,
    workspace_snapshot::{
        graph::{
            WorkspaceSnapshotGraphError,
            WorkspaceSnapshotGraphResult,
            traits::approval_requirement::{
                ApprovalRequirementApprover,
                ApprovalRequirementExt,
                ApprovalRequirementPermissionLookup,
                ApprovalRequirementRule,
                ApprovalRequirementsBag,
            },
        },
        node_weight::traits::SiNodeWeight,
    },
};

impl ApprovalRequirementExt for WorkspaceSnapshotGraphV4 {
    fn approval_requirements_for_changes(
        &self,
        workspace_id: WorkspacePk,
        changes: &[Change],
    ) -> WorkspaceSnapshotGraphResult<(
        Vec<ApprovalRequirementsBag>,
        HashMap<EntityId, MerkleTreeHash>,
    )> {
        let mut requirements = Vec::new();
        let mut ids_with_hashes_for_deleted_nodes = HashMap::new();

        // Some changes should be treated as though they are a Change for something else, until we
        // re-work how this is all being generated to be able to "generate" explicit requirements
        // for an EntityId that does not directly have the ApprovalRequirementDefinition it is using
        // to generate the approval requirement.
        let mut changes_to_add = Vec::new();
        let mut change_idxs_to_remove = Vec::new();
        let mut modified_view_ids = HashSet::new();
        for (change_idx, change) in changes.iter().enumerate() {
            match change.entity_kind {
                // Keep track of which Views already have Changes so we don't add duplicate ones
                // when we add Changes for the Views containing the Component for the Action.
                EntityKind::View => {
                    modified_view_ids.insert(change.entity_id);
                    continue;
                }
                // If there is a change involving an Action, and we can determine the Component for the Action,
                // We want to treat the Views that Component is in as changed to have them generate approval
                // requirements.
                EntityKind::Action => {
                    let Some(action_node_idx) = self.get_node_index_by_id_opt(change.entity_id)
                    else {
                        // The Action has been removed, and we no longer have access to the base graph where it did exist
                        // at this point, so let the default virtual requirement handling take care of this Action.
                        continue;
                    };
                    let mut maybe_component_node_idx = None;
                    for edge_ref in self.edges_directed(action_node_idx, Direction::Outgoing) {
                        let target_node_weight = self.get_node_weight(edge_ref.target())?;
                        if NodeWeightDiscriminants::Component == target_node_weight.into() {
                            maybe_component_node_idx = Some(edge_ref.target());
                            break;
                        }
                    }

                    if let Some(component_node_idx) = maybe_component_node_idx {
                        let component_id = self.get_node_weight(component_node_idx)?.id();
                        let view_ids = crate::workspace_snapshot::graph::traits::diagram::view::ViewExt::list_for_component_id(
                            self,
                            component_id.into(),
                        )?;
                        for view_id in view_ids {
                            changes_to_add.push(Change {
                                entity_id: view_id.into_inner().into(),
                                entity_kind: EntityKind::View,
                                merkle_tree_hash: self
                                    .get_node_weight_by_id(view_id)?
                                    .merkle_tree_hash(),
                            });
                        }
                        change_idxs_to_remove.push(change_idx);
                    }
                }
                _ => {}
            }
        }
        change_idxs_to_remove.sort();
        change_idxs_to_remove.reverse();
        let mut local_changes = changes.to_vec();
        for change_idx_to_remove in change_idxs_to_remove {
            local_changes.remove(change_idx_to_remove);
        }
        changes_to_add.retain(|change| !modified_view_ids.contains(&change.entity_id));
        local_changes.extend(changes_to_add);

        for change in &local_changes {
            let mut explicit_approval_requirement_definition_ids = Vec::new();
            let mut virtual_approval_requirement_rules = Vec::new();

            // Check if the node exists in the current graph. If it does, we are working with an
            // addition or a modification. If it does not, we are working with a removal.
            if let Some(entity_node_index) = self.get_node_index_by_id_opt(change.entity_id) {
                for (_, _, requirement_node_index) in self.edges_directed_for_edge_weight_kind(
                    entity_node_index,
                    Direction::Outgoing,
                    EdgeWeightKindDiscriminants::ApprovalRequirementDefinition,
                ) {
                    let requirement_node_weight = self
                        .get_node_weight(requirement_node_index)?
                        .get_approval_requirement_definition_node_weight()?;
                    explicit_approval_requirement_definition_ids
                        .push(requirement_node_weight.id().into());
                }

                // If we did not find any explicit requirements, check if we need to create virtual
                // requirements.
                if explicit_approval_requirement_definition_ids.is_empty() {
                    if let Some(virtual_rule) = new_virtual_requirement_rule(workspace_id, change) {
                        virtual_approval_requirement_rules.push(virtual_rule);
                    }
                }
            } else {
                // If the node does not exist on the current graph, then we know it was deleted.
                if let Some(existing_merkle_tree_hash) = ids_with_hashes_for_deleted_nodes
                    .insert(change.entity_id, change.merkle_tree_hash)
                {
                    // NOTE(nick): this is one of those "heat death of the universe" errors, but
                    // both you and I do not want to be paged because of a hidden map insertion,
                    // now do we? Didn't think so!
                    return Err(
                        WorkspaceSnapshotGraphError::MultipleMerkleTreeHashesForEntity(
                            change.entity_id,
                            change.merkle_tree_hash,
                            existing_merkle_tree_hash,
                        ),
                    );
                };

                // If the node does not exist on the current graph and it is an approval
                // requirement definition node, then we know that the approval requirement
                // definition node was deleted. We will need a virtual requirement for this
                // removal.
                if let EntityKind::ApprovalRequirementDefinition = change.entity_kind {
                    virtual_approval_requirement_rules.push(ApprovalRequirementRule {
                        entity_id: change.entity_id,
                        entity_kind: change.entity_kind,
                        minimum: 1,
                        approvers: HashSet::from([ApprovalRequirementApprover::PermissionLookup(
                            ApprovalRequirementPermissionLookup {
                                object_type: "workspace".to_string(),
                                object_id: workspace_id.to_string(),
                                permission: "approve".to_string(),
                            },
                        )]),
                    });
                }
            }

            requirements.push(ApprovalRequirementsBag {
                entity_id: change.entity_id,
                entity_kind: change.entity_kind,
                explicit_approval_requirement_definition_ids,
                virtual_approval_requirement_rules,
            });
        }

        Ok((requirements, ids_with_hashes_for_deleted_nodes))
    }

    fn approval_requirement_definitions_for_entity_id_opt(
        &self,
        entity_id: EntityId,
    ) -> WorkspaceSnapshotGraphResult<Option<Vec<ApprovalRequirementDefinitionId>>> {
        let mut explicit_approval_requirement_definition_ids = Vec::new();
        if let Some(entity_node_index) = self.get_node_index_by_id_opt(entity_id) {
            for (_, _, requirement_node_index) in self.edges_directed_for_edge_weight_kind(
                entity_node_index,
                Direction::Outgoing,
                EdgeWeightKindDiscriminants::ApprovalRequirementDefinition,
            ) {
                let requirement_node_weight = self
                    .get_node_weight(requirement_node_index)?
                    .get_approval_requirement_definition_node_weight()?;
                explicit_approval_requirement_definition_ids
                    .push(requirement_node_weight.id().into());
            }
        } else {
            return Ok(None);
        }

        Ok(Some(explicit_approval_requirement_definition_ids))
    }

    fn entity_id_for_approval_requirement(
        &self,
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotGraphResult<EntityId> {
        if let Some(approval_requirement_index) =
            self.get_node_index_by_id_opt(approval_requirement_definition_id)
        {
            for (_, source_index, _) in self.edges_directed_for_edge_weight_kind(
                approval_requirement_index,
                Direction::Incoming,
                EdgeWeightKindDiscriminants::ApprovalRequirementDefinition,
            ) {
                if let Some(entity_id) = self.node_index_to_id(source_index) {
                    return Ok(entity_id.into());
                }
            }
        }
        Err(
            WorkspaceSnapshotGraphError::EntityNotFoundForApprovalRequirementDefinition(
                approval_requirement_definition_id,
            ),
        )
    }
}

pub fn new_virtual_requirement_rule(
    workspace_id: WorkspacePk,
    change: &Change,
) -> Option<ApprovalRequirementRule> {
    match change.entity_kind {
        // Default approval requirement rule for actions, funcs, schemas, schema variants,
        // and views until we get proper fallback logic for who should be approving what.
        EntityKind::Action
        | EntityKind::Func
        | EntityKind::Schema
        | EntityKind::SchemaVariant
        | EntityKind::View => Some(ApprovalRequirementRule {
            entity_id: change.entity_id,
            entity_kind: change.entity_kind,
            minimum: 1,
            approvers: HashSet::from([ApprovalRequirementApprover::PermissionLookup(
                ApprovalRequirementPermissionLookup {
                    object_type: "workspace".to_string(),
                    object_id: workspace_id.to_string(),
                    permission: "approve".to_string(),
                },
            )]),
        }),
        // For any changes to explicit approval requirements, we need approvals from
        // workspace approvers.
        EntityKind::ApprovalRequirementDefinition => Some(ApprovalRequirementRule {
            entity_id: change.entity_id,
            entity_kind: change.entity_kind,
            minimum: 1,
            approvers: HashSet::from([ApprovalRequirementApprover::PermissionLookup(
                ApprovalRequirementPermissionLookup {
                    object_type: "workspace".to_string(),
                    object_id: workspace_id.to_string(),
                    permission: "approve".to_string(),
                },
            )]),
        }),
        _ => None,
    }
}
