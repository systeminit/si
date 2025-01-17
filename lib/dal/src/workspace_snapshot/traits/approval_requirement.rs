use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use si_events::ContentHash;
use si_id::{ulid::Ulid, ApprovalRequirementDefinitionId};

use crate::{
    approval_requirement::{ApprovalRequirement, ApprovalRequirementExplicit},
    layer_db_types::{
        ApprovalRequirementDefinitionContent, ApprovalRequirementDefinitionContentV1,
    },
    workspace_snapshot::{
        graph::{
            detector::Change,
            traits::approval_requirement::ApprovalRequirementExt as ApprovalRequirementExtGraph,
        },
        node_weight::{traits::SiNodeWeight, NodeWeight},
        WorkspaceSnapshotResult,
    },
    DalContext, EdgeWeight, EdgeWeightKind, WorkspaceSnapshot, WorkspaceSnapshotError,
};

pub use crate::workspace_snapshot::graph::traits::approval_requirement::{
    ApprovalRequirementApprover, ApprovalRequirementRule,
};

#[async_trait]
pub trait ApprovalRequirementExt {
    async fn new_approval_requirement_definition(
        &self,
        ctx: &DalContext,
        entity_id: Ulid,
        minimum_approvers_count: usize,
        approvers: Vec<ApprovalRequirementApprover>,
    ) -> WorkspaceSnapshotResult<ApprovalRequirementDefinitionId>;

    async fn approval_requirements_for_changes(
        &self,
        ctx: &DalContext,
        changes: &[Change],
    ) -> WorkspaceSnapshotResult<Vec<ApprovalRequirement>>;
}

#[async_trait]
impl ApprovalRequirementExt for WorkspaceSnapshot {
    async fn new_approval_requirement_definition(
        &self,
        ctx: &DalContext,
        entity_id: Ulid,
        minimum_approvers_count: usize,
        approvers: Vec<ApprovalRequirementApprover>,
    ) -> WorkspaceSnapshotResult<ApprovalRequirementDefinitionId> {
        let content = ApprovalRequirementDefinitionContentV1 {
            minimum: minimum_approvers_count,
            approvers,
        };

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(ApprovalRequirementDefinitionContent::V1(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let id = self.generate_ulid().await?;
        let lineage_id = self.generate_ulid().await?;
        let node_weight = NodeWeight::new_approval_requirement_definition(id, lineage_id, hash);
        self.add_or_replace_node(node_weight).await?;

        self.add_edge(
            entity_id,
            EdgeWeight::new(EdgeWeightKind::HasApprovalRequirement),
            id,
        )
        .await?;

        Ok(id.into())
    }

    async fn approval_requirements_for_changes(
        &self,
        ctx: &DalContext,
        changes: &[Change],
    ) -> WorkspaceSnapshotResult<Vec<ApprovalRequirement>> {
        let mut results = Vec::new();

        let workspace_id = ctx.workspace_pk()?;
        let bags = self
            .working_copy()
            .await
            .approval_requirements_for_changes(workspace_id, changes)?;

        let mut cache = HashMap::new();
        for bag in bags {
            // For the explicit requirements, build a cache of hashes.
            for approval_requirement_definition_id in
                bag.explicit_approval_requirement_definition_ids
            {
                let requirement_node_weight = self
                    .working_copy()
                    .await
                    .get_node_weight_by_id(approval_requirement_definition_id)?
                    .get_approval_requirement_definition_node_weight()?;
                let hash = requirement_node_weight.content_hash();
                cache.insert(
                    hash,
                    (
                        approval_requirement_definition_id,
                        bag.entity_id,
                        bag.entity_kind,
                    ),
                );
            }

            // For the virtual requirements, add them directly to our requirements list.
            results.extend(
                bag.virtual_approval_requirement_rules
                    .iter()
                    .cloned()
                    .map(|v| ApprovalRequirement::Virtual(v)),
            );
        }

        // From the cache of hashes, perform one bulk retrieval from CAS.
        let hashes: Vec<ContentHash> = Vec::from_iter(cache.keys().cloned());
        let content_map: HashMap<ContentHash, ApprovalRequirementDefinitionContent> =
            ctx.layer_db().cas().try_read_many_as(&hashes).await?;

        // With both the content map and cache in hand, we can assemble all of the explicit
        // requirements.
        for (hash, (approval_requirement_definition_id, entity_id, entity_kind)) in cache {
            if let Some(content) = content_map.get(&hash) {
                // NOTE(nick): if we had a v2, then there would be migration logic here.
                let ApprovalRequirementDefinitionContent::V1(inner) = content;

                results.push(ApprovalRequirement::Explicit(ApprovalRequirementExplicit {
                    id: approval_requirement_definition_id,
                    rule: ApprovalRequirementRule {
                        entity_id,
                        entity_kind,
                        minimum: inner.minimum,
                        approvers: inner.approvers.to_owned(),
                    },
                }));
            } else {
                return Err(WorkspaceSnapshotError::MissingContentFromContentMap(
                    hash,
                    approval_requirement_definition_id,
                ));
            }
        }

        Ok(results)
    }
}
