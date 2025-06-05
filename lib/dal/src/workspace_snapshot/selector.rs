use std::{
    collections::{
        HashMap,
        HashSet,
    },
    sync::Arc,
};

use async_trait::async_trait;
use petgraph::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    WorkspaceSnapshotAddress,
    merkle_tree_hash::MerkleTreeHash,
    workspace_snapshot::{
        Change,
        EntityKind,
    },
};
use si_id::{
    ApprovalRequirementDefinitionId,
    AttributeValueId,
    ComponentId,
    EntityId,
    InputSocketId,
    PropId,
    SchemaId,
    SchemaVariantId,
    UserPk,
    ViewId,
    ulid::Ulid,
};
use strum::{
    EnumDiscriminants,
    EnumString,
};

use super::{
    CycleCheckGuard,
    DependentValueRoot,
    EntityKindExt,
    InferredConnectionsWriteGuard,
    InputSocketExt,
    SchemaVariantExt,
    WorkspaceSnapshot,
    WorkspaceSnapshotError,
    WorkspaceSnapshotResult,
    graph::LineageId,
    node_weight::{
        NodeWeight,
        category_node_weight::CategoryNodeKind,
    },
    split_snapshot::SplitSnapshot,
    traits::{
        approval_requirement::ApprovalRequirementExt,
        diagram::view::ViewExt,
        prop::PropExt,
    },
};
use crate::{
    DalContext,
    EdgeWeight,
    EdgeWeightKindDiscriminants,
    InputSocket,
    approval_requirement::{
        ApprovalRequirement,
        ApprovalRequirementApprover,
        ApprovalRequirementDefinition,
    },
    component::ComponentResult,
    prop::PropResult,
};

/// The `WorkspaceSnapshotSelector` enum acts as a dispatcher for workspace snapshot methods
/// between the legacy `WorkspaceSnapshot` and the new `SplitSnapshot` implementations.
///
/// This approach provides a unified interface to work with both snapshot types without
/// requiring dynamic dispatch through trait objects (`dyn Trait`). Instead, it uses
/// pattern matching on the enum variants to route method calls to the appropriate
/// implementation, which is theoretically more performant than dynamic dispatch / v-table.
///
/// By wrapping both snapshot types in a single enum, code that interacts with workspace
/// snapshots can remain implementation-agnostic and work with either snapshot type
/// without needing to be aware of the underlying differences.
#[derive(Clone, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize, EnumString))]
pub enum WorkspaceSnapshotSelector {
    LegacySnapshot(Arc<WorkspaceSnapshot>),
    SplitSnapshot(Arc<SplitSnapshot>),
}

impl WorkspaceSnapshotSelector {
    pub async fn address(&self) -> WorkspaceSnapshotAddress {
        match self {
            Self::LegacySnapshot(snap) => snap.id().await,
            Self::SplitSnapshot(snap) => snap.id().await,
        }
    }

    pub fn as_legacy_snapshot(&self) -> WorkspaceSnapshotResult<Arc<WorkspaceSnapshot>> {
        match self {
            Self::LegacySnapshot(snap) => Ok(snap.clone()),
            _ => Err(WorkspaceSnapshotError::UnexpectedSnapshotKind(self.into())),
        }
    }

    pub fn as_split_snapshot(&self) -> WorkspaceSnapshotResult<Arc<SplitSnapshot>> {
        match self {
            Self::SplitSnapshot(snap) => Ok(snap.clone()),
            _ => Err(WorkspaceSnapshotError::UnexpectedSnapshotKind(self.into())),
        }
    }

