use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicBool, Arc},
};

use async_trait::async_trait;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use si_events::{
    merkle_tree_hash::MerkleTreeHash,
    workspace_snapshot::{Change, Checksum},
    ContentHash, SplitSnapshotAddress,
};
use si_id::{
    ulid::Ulid, ApprovalRequirementDefinitionId, AttributeValueId, ComponentId, EntityId, FuncId,
    InputSocketId, PropId, SchemaId, SchemaVariantId, UserPk, ViewId,
};
use si_split_graph::SplitGraph;
use strum::{EnumDiscriminants, EnumIter, EnumString};
use tokio::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    approval_requirement::{
        ApprovalRequirement, ApprovalRequirementApprover, ApprovalRequirementDefinition,
    },
    component::{inferred_connection_graph::InferredConnectionGraph, ComponentResult, Connection},
    prop::PropResult,
    socket::connection_annotation::ConnectionAnnotation,
    DalContext, EdgeWeight, EdgeWeightKindDiscriminants, InputSocket, SocketArity, SocketKind,
};

use super::{
    graph::LineageId,
    node_weight::{category_node_weight::CategoryNodeKind, NodeWeight},
    traits::{approval_requirement::ApprovalRequirementExt, diagram::view::ViewExt, prop::PropExt},
    CycleCheckGuard, DependentValueRoot, InferredConnectionsWriteGuard, InputSocketExt,
    SchemaVariantExt, WorkspaceSnapshotResult,
};

pub type SplitSnapshotGraphV1 = SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>;
pub type SplitSnapshotGraphVCurrent = SplitSnapshotGraphV1;

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize, EnumString, EnumIter))]
pub enum SplitSnapshotGraph {
    V1(SplitSnapshotGraphV1),
}

impl std::ops::Deref for SplitSnapshotGraph {
    type Target = SplitSnapshotGraphVCurrent;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl std::ops::DerefMut for SplitSnapshotGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

impl SplitSnapshotGraph {
    pub fn inner(&self) -> &SplitSnapshotGraphVCurrent {
        match self {
            SplitSnapshotGraph::V1(inner) => inner,
        }
    }

    pub fn inner_mut(&mut self) -> &mut SplitSnapshotGraphVCurrent {
        match self {
            SplitSnapshotGraph::V1(inner) => inner,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SplitGraphUpdates {
    updates: Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
}

#[must_use = "if unused the lock will be released immediately"]
struct SnapshotReadGuard<'a> {
    read_only_graph: Arc<SplitSnapshotGraph>,
    working_copy_read_guard: RwLockReadGuard<'a, Option<SplitSnapshotGraphVCurrent>>,
}

#[must_use = "if unused the lock will be released immediately"]
struct SnapshotWriteGuard<'a> {
    working_copy_write_guard: RwLockWriteGuard<'a, Option<SplitSnapshotGraphVCurrent>>,
}

impl std::ops::Deref for SnapshotReadGuard<'_> {
    type Target = SplitSnapshotGraphVCurrent;

    fn deref(&self) -> &Self::Target {
        if self.working_copy_read_guard.is_some() {
            let option = &*self.working_copy_read_guard;
            option.as_ref().expect("we confirmed it was some above")
        } else {
            &self.read_only_graph
        }
    }
}

impl std::ops::Deref for SnapshotWriteGuard<'_> {
    type Target = SplitSnapshotGraphVCurrent;

    fn deref(&self) -> &Self::Target {
        let option = &*self.working_copy_write_guard;
        option.as_ref().expect(
            "attempted to deref snapshot without copying contents into the mutable working copy",
        )
    }
}

impl std::ops::DerefMut for SnapshotWriteGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let option: &mut Option<SplitSnapshotGraphVCurrent> = &mut self.working_copy_write_guard;
        &mut *option.as_mut().expect("attempted to DerefMut a snapshot without copying contents into the mutable working copy")
    }
}

#[derive(Debug, Clone)]
pub struct SplitSnapshot {
    address: Arc<Mutex<SplitSnapshotAddress>>,
    read_only_graph: Arc<SplitSnapshotGraph>,
    working_copy: Arc<RwLock<Option<SplitSnapshotGraphVCurrent>>>,
    cycle_check: Arc<AtomicBool>,
    dvu_roots: Arc<Mutex<HashSet<DependentValueRoot>>>,
    inferred_connection_graph: Arc<RwLock<Option<InferredConnectionGraph>>>,
}

impl SplitSnapshot {
    pub async fn id(&self) -> SplitSnapshotAddress {
        *self.address.lock().await
    }

