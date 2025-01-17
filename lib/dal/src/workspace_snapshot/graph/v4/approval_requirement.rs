use petgraph::Direction;
use si_events::workspace_snapshot::EntityKind;
use si_id::{ApprovalRequirementDefinitionId, WorkspacePk};

use crate::{
    workspace_snapshot::{
        graph::{
            detector::Change,
            traits::{
                approval_requirement::{
                    ApprovalRequirementApprover, ApprovalRequirementExt,
                    ApprovalRequirementPermissionLookup, ApprovalRequirementRule,
                    ApprovalRequirementsBag,
                },
                entity_kind::EntityKindExt,
            },
            WorkspaceSnapshotGraphResult,
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
    ) -> WorkspaceSnapshotGraphResult<Vec<ApprovalRequirementsBag>> {
        let mut requirements = Vec::new();

        for change in changes {
            let mut explicit_approval_requirement_definition_ids = Vec::new();
            let mut virtual_approval_requirement_rules = Vec::new();

            let entity_id = change.id.into();
            let entity_kind = self.get_entity_kind_for_id(entity_id)?;
            let entity_node_index = self.get_node_index_by_id(entity_id)?;

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

            // Add virtual approval requirements, as needed.
            match entity_kind {
                // TODO(nick,jacob): replace this hard-coded virtual rule with an explicit definition
                // the schema variant category.
                EntityKind::SchemaVariant
                    if explicit_approval_requirement_definition_ids.is_empty() =>
                {
                    virtual_approval_requirement_rules.push(ApprovalRequirementRule {
                        entity_id,
                        entity_kind,
                        minimum: 1,
                        approvers: vec![ApprovalRequirementApprover::PermissionLookup(
                            ApprovalRequirementPermissionLookup {
                                object_type: "workspace".to_string(),
                                object_id: workspace_id.to_string(),
                                permission: "approve".to_string(),
                            },
                        )],
                    });
                }
                // For any changes to explicit approval requirements, we need approvals from
                // workspace approvers.
                EntityKind::ApprovalRequirementDefinition => {
                    virtual_approval_requirement_rules.push(ApprovalRequirementRule {
                        entity_id,
                        entity_kind,
                        minimum: 1,
                        approvers: vec![ApprovalRequirementApprover::PermissionLookup(
                            ApprovalRequirementPermissionLookup {
                                object_type: "workspace".to_string(),
                                object_id: workspace_id.to_string(),
                                permission: "approve".to_string(),
                            },
                        )],
                    });
                }
                _ => {}
            }

            requirements.push(ApprovalRequirementsBag {
                entity_id,
                entity_kind,
                explicit_approval_requirement_definition_ids,
                virtual_approval_requirement_rules,
            });
        }

        Ok(requirements)
    }
}