    pub async fn write(
        &self,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotResult<WorkspaceSnapshotAddress> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.write(ctx).await,
            Self::SplitSnapshot(snapshot) => snapshot.write(ctx).await,
        }
    }

    pub async fn subgraph_count(&self) -> usize {
        match self {
            Self::LegacySnapshot(_) => 1,
            Self::SplitSnapshot(snapshot) => snapshot.subgraph_count().await,
        }
    }

    pub async fn id(&self) -> WorkspaceSnapshotAddress {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.id().await,
            Self::SplitSnapshot(snapshot) => snapshot.id().await,
        }
    }

    pub async fn root(&self) -> WorkspaceSnapshotResult<Ulid> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.root().await,
            Self::SplitSnapshot(snapshot) => snapshot.root().await,
        }
    }

    pub async fn generate_ulid(&self) -> WorkspaceSnapshotResult<Ulid> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.generate_ulid().await,
            Self::SplitSnapshot(_) => Ok(Ulid::new()),
        }
    }

    pub async fn enable_cycle_check(&self) -> CycleCheckGuard {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.enable_cycle_check().await,
            Self::SplitSnapshot(snapshot) => snapshot.enable_cycle_check().await,
        }
    }

    pub async fn disable_cycle_check(&self) {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.disable_cycle_check().await,
            Self::SplitSnapshot(snapshot) => snapshot.disable_cycle_check().await,
        }
    }

    pub async fn cycle_check(&self) -> bool {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.cycle_check().await,
            Self::SplitSnapshot(snapshot) => snapshot.cycle_check().await,
        }
    }

    pub async fn is_acyclic_directed(&self) -> bool {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.is_acyclic_directed().await,
            Self::SplitSnapshot(snapshot) => snapshot.is_acyclic_directed().await,
        }
    }

    pub async fn add_or_replace_node(&self, node: NodeWeight) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.add_or_replace_node(node).await,
            Self::SplitSnapshot(snapshot) => snapshot.add_or_replace_node(node).await,
        }
    }

    pub async fn add_ordered_node(&self, node: NodeWeight) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.add_ordered_node(node).await,
            Self::SplitSnapshot(snapshot) => snapshot.add_ordered_node(node).await,
        }
    }

    pub async fn update_content(
        &self,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.update_content(id, new_content_hash).await,
            Self::SplitSnapshot(snapshot) => snapshot.update_content(id, new_content_hash).await,
        }
    }

    pub async fn add_edge(
        &self,
        from_node_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_node_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .add_edge(from_node_id, edge_weight, to_node_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .add_edge(from_node_id, edge_weight, to_node_id)
                    .await
            }
        }
    }

    pub async fn add_ordered_edge(
        &self,
        from_node_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_node_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .add_ordered_edge(from_node_id, edge_weight, to_node_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .add_ordered_edge(from_node_id, edge_weight, to_node_id)
                    .await
            }
        }
    }

    pub async fn import_component_subgraph(
        &self,
        other: &WorkspaceSnapshotSelector,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                let other = other.as_legacy_snapshot()?;
                snapshot
                    .import_component_subgraph(&other, component_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                let other = other.as_split_snapshot()?;
                snapshot
                    .import_component_subgraph(&other, component_id)
                    .await
            }
        }
    }

    pub async fn get_node_weight(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<NodeWeight> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.get_node_weight(id).await,
            Self::SplitSnapshot(snapshot) => snapshot.get_node_weight(id).await,
        }
    }

    pub async fn get_node_weight_opt(&self, id: impl Into<Ulid>) -> Option<NodeWeight> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.get_node_weight_opt(id).await,
            Self::SplitSnapshot(snapshot) => snapshot.get_node_weight_opt(id).await,
        }
    }

    pub async fn cleanup(&self) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.cleanup().await,
            Self::SplitSnapshot(snapshot) => snapshot.cleanup().await,
        }
    }

    pub async fn cleanup_and_merkle_tree_hash(&self) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.cleanup_and_merkle_tree_hash().await,
            Self::SplitSnapshot(snapshot) => snapshot.cleanup_and_merkle_tree_hash().await,
        }
    }

    pub async fn nodes(&self) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.nodes().await,
            Self::SplitSnapshot(snapshot) => snapshot.nodes().await,
        }
    }

    pub async fn edges(&self) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.edges().await,
            Self::SplitSnapshot(snapshot) => snapshot.edges().await,
        }
    }

    pub async fn node_exists(&self, id: impl Into<Ulid>) -> bool {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.node_exists(id).await,
            Self::SplitSnapshot(snapshot) => snapshot.node_exists(id).await,
        }
    }

    pub async fn get_category_node_or_err(
        &self,
        source: Option<Ulid>,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<Ulid> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.get_category_node_or_err(source, kind).await,
            Self::SplitSnapshot(snapshot) => snapshot.get_category_node_or_err(source, kind).await,
        }
    }

    pub async fn get_category_node(
        &self,
        source: Option<Ulid>,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<Option<Ulid>> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.get_category_node(source, kind).await,
            Self::SplitSnapshot(snapshot) => snapshot.get_category_node(source, kind).await,
        }
    }

    pub async fn edges_directed(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
    ) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.edges_directed(id, direction).await,
            Self::SplitSnapshot(snapshot) => snapshot.edges_directed(id, direction).await,
        }
    }

    /// Get edges for a specific direction and edge weight kind
    /// (edge_weight, source_id, target_id)
    pub async fn edges_directed_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .edges_directed_for_edge_weight_kind(id, direction, edge_kind)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .edges_directed_for_edge_weight_kind(id, direction, edge_kind)
                    .await
            }
        }
    }

    pub async fn remove_all_edges(&self, id: impl Into<Ulid>) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.remove_all_edges(id).await,
            Self::SplitSnapshot(snapshot) => snapshot.remove_all_edges(id).await,
        }
    }

    pub async fn incoming_sources_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: impl Into<EdgeWeightKindDiscriminants>,
    ) -> WorkspaceSnapshotResult<Vec<Ulid>> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .incoming_sources_for_edge_weight_kind(id, edge_weight_kind_discrim.into())
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .incoming_sources_for_edge_weight_kind(id, edge_weight_kind_discrim.into())
                    .await
            }
        }
    }

    pub async fn source_opt(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: impl Into<EdgeWeightKindDiscriminants>,
    ) -> WorkspaceSnapshotResult<Option<Ulid>> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .source_opt(id, edge_weight_kind_discrim.into())
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .source_opt(id, edge_weight_kind_discrim.into())
                    .await
            }
        }
    }

    pub async fn outgoing_targets_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: impl Into<EdgeWeightKindDiscriminants>,
    ) -> WorkspaceSnapshotResult<Vec<Ulid>> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .outgoing_targets_for_edge_weight_kind(id, edge_weight_kind_discrim.into())
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .outgoing_targets_for_edge_weight_kind(id, edge_weight_kind_discrim.into())
                    .await
            }
        }
    }

    pub async fn all_outgoing_targets(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.all_outgoing_targets(id).await,
            Self::SplitSnapshot(snapshot) => snapshot.all_outgoing_targets(id).await,
        }
    }

    pub async fn all_incoming_sources(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.all_incoming_sources(id).await,
            Self::SplitSnapshot(snapshot) => snapshot.all_incoming_sources(id).await,
        }
    }

    pub async fn remove_incoming_edges_of_kind(
        &self,
        target_id: impl Into<Ulid>,
        kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .remove_incoming_edges_of_kind(target_id, kind)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .remove_incoming_edges_of_kind(target_id, kind)
                    .await
            }
        }
    }

    pub async fn remove_outgoing_edges_of_kind(
        &self,
        source_id: impl Into<Ulid>,
        kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .remove_outgoing_edges_of_kind(source_id, kind)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .remove_outgoing_edges_of_kind(source_id, kind)
                    .await
            }
        }
    }

    pub async fn get_edges_between_nodes(
        &self,
        from_node_id: Ulid,
        to_node_id: Ulid,
    ) -> WorkspaceSnapshotResult<Vec<EdgeWeight>> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .get_edges_between_nodes(from_node_id, to_node_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .get_edges_between_nodes(from_node_id, to_node_id)
                    .await
            }
        }
    }

    pub async fn remove_node_by_id(&self, id: impl Into<Ulid>) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.remove_node_by_id(id).await,
            Self::SplitSnapshot(snapshot) => snapshot.remove_node_by_id(id).await,
        }
    }

    pub async fn remove_edge(
        &self,
        source_id: impl Into<Ulid>,
        target_id: impl Into<Ulid>,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot.remove_edge(source_id, target_id, edge_kind).await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot.remove_edge(source_id, target_id, edge_kind).await
            }
        }
    }

    pub async fn find_edge(
        &self,
        from_id: impl Into<Ulid>,
        to_id: impl Into<Ulid>,
        edge_weight_kind: EdgeWeightKindDiscriminants,
    ) -> Option<EdgeWeight> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot.find_edge(from_id, to_id, edge_weight_kind).await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot.find_edge(from_id, to_id, edge_weight_kind).await
            }
        }
    }

    /// If the `new_id` for the node is the id of an existing node weight (for example if you
    /// are creating a new thing and want to "swap it" back in for the old thing), then you
    /// *MUST* remove the existing node *before* issuing the update_node_id call.
    pub async fn update_node_id(
        &self,
        current_id: impl Into<Ulid>,
        new_id: impl Into<Ulid>,
        new_lineage_id: LineageId,
    ) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .update_node_id(current_id, new_id, new_lineage_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .update_node_id(current_id, new_id, new_lineage_id)
                    .await
            }
        }
    }

    pub async fn ordered_children_for_node(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<Vec<Ulid>>> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.ordered_children_for_node(id).await,
            Self::SplitSnapshot(snapshot) => snapshot.ordered_children_for_node(id).await,
        }
    }

    pub async fn dvu_root_check(&self, root: DependentValueRoot) -> bool {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.dvu_root_check(root).await,
            Self::SplitSnapshot(snapshot) => snapshot.dvu_root_check(root).await,
        }
    }

    pub async fn schema_variant_id_for_component_id(
        &self,
        component_id: ComponentId,
    ) -> ComponentResult<SchemaVariantId> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .schema_variant_id_for_component_id(component_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .schema_variant_id_for_component_id(component_id)
                    .await
            }
        }
    }

    pub async fn frame_contains_components(
        &self,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<ComponentId>> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot.frame_contains_components(component_id).await
            }
            Self::SplitSnapshot(snapshot) => snapshot.frame_contains_components(component_id).await,
        }
    }

    pub async fn inferred_connection_graph(
        &self,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotResult<InferredConnectionsWriteGuard<'_>> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.inferred_connection_graph(ctx).await,
            Self::SplitSnapshot(snapshot) => snapshot.inferred_connection_graph(ctx).await,
        }
    }

    pub async fn clear_inferred_connection_graph(&self) {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.clear_inferred_connection_graph().await,
            Self::SplitSnapshot(snapshot) => snapshot.clear_inferred_connection_graph().await,
        }
    }

    pub async fn revert(&self) {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.revert().await,
            Self::SplitSnapshot(snapshot) => snapshot.revert().await,
        }
    }
}