    async fn working_copy(&self) -> SnapshotReadGuard<'_> {
        SnapshotReadGuard {
            read_only_graph: self.read_only_graph.clone(),
            working_copy_read_guard: self.working_copy.read().await,
        }
    }

    async fn working_copy_mut(&self) -> SnapshotWriteGuard<'_> {
        let mut working_copy = self.working_copy.write().await;
        if working_copy.is_none() {
            *working_copy = Some(self.read_only_graph.inner().clone());
        }
        SnapshotWriteGuard {
            working_copy_write_guard: working_copy,
        }
    }

    pub async fn root(&self) -> WorkspaceSnapshotResult<Ulid> {
        Ok(self.working_copy().await.root_id()?)
    }

    pub async fn generate_ulid(&self) -> WorkspaceSnapshotResult<Ulid> {
        todo!()
    }

    pub async fn enable_cycle_check(&self) -> CycleCheckGuard {
        todo!()
    }

    pub async fn disable_cycle_check(&self) {
        todo!()
    }

    pub async fn cycle_check(&self) -> bool {
        todo!()
    }

    pub async fn serialized(&self) -> WorkspaceSnapshotResult<Vec<u8>> {
        todo!()
    }

    pub async fn is_acyclic_directed(&self) -> bool {
        todo!()
    }

    pub async fn add_or_replace_node(&self, node: NodeWeight) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn add_ordered_node(&self, node: NodeWeight) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn update_content(
        &self,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn add_edge(
        &self,
        from_node_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_node_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn add_edge_unchecked(
        &self,
        from_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn add_ordered_edge(
        &self,
        from_node_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_node_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn detect_changes(
        &self,
        updated_snapshot: &Self,
    ) -> WorkspaceSnapshotResult<Vec<Change>> {
        todo!()
    }

    pub async fn calculate_checksum(
        &self,
        ctx: &DalContext,
        ids_with_hashes: Vec<(EntityId, MerkleTreeHash)>,
    ) -> WorkspaceSnapshotResult<Checksum> {
        todo!()
    }

    pub async fn import_component_subgraph(
        &self,
        other: &Arc<Self>,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn get_node_weight(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<NodeWeight> {
        todo!()
    }

    pub async fn get_node_weight_opt(&self, id: impl Into<Ulid>) -> Option<NodeWeight> {
        todo!()
    }

    pub async fn cleanup(&self) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn cleanup_and_merkle_tree_hash(&self) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn nodes(&self) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        todo!()
    }

    pub async fn edges(&self) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        todo!()
    }

    pub async fn dot(&self) {
        todo!()
    }

    pub async fn node_exists(&self, id: impl Into<Ulid>) -> bool {
        todo!()
    }

    pub async fn get_category_node_or_err(
        &self,
        source: Option<Ulid>,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<Ulid> {
        todo!()
    }

    pub async fn get_category_node(
        &self,
        source: Option<Ulid>,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<Option<Ulid>> {
        todo!()
    }

    pub async fn edges_directed(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
    ) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        todo!()
    }

    pub async fn edges_directed_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        todo!()
    }

    pub async fn remove_all_edges(&self, id: impl Into<Ulid>) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn incoming_sources_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<Ulid>> {
        todo!()
    }

    pub async fn outgoing_targets_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<Ulid>> {
        todo!()
    }

    pub async fn all_outgoing_targets(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        todo!()
    }

    pub async fn all_incoming_sources(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        todo!()
    }

    pub async fn remove_incoming_edges_of_kind(
        &self,
        target_id: impl Into<Ulid>,
        kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn get_edges_between_nodes(
        &self,
        from_node_id: Ulid,
        to_node_id: Ulid,
    ) -> WorkspaceSnapshotResult<Vec<EdgeWeight>> {
        todo!()
    }

    pub async fn remove_node_by_id(&self, id: impl Into<Ulid>) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn remove_edge(
        &self,
        source_id: impl Into<Ulid>,
        target_id: impl Into<Ulid>,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn find_edge(
        &self,
        from_id: impl Into<Ulid>,
        to_id: impl Into<Ulid>,
        edge_weight_kind: EdgeWeightKindDiscriminants,
    ) -> Option<EdgeWeight> {
        todo!()
    }

    pub async fn remove_edge_for_ulids(
        &self,
        source_node_id: impl Into<Ulid>,
        target_node_id: impl Into<Ulid>,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn mark_prop_as_able_to_be_used_as_prototype_arg(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn ordering_node_for_container(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<OrderingNodeWeight>> {
        todo!()
    }

    pub async fn update_node_id(
        &self,
        current_id: impl Into<Ulid>,
        new_id: impl Into<Ulid>,
        new_lineage_id: LineageId,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn ordered_children_for_node(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<Vec<Ulid>>> {
        todo!()
    }

    pub async fn socket_edges_removed_relative_to_base(
        &self,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotResult<Vec<Connection>> {
        todo!()
    }

    pub async fn add_dependent_value_root(
        &self,
        root: DependentValueRoot,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn has_dependent_value_roots(&self) -> WorkspaceSnapshotResult<bool> {
        todo!()
    }

    pub async fn take_dependent_values(&self) -> WorkspaceSnapshotResult<Vec<DependentValueRoot>> {
        todo!()
    }

    pub async fn get_dependent_value_roots(
        &self,
    ) -> WorkspaceSnapshotResult<Vec<DependentValueRoot>> {
        todo!()
    }

    pub async fn schema_variant_id_for_component_id(
        &self,
        component_id: ComponentId,
    ) -> ComponentResult<SchemaVariantId> {
        todo!()
    }

    pub async fn frame_contains_components(
        &self,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<ComponentId>> {
        todo!()
    }

    pub async fn inferred_connection_graph(
        &self,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotResult<InferredConnectionsWriteGuard<'_>> {
        todo!()
    }

    pub async fn clear_inferred_connection_graph(&self) {
        todo!()
    }

    pub async fn map_all_nodes_to_change_objects(&self) -> WorkspaceSnapshotResult<Vec<Change>> {
        todo!()
    }

    pub async fn revert(&self) {
        todo!()
    }
}
#[async_trait]
impl ApprovalRequirementExt for SplitSnapshot {
    async fn new_definition(
        &self,
        ctx: &DalContext,
        entity_id: Ulid,
        minimum_approvers_count: usize,
        approvers: HashSet<ApprovalRequirementApprover>,
    ) -> WorkspaceSnapshotResult<ApprovalRequirementDefinitionId> {
        todo!()
    }

    async fn remove_definition(
        &self,
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    async fn add_individual_approver_for_definition(
        &self,
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
        user_id: UserPk,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    async fn remove_individual_approver_for_definition(
        &self,
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
        user_id: UserPk,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    async fn approval_requirements_for_changes(
        &self,
        ctx: &DalContext,
        changes: &[Change],
    ) -> WorkspaceSnapshotResult<(Vec<ApprovalRequirement>, HashMap<EntityId, MerkleTreeHash>)>
    {
        todo!()
    }

    async fn approval_requirement_definitions_for_entity_id_opt(
        &self,
        ctx: &DalContext,
        entity_id: EntityId,
    ) -> WorkspaceSnapshotResult<Option<Vec<ApprovalRequirementDefinition>>> {
        todo!()
    }

    async fn entity_id_for_approval_requirement_definition_id(
        &self,
        id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<EntityId> {
        todo!()
    }

    async fn get_approval_requirement_definition_by_id(
        &self,
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<ApprovalRequirementDefinition> {
        todo!()
    }
}

#[async_trait]
impl InputSocketExt for SplitSnapshot {
    async fn get_input_socket(
        &self,
        ctx: &DalContext,
        id: InputSocketId,
    ) -> WorkspaceSnapshotResult<InputSocket> {
        todo!()
    }

    async fn get_input_socket_by_name_opt(
        &self,
        ctx: &DalContext,
        name: &str,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Option<InputSocket>> {
        todo!()
    }

    #[allow(clippy::too_many_arguments)]
    async fn new_input_socket(
        &self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        name: String,
        func_id: FuncId,
        arity: SocketArity,
        kind: SocketKind,
        connection_annotations: Option<Vec<ConnectionAnnotation>>,
    ) -> WorkspaceSnapshotResult<InputSocket> {
        todo!()
    }

    async fn list_input_socket_ids_for_schema_variant(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Vec<InputSocketId>> {
        todo!()
    }

    async fn list_input_sockets(
        &self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Vec<InputSocket>> {
        todo!()
    }

    async fn all_attribute_value_ids_everywhere_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotResult<Vec<AttributeValueId>> {
        todo!()
    }

    async fn component_attribute_value_id_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotResult<AttributeValueId> {
        todo!()
    }

    async fn input_socket_id_find_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> WorkspaceSnapshotResult<Option<InputSocketId>> {
        todo!()
    }
}

#[async_trait]
impl SchemaVariantExt for SplitSnapshot {
    async fn schema_id_for_schema_variant_id(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<SchemaId> {
        todo!()
    }

    async fn schema_variant_add_edge_to_input_socket(
        &self,
        schema_variant_id: SchemaVariantId,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotResult<()> {
        todo!()
    }
}

#[async_trait]
impl ViewExt for SplitSnapshot {
    async fn view_remove(&self, view_id: ViewId) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    async fn list_for_component_id(&self, id: ComponentId) -> WorkspaceSnapshotResult<Vec<ViewId>> {
        todo!()
    }
}

#[async_trait]
impl PropExt for SplitSnapshot {
    async fn ts_type(&self, prop_id: PropId) -> PropResult<String> {
        todo!()
    }
}
