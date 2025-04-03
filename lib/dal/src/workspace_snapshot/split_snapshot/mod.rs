use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicBool, Arc},
};

use async_trait::async_trait;
use petgraph::Direction::{self, Incoming, Outgoing};
use serde::{Deserialize, Serialize};
use si_events::{
    merkle_tree_hash::MerkleTreeHash,
    workspace_snapshot::{Change, Checksum, EntityKind},
    ContentHash, WorkspaceSnapshotAddress,
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
    CycleCheckGuard, DependentValueRoot, EntityKindExt, InferredConnectionsWriteGuard,
    InputSocketExt, SchemaVariantExt, WorkspaceSnapshotError, WorkspaceSnapshotResult,
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
    address: Arc<Mutex<WorkspaceSnapshotAddress>>,
    read_only_graph: Arc<SplitSnapshotGraph>,
    working_copy: Arc<RwLock<Option<SplitSnapshotGraphVCurrent>>>,
    cycle_check: Arc<AtomicBool>,
    dvu_roots: Arc<Mutex<HashSet<DependentValueRoot>>>,
    inferred_connection_graph: Arc<RwLock<Option<InferredConnectionGraph>>>,
}

impl SplitSnapshot {
    pub async fn id(&self) -> WorkspaceSnapshotAddress {
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
        // XXX: do we need to use the generator for monotonically increasing IDs? is that really necessary?
        Ok(Ulid::new())
    }

    pub async fn enable_cycle_check(&self) -> CycleCheckGuard {
        self.cycle_check
            .store(true, std::sync::atomic::Ordering::Relaxed);
        CycleCheckGuard::new(self.cycle_check.clone())
    }

    pub async fn disable_cycle_check(&self) {
        self.cycle_check
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn cycle_check(&self) -> bool {
        self.cycle_check.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn serialized(&self) -> WorkspaceSnapshotResult<Vec<u8>> {
        todo!()
    }

    pub async fn is_acyclic_directed(&self) -> bool {
        todo!()
    }

    pub async fn add_or_replace_node(&self, node: NodeWeight) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.add_or_replace_node(node)?;

        Ok(())
    }

    pub async fn add_ordered_node(&self, node: NodeWeight) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.add_ordered_node(node)?;

        Ok(())
    }

    pub async fn update_content(
        &self,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotResult<()> {
        let mut working_copy = self.working_copy_mut().await;

        match working_copy.node_weight_mut(id) {
            Some(node_weight_mut) => {
                node_weight_mut.new_content_hash(new_content_hash)?;
                working_copy.touch_node(id);
                Ok(())
            }
            None => Err(WorkspaceSnapshotError::NodeNotFoundAtId(id)),
        }
    }

    pub async fn add_edge(
        &self,
        from_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut()
            .await
            .add_edge(from_id.into(), edge_weight, to_id.into())?;
        Ok(())
    }

    pub async fn add_edge_unchecked(
        &self,
        from_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        // TODO: Implement add_edge_unchecked
        self.add_edge(from_id, edge_weight, to_id).await
    }

    pub async fn add_ordered_edge(
        &self,
        from_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.add_ordered_edge(
            from_id.into(),
            edge_weight,
            to_id.into(),
        )?;
        Ok(())
    }

    pub async fn detect_changes(
        &self,
        updated_snapshot: &Self,
    ) -> WorkspaceSnapshotResult<Vec<Change>> {
        Ok(self
            .working_copy()
            .await
            .detect_changes(&*updated_snapshot.working_copy().await)?)
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
        let id = id.into();
        Ok(self
            .get_node_weight_opt(id)
            .await
            .ok_or(WorkspaceSnapshotError::NodeNotFoundAtId(id))?)
    }

    pub async fn get_node_weight_opt(&self, id: impl Into<Ulid>) -> Option<NodeWeight> {
        self.working_copy().await.node_weight(id.into()).cloned()
    }

    pub async fn cleanup(&self) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.cleanup();
        Ok(())
    }