#[async_trait]
impl ApprovalRequirementExt for WorkspaceSnapshotSelector {
    async fn new_definition(
        &self,
        ctx: &DalContext,
        entity_id: Ulid,
        minimum_approvers_count: usize,
        approvers: HashSet<ApprovalRequirementApprover>,
    ) -> WorkspaceSnapshotResult<ApprovalRequirementDefinitionId> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .new_definition(ctx, entity_id, minimum_approvers_count, approvers)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .new_definition(ctx, entity_id, minimum_approvers_count, approvers)
                    .await
            }
        }
    }

    async fn remove_definition(
        &self,
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .remove_definition(approval_requirement_definition_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .remove_definition(approval_requirement_definition_id)
                    .await
            }
        }
    }

    async fn add_individual_approver_for_definition(
        &self,
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
        user_id: UserPk,
    ) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .add_individual_approver_for_definition(ctx, id, user_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .add_individual_approver_for_definition(ctx, id, user_id)
                    .await
            }
        }
    }

    async fn remove_individual_approver_for_definition(
        &self,
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
        user_id: UserPk,
    ) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .remove_individual_approver_for_definition(ctx, id, user_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .remove_individual_approver_for_definition(ctx, id, user_id)
                    .await
            }
        }
    }

    async fn approval_requirements_for_changes(
        &self,
        ctx: &DalContext,
        changes: &[Change],
    ) -> WorkspaceSnapshotResult<(Vec<ApprovalRequirement>, HashMap<EntityId, MerkleTreeHash>)>
    {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .approval_requirements_for_changes(ctx, changes)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .approval_requirements_for_changes(ctx, changes)
                    .await
            }
        }
    }

    async fn approval_requirement_definitions_for_entity_id_opt(
        &self,
        ctx: &DalContext,
        entity_id: EntityId,
    ) -> WorkspaceSnapshotResult<Option<Vec<ApprovalRequirementDefinition>>> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .approval_requirement_definitions_for_entity_id_opt(ctx, entity_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .approval_requirement_definitions_for_entity_id_opt(ctx, entity_id)
                    .await
            }
        }
    }

    async fn entity_id_for_approval_requirement_definition_id(
        &self,
        id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<EntityId> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .entity_id_for_approval_requirement_definition_id(id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .entity_id_for_approval_requirement_definition_id(id)
                    .await
            }
        }
    }

    async fn get_approval_requirement_definition_by_id(
        &self,
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<ApprovalRequirementDefinition> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .get_approval_requirement_definition_by_id(ctx, id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .get_approval_requirement_definition_by_id(ctx, id)
                    .await
            }
        }
    }
}

