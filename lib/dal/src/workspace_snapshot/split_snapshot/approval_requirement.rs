use std::{
    collections::{
        BTreeSet,
        HashMap,
        HashSet,
    },
    sync::Arc,
};

use async_trait::async_trait;
use petgraph::Direction::{
    Incoming,
    Outgoing,
};
use si_events::{
    ContentHash,
    merkle_tree_hash::MerkleTreeHash,
    workspace_snapshot::{
        Change,
        EntityKind,
    },
};
use si_id::{
    ApprovalRequirementDefinitionId,
    EntityId,
    UserPk,
    ulid::Ulid,
};
use si_split_graph::{
    CustomNodeWeight as _,
    SplitGraphResult,
};

use super::{
    SplitSnapshot,
    SplitSnapshotGraphV1,
};
use crate::{
    DalContext,
    EdgeWeight,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    NodeWeightDiscriminants,
    WorkspaceSnapshotError,
    approval_requirement::{
        ApprovalRequirement,
        ApprovalRequirementApprover,
        ApprovalRequirementDefinition,
        ApprovalRequirementExplicit,
        ApprovalRequirementRule,
    },
    layer_db_types::{
        ApprovalRequirementDefinitionContent,
        ApprovalRequirementDefinitionContentV1,
    },
    workspace_snapshot::{
        self,
        WorkspaceSnapshotResult,
        graph::{
            self,
            traits::approval_requirement::ApprovalRequirementsBag,
            v4::approval_requirement::new_virtual_requirement_rule,
        },
        node_weight::{
            NodeWeight,
            traits::SiVersionedNodeWeight,
        },
    },
};

#[async_trait]
impl workspace_snapshot::traits::approval_requirement::ApprovalRequirementExt for SplitSnapshot {
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
        let node_weight = self.get_node_weight(id).await?;
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
        let node_weight = self.get_node_weight(id).await?;
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
        let (bags, ids_with_hashes_for_deleted_nodes) =
            approval_requirements_for_changes(&*self.working_copy().await, workspace_id, changes)?;

        let mut cache = HashMap::new();
        for bag in bags {
            // For the explicit requirements, build a cache of hashes.
            for approval_requirement_definition_id in
                bag.explicit_approval_requirement_definition_ids
            {
                let requirement_node_weight = self
                    .get_node_weight(approval_requirement_definition_id)
                    .await?
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
                // NOTE(nick): if we had a v2, then there would be migration logic here
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
        if !self.node_exists(entity_id).await {
            return Ok(None);
        }

        let working_copy = self.working_copy().await;

        let mut results = vec![];
        for requirement_node_id in working_copy.outgoing_targets(
            entity_id.into(),
            EdgeWeightKindDiscriminants::ApprovalRequirementDefinition,
        )? {
            let Some(node_weight) = working_copy.node_weight(requirement_node_id) else {
                continue;
            };

            let Some(ApprovalRequirementDefinitionContent::V1(definition_content)) = ctx
                .layer_db()
                .cas()
                .try_read_as(&node_weight.content_hash())
                .await?
            else {
                return Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ));
            };

            results.push(ApprovalRequirementDefinition::assemble(
                requirement_node_id.into(),
                definition_content,
            ));
        }

