use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_trait::async_trait;
use si_events::{merkle_tree_hash::MerkleTreeHash, ContentHash};
use si_id::{ulid::Ulid, ApprovalRequirementDefinitionId, EntityId, UserPk};

use crate::{
    approval_requirement::{
        ApprovalRequirement, ApprovalRequirementDefinition, ApprovalRequirementExplicit,
    },
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
    async fn new_definition(
        &self,
        ctx: &DalContext,
        entity_id: Ulid,
        minimum_approvers_count: usize,
        approvers: HashSet<ApprovalRequirementApprover>,
    ) -> WorkspaceSnapshotResult<ApprovalRequirementDefinitionId>;

    async fn remove_definition(
        &self,
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<()>;

    async fn add_individual_approver_for_definition(
        &self,
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
        user_id: UserPk,
    ) -> WorkspaceSnapshotResult<()>;

    async fn remove_individual_approver_for_definition(
        &self,
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
        user_id: UserPk,
    ) -> WorkspaceSnapshotResult<()>;

    async fn approval_requirements_for_changes(
        &self,
        ctx: &DalContext,
        changes: &[Change],
    ) -> WorkspaceSnapshotResult<(Vec<ApprovalRequirement>, HashMap<EntityId, MerkleTreeHash>)>;

    async fn approval_requirement_definitions_for_entity_id_opt(
        &self,
        ctx: &DalContext,
        entity_id: EntityId,
    ) -> WorkspaceSnapshotResult<Option<Vec<ApprovalRequirementDefinition>>>;

    async fn entity_id_for_approval_requirement_definition_id(
        &self,
        id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<EntityId>;

    async fn get_approval_requirement_definition_by_id(
        &self,
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<ApprovalRequirementDefinition>;
}

#[async_trait]
impl ApprovalRequirementExt for WorkspaceSnapshot {
    async fn get_approval_requirement_definition_by_id(
        &self,
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<ApprovalRequirementDefinition> {
        let node_weight = self
            .get_node_weight_by_id(id)
            .await?
            .get_approval_requirement_definition_node_weight()?;
        let content: ApprovalRequirementDefinitionContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&node_weight.content_hash())
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        // This should always expect the newest version since we migrate the world when we perform graph migrations.
        let ApprovalRequirementDefinitionContent::V1(inner) = content;
        Ok(ApprovalRequirementDefinition::assemble(id, inner))
    }
    async fn new_definition(
        &self,
        ctx: &DalContext,
        entity_id: Ulid,
        minimum_approvers_count: usize,
        approvers: HashSet<ApprovalRequirementApprover>,
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
            EdgeWeight::new(EdgeWeightKind::ApprovalRequirementDefinition),
            id,
        )
        .await?;

        Ok(id.into())
    }

    async fn remove_definition(
        &self,
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<()> {
        self.remove_node_by_id(approval_requirement_definition_id)
            .await
    }

    async fn add_individual_approver_for_definition(
        &self,
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
        user_id: UserPk,
    ) -> WorkspaceSnapshotResult<()> {
        let node_weight = self.get_node_weight_by_id(id).await?;
        let content: ApprovalRequirementDefinitionContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&node_weight.content_hash())
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        // This should always expect the newest version since we migrate the world when we perform graph migrations.
        let ApprovalRequirementDefinitionContent::V1(mut inner) = content;

        // Only update the content store and node if the approver wasn't already in the set.
        if inner
            .approvers
            .insert(ApprovalRequirementApprover::User(user_id))
        {
            let (hash, _) = ctx.layer_db().cas().write(
                Arc::new(ApprovalRequirementDefinitionContent::V1(inner).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;

            ctx.workspace_snapshot()?
                .update_content(id.into(), hash)
                .await?;
        }

        Ok(())
    }

    async fn remove_individual_approver_for_definition(
        &self,
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
        user_id: UserPk,
    ) -> WorkspaceSnapshotResult<()> {
        let node_weight = self.get_node_weight_by_id(id).await?;
        let content: ApprovalRequirementDefinitionContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&node_weight.content_hash())
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        // This should always expect the newest version since we migrate the world when we perform graph migrations.
        let ApprovalRequirementDefinitionContent::V1(mut inner) = content;

        // Only update the content store and node if the approver already existed in the set.
        if inner
            .approvers
            .remove(&ApprovalRequirementApprover::User(user_id))
        {
            let (hash, _) = ctx.layer_db().cas().write(
                Arc::new(ApprovalRequirementDefinitionContent::V1(inner).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;

            ctx.workspace_snapshot()?
                .update_content(id.into(), hash)
                .await?;
        }

        Ok(())
    }

    async fn approval_requirements_for_changes(
        &self,
        ctx: &DalContext,
        changes: &[Change],
    ) -> WorkspaceSnapshotResult<(Vec<ApprovalRequirement>, HashMap<EntityId, MerkleTreeHash>)>
    {
        let mut results = Vec::new();

        let workspace_id = ctx.workspace_pk()?;
        let (bags, ids_with_hashes_for_deleted_nodes) = self
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
                    .map(ApprovalRequirement::Virtual),
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

        Ok((results, ids_with_hashes_for_deleted_nodes))
    }

    async fn approval_requirement_definitions_for_entity_id_opt(
        &self,
        ctx: &DalContext,
        entity_id: EntityId,
    ) -> WorkspaceSnapshotResult<Option<Vec<ApprovalRequirementDefinition>>> {
        let Some(approval_requirement_definition_ids) = self
            .working_copy()
            .await
            .approval_requirement_definitions_for_entity_id_opt(entity_id)?
        else {
            return Ok(None);
        };

        let mut results = Vec::new();
        for approval_requirement_definition_id in approval_requirement_definition_ids {
            let definition_node_weight = self
                .working_copy()
                .await
                .get_node_weight_by_id(approval_requirement_definition_id)?
                .get_approval_requirement_definition_node_weight()?;
            let Some(ApprovalRequirementDefinitionContent::V1(definition_content)) = ctx
                .layer_db()
                .cas()
                .try_read_as(&definition_node_weight.content_hash())
                .await?
            else {
                return Err(WorkspaceSnapshotError::MissingContentFromStore(
                    definition_node_weight.id(),
                ));
            };
            results.push(ApprovalRequirementDefinition::assemble(
                approval_requirement_definition_id,
                definition_content,
            ));
        }

        Ok(Some(results))
    }

    async fn entity_id_for_approval_requirement_definition_id(
        &self,
        id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<EntityId> {
        Ok(self
            .working_copy()
            .await
            .entity_id_for_approval_requirement(id)?)
    }
}
