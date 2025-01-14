use std::collections::{HashMap, HashSet};

use petgraph::Direction;
use si_events::{merkle_tree_hash::MerkleTreeHash, workspace_snapshot::EntityKind};
use si_id::{EntityId, WorkspacePk};

use crate::{
    workspace_snapshot::{
        graph::{
            detector::Change,
            traits::approval_requirement::{
                ApprovalRequirementApprover, ApprovalRequirementExt,
                ApprovalRequirementPermissionLookup, ApprovalRequirementRule,
                ApprovalRequirementsBag,
            },
            WorkspaceSnapshotGraphError, WorkspaceSnapshotGraphResult,
        },
        node_weight::traits::SiNodeWeight,
    },
    EdgeWeightKindDiscriminants,
};

use super::WorkspaceSnapshotGraphV4;

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

        for change in changes {
            let mut explicit_approval_requirement_definition_ids = Vec::new();
            let mut virtual_approval_requirement_rules = Vec::new();

            // Check if the node exists in the current graph. If it does, we are working with an
            // addition or a modification. If it does not, we are working with a removal.
            if let Some(entity_node_index) = self.get_node_index_by_id_opt(change.entity_id) {
                for (_, _, requirement_node_index) in self.edges_directed_for_edge_weight_kind(
                    entity_node_index,
                    Direction::Outgoing,
                    EdgeWeightKindDiscriminants::HasApprovalRequirement,
                ) {
                    let requirement_node_weight = self
                        .get_node_weight(requirement_node_index)?
                        .get_approval_requirement_definition_node_weight()?;
                    explicit_approval_requirement_definition_ids
                        .push(requirement_node_weight.id().into());
                }

                // After processing all explicit approval requirements from approval requirement
                // definitions, let's check if we need to add virtual requirements.
                match change.entity_kind {
                    // TODO(nick,jacob): replace this hard-coded virtual rule with an explicit definition
                    // the schema variant category.
                    EntityKind::SchemaVariant
                        if explicit_approval_requirement_definition_ids.is_empty() =>
                    {
                        virtual_approval_requirement_rules.push(ApprovalRequirementRule {
                            entity_id: change.entity_id,
                            entity_kind: change.entity_kind,
                            minimum: 1,
                            approvers: HashSet::from([
                                ApprovalRequirementApprover::PermissionLookup(
                                    ApprovalRequirementPermissionLookup {
                                        object_type: "workspace".to_string(),
                                        object_id: workspace_id.to_string(),
                                        permission: "approve".to_string(),
                                    },
                                ),
                            ]),
                        });
                    }
                    // For any changes to explicit approval requirements, we need approvals from
                    // workspace approvers.
                    EntityKind::ApprovalRequirementDefinition => {
                        virtual_approval_requirement_rules.push(ApprovalRequirementRule {
                            entity_id: change.entity_id,
                            entity_kind: change.entity_kind,
                            minimum: 1,
                            approvers: HashSet::from([
                                ApprovalRequirementApprover::PermissionLookup(
                                    ApprovalRequirementPermissionLookup {
                                        object_type: "workspace".to_string(),
                                        object_id: workspace_id.to_string(),
                                        permission: "approve".to_string(),
                                    },
                                ),
                            ]),
                        });
                    }
                    _ => {}
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
}