        Ok(Some(results))
    }

    async fn entity_id_for_approval_requirement_definition_id(
        &self,
        id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<EntityId> {
        if !self.node_exists(id).await {
            return Err(WorkspaceSnapshotError::EntityNotFoundForApprovalRequirementDefinition(id));
        }

        let working_copy = self.working_copy().await;
        let maybe_entity_id = working_copy
            .incoming_sources(
                id.into(),
                EdgeWeightKindDiscriminants::ApprovalRequirementDefinition,
            )?
            .next()
            .map(Into::into);

        Ok(maybe_entity_id
            .ok_or(WorkspaceSnapshotError::EntityNotFoundForApprovalRequirementDefinition(id))?)
    }

    async fn get_approval_requirement_definition_by_id(
        &self,
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<ApprovalRequirementDefinition> {
        let node_weight = self
            .get_node_weight(id)
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
}

pub fn approval_requirements_for_changes(
    graph: &SplitSnapshotGraphV1,
    workspace_id: si_id::WorkspacePk,
    changes: &[Change],
) -> SplitGraphResult<(
    Vec<workspace_snapshot::graph::traits::approval_requirement::ApprovalRequirementsBag>,
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
            }

            // If there is a change involving an Action, and we can determine the Component for the Action,
            // We want to treat the Views that Component is in as changed to have them generate approval
            // requirements.
            EntityKind::Action => {
                let action_id = change.entity_id.into();
                if !graph.node_exists(action_id) {
                    // The Action has been removed, and we no longer have access to the base graph where it did exist
                    // at this point, so let the default virtual requirement handling take care of this Action.
                    continue;
                }

                let mut maybe_component_node_id = None;
                for edge_ref in graph.edges_directed(action_id, Outgoing)? {
                    let Some(target_node_weight) = graph.node_weight(edge_ref.target()) else {
                        continue;
                    };
                    if NodeWeightDiscriminants::Component == target_node_weight.kind() {
                        maybe_component_node_id = Some(edge_ref.target());
                        break;
                    }
                }

                if let Some(component_id) = maybe_component_node_id {
                    let view_ids = list_views_for_component_id(graph, component_id.into())?;
                    for view_id in view_ids {
                        let Some(view_node_weight) = graph.node_weight(view_id.into()) else {
                            continue;
                        };

                        changes_to_add.push(Change {
                            entity_id: view_id.into_inner().into(),
                            entity_kind: EntityKind::View,
                            merkle_tree_hash: view_node_weight.merkle_tree_hash(),
                        });
                    }

                    change_idxs_to_remove.push(change_idx);
                }
            }

            _ => {}
        }
    }

    let mut local_changes = changes.to_vec();
    for change_idx_to_remove in change_idxs_to_remove.iter().rev() {
        local_changes.remove(*change_idx_to_remove);
    }

    changes_to_add.retain(|change| !modified_view_ids.contains(&change.entity_id));
    local_changes.extend(changes_to_add);

    for change in &local_changes {
        let mut explicit_approval_requirement_definition_ids = Vec::new();
        let mut virtual_approval_requirement_rules = Vec::new();

        // Check if the node exists in the current graph. If it does, we are working with an
        // addition or a modification. If it does not, we are working with a removal.
        if graph.node_exists(change.entity_id.into()) {
            for requirement_node_id in graph.outgoing_targets(
                change.entity_id.into(),
                EdgeWeightKindDiscriminants::ApprovalRequirementDefinition,
            )? {
                explicit_approval_requirement_definition_ids.push(requirement_node_id.into());
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
            ids_with_hashes_for_deleted_nodes.insert(change.entity_id, change.merkle_tree_hash);

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
                        graph::traits::approval_requirement::ApprovalRequirementPermissionLookup {
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

pub fn list_views_for_component_id(
    graph: &SplitSnapshotGraphV1,
    component_id: si_id::ComponentId,
) -> SplitGraphResult<Vec<si_id::ViewId>> {
    if !graph.node_exists(component_id.into()) {
        return Ok(vec![]);
    }

    let mut view_ids_set = BTreeSet::new();

    for represents_edge_ref in graph.edges_directed_for_edge_weight_kind(
        component_id.into(),
        Incoming,
        EdgeWeightKindDiscriminants::Represents,
    )? {
        if let Some(view_id) = graph.directed_unique_neighbor_of_edge_weight_kind(
            represents_edge_ref.source(),
            Incoming,
            EdgeWeightKindDiscriminants::Use,
        )? {
            view_ids_set.insert(view_id);
        }
    }

    Ok(view_ids_set.into_iter().map(Into::into).collect())
}