#[async_trait]
impl InputSocketExt for WorkspaceSnapshotSelector {
    async fn get_input_socket(
        &self,
        ctx: &DalContext,
        id: InputSocketId,
    ) -> WorkspaceSnapshotResult<InputSocket> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.get_input_socket(ctx, id).await,
            Self::SplitSnapshot(snapshot) => snapshot.get_input_socket(ctx, id).await,
        }
    }

    async fn get_input_socket_by_name_opt(
        &self,
        ctx: &DalContext,
        name: &str,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Option<InputSocket>> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .get_input_socket_by_name_opt(ctx, name, schema_variant_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .get_input_socket_by_name_opt(ctx, name, schema_variant_id)
                    .await
            }
        }
    }

    async fn list_input_socket_ids_for_schema_variant(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Vec<InputSocketId>> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .list_input_socket_ids_for_schema_variant(schema_variant_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .list_input_socket_ids_for_schema_variant(schema_variant_id)
                    .await
            }
        }
    }

    async fn list_input_sockets(
        &self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Vec<InputSocket>> {
        let mut result = match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot.list_input_sockets(ctx, schema_variant_id).await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot.list_input_sockets(ctx, schema_variant_id).await
            }
        }?;
        result.sort_by_key(|socket| socket.id());

        Ok(result)
    }

    async fn all_attribute_value_ids_everywhere_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotResult<Vec<AttributeValueId>> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .all_attribute_value_ids_everywhere_for_input_socket_id(input_socket_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .all_attribute_value_ids_everywhere_for_input_socket_id(input_socket_id)
                    .await
            }
        }
    }

    async fn component_attribute_value_id_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotResult<AttributeValueId> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .component_attribute_value_id_for_input_socket_id(input_socket_id, component_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .component_attribute_value_id_for_input_socket_id(input_socket_id, component_id)
                    .await
            }
        }
    }

    async fn input_socket_id_find_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> WorkspaceSnapshotResult<Option<InputSocketId>> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .input_socket_id_find_for_attribute_value_id(attribute_value_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .input_socket_id_find_for_attribute_value_id(attribute_value_id)
                    .await
            }
        }
    }
}