    pub async fn cleanup_and_merkle_tree_hash(&self) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.cleanup_and_merkle_tree_hash();
        Ok(())
    }

    pub async fn nodes(&self) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        Ok(self.working_copy().await.nodes().cloned().collect())
    }

    pub async fn edges(&self) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        Ok(self
            .working_copy()
            .await
            .edges()
            .map(|(weight, src, dst)| (weight.clone(), src, dst))
            .collect())
    }

    pub async fn node_exists(&self, id: impl Into<Ulid>) -> bool {
        self.working_copy()
            .await
            .node_id_to_index(id.into())
            .is_some()
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
        Ok(self
            .working_copy()
            .await
            .edges_directed(id.into(), direction)?
            .filter_map(|edge_ref| {
                edge_ref
                    .weight()
                    .custom()
                    .cloned()
                    .map(|weight| (weight, edge_ref.source(), edge_ref.target()))
            })
            .collect())
    }

    pub async fn edges_directed_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        Ok(self
            .working_copy()
            .await
            .edges_directed(id.into(), direction)?
            .filter_map(|edge_ref| {
                edge_ref
                    .weight()
                    .custom()
                    .cloned()
                    .map(|weight| (weight, edge_ref.source(), edge_ref.target()))
            })
            .filter(|(weight, _, _)| edge_kind == weight.kind().into())
            .collect())
    }

    pub async fn remove_all_edges(&self, id: impl Into<Ulid>) -> WorkspaceSnapshotResult<()> {
        todo!()
    }

    pub async fn incoming_sources_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<Ulid>> {
        Ok(self
            .working_copy()
            .await
            .edges_directed(id.into(), Incoming)?
            .filter_map(|edge_ref| match edge_ref.weight().custom() {
                Some(weight) => {
                    if edge_weight_kind_discrim == weight.kind().into() {
                        Some(edge_ref.source())
                    } else {
                        None
                    }
                }
                None => None,
            })
            .collect())
    }

    pub async fn outgoing_targets_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<Ulid>> {
        Ok(self
            .working_copy()
            .await
            .edges_directed(id.into(), Outgoing)?
            .filter_map(|edge_ref| match edge_ref.weight().custom() {
                Some(weight) => {
                    if edge_weight_kind_discrim == weight.kind().into() {
                        Some(edge_ref.target())
                    } else {
                        None
                    }
                }
                None => None,
            })
            .collect())
    }

    pub async fn all_outgoing_targets(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        let working_copy = self.working_copy().await;
        let targets = working_copy
            .edges_directed(id.into(), Outgoing)?
            .filter_map(|edge_ref| working_copy.node_weight(edge_ref.target()))
            .cloned()
            .collect();

        Ok(targets)
    }

    pub async fn all_incoming_sources(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        let working_copy = self.working_copy().await;
        let sources = working_copy
            .edges_directed(id.into(), Incoming)?
            .filter_map(|edge_ref| working_copy.node_weight(edge_ref.source()))
            .cloned()
            .collect();

        Ok(sources)
    }

    pub async fn remove_incoming_edges_of_kind(
        &self,
        target_id: impl Into<Ulid>,
        kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        let target_id = target_id.into();
        let mut working_copy = self.working_copy_mut().await;
        let sources: Vec<_> = working_copy
            .edges_directed(target_id, Incoming)?
            .filter_map(|edge_ref| match edge_ref.weight().custom() {
                Some(weight) => {
                    if kind == weight.kind().into() {
                        Some(edge_ref.source())
                    } else {
                        None
                    }
                }
                None => None,
            })
            .collect();

        for source_id in sources {
            working_copy.remove_edge(source_id, kind, target_id)?;
        }

        Ok(())
    }

    pub async fn get_edges_between_nodes(
        &self,
        from_node_id: Ulid,
        to_node_id: Ulid,
    ) -> WorkspaceSnapshotResult<Vec<EdgeWeight>> {
        Ok(self
            .working_copy()
            .await
            .edges_directed(from_node_id, Outgoing)?
            .filter_map(|edge_ref| match edge_ref.weight().custom() {
                Some(weight) => {
                    if edge_ref.target() == to_node_id {
                        Some(weight.clone())
                    } else {
                        None
                    }
                }
                None => None,
            })
            .collect())
    }

    pub async fn remove_node_by_id(&self, id: impl Into<Ulid>) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.remove_node(id.into())?;
        Ok(())
    }

    pub async fn remove_edge(
        &self,
        source_id: impl Into<Ulid>,
        target_id: impl Into<Ulid>,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut()
            .await
            .remove_edge(source_id.into(), edge_kind, target_id.into())?;
        Ok(())
    }

    pub async fn find_edge(
        &self,
        from_id: impl Into<Ulid>,
        to_id: impl Into<Ulid>,
        edge_weight_kind: EdgeWeightKindDiscriminants,
    ) -> Option<EdgeWeight> {
        self.working_copy()
            .await
            .find_edge(from_id.into(), to_id.into(), edge_weight_kind)
            .cloned()
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

    pub async fn update_node_id(
        &self,
        current_id: impl Into<Ulid>,
        new_id: impl Into<Ulid>,
        new_lineage_id: LineageId,
    ) -> WorkspaceSnapshotResult<()> {
        let current_id = current_id.into();
        let mut working_copy = self.working_copy_mut().await;
        let node_weight = working_copy
            .node_weight_mut(current_id)
            .ok_or(WorkspaceSnapshotError::NodeNotFoundAtId(current_id))?;

        node_weight.set_id_and_lineage(new_id, new_lineage_id);

        Ok(())
    }

    pub async fn ordered_children_for_node(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<Vec<Ulid>>> {
        Ok(self.working_copy().await.ordered_children(id.into()))
    }

    pub async fn dvu_root_check(&self, root: DependentValueRoot) -> bool {
        // ensure we don't grow the graph unnecessarily by adding the same value
        // in a single edit session
        let mut dvu_roots = self.dvu_roots.lock().await;

        if dvu_roots.contains(&root) {
            true
        } else {
            dvu_roots.insert(root);
            false
        }
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
impl EntityKindExt for SplitSnapshot {
    async fn get_entity_kind_for_id(&self, id: EntityId) -> WorkspaceSnapshotResult<EntityKind> {
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