#[async_trait]
impl SchemaVariantExt for WorkspaceSnapshotSelector {
    async fn schema_id_for_schema_variant_id(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<SchemaId> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .schema_id_for_schema_variant_id(schema_variant_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .schema_id_for_schema_variant_id(schema_variant_id)
                    .await
            }
        }
    }

    async fn schema_variant_add_edge_to_input_socket(
        &self,
        schema_variant_id: SchemaVariantId,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => {
                snapshot
                    .schema_variant_add_edge_to_input_socket(schema_variant_id, input_socket_id)
                    .await
            }
            Self::SplitSnapshot(snapshot) => {
                snapshot
                    .schema_variant_add_edge_to_input_socket(schema_variant_id, input_socket_id)
                    .await
            }
        }
    }
}

#[async_trait]
impl EntityKindExt for WorkspaceSnapshotSelector {
    async fn get_entity_kind_for_id(&self, id: EntityId) -> WorkspaceSnapshotResult<EntityKind> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.get_entity_kind_for_id(id).await,
            Self::SplitSnapshot(snapshot) => snapshot.get_entity_kind_for_id(id).await,
        }
    }
}

#[async_trait]
impl ViewExt for WorkspaceSnapshotSelector {
    async fn view_remove(&self, view_id: ViewId) -> WorkspaceSnapshotResult<()> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.view_remove(view_id).await,
            Self::SplitSnapshot(snapshot) => snapshot.view_remove(view_id).await,
        }
    }

    async fn list_for_component_id(&self, id: ComponentId) -> WorkspaceSnapshotResult<Vec<ViewId>> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.list_for_component_id(id).await,
            Self::SplitSnapshot(snapshot) => snapshot.list_for_component_id(id).await,
        }
    }
}

#[async_trait]
impl PropExt for WorkspaceSnapshotSelector {
    async fn ts_type(&self, prop_id: PropId) -> PropResult<String> {
        match self {
            Self::LegacySnapshot(snapshot) => snapshot.ts_type(prop_id).await,
            Self::SplitSnapshot(snapshot) => snapshot.ts_type(prop_id).await,
        }
    }
}
