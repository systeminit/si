use std::collections::{hash_map::Entry, HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};

use deprecated::DeprecatedWorkspaceSnapshotGraphV1;
use detect_updates::{Detector, Update};
/// Ensure [`NodeIndex`] is usable by external crates.
pub use petgraph::graph::NodeIndex;
use petgraph::stable_graph::Edges;
use petgraph::visit::DfsEvent;
pub use petgraph::Direction;
use petgraph::{algo, prelude::*};
use serde::{Deserialize, Serialize};
use si_events::merkle_tree_hash::MerkleTreeHash;
use si_events::{ulid::Ulid, ContentHash};
use si_layer_cache::db::serialize;
use strum::{EnumDiscriminants, EnumString};
use thiserror::Error;

use telemetry::prelude::*;
use ulid::Generator;

use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{CategoryNodeWeight, NodeWeightDiscriminants};
use crate::workspace_snapshot::{
    content_address::ContentAddress,
    edge_weight::{EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants},
    node_weight::{NodeWeight, NodeWeightError, OrderingNodeWeight},
};

pub mod correct_transforms;
pub mod deprecated;
pub mod detect_updates;
mod tests;

pub type LineageId = Ulid;

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum WorkspaceSnapshotGraphError {
    #[error("Cannot compare ordering of container elements between ordered, and un-ordered container: {0:?}, {1:?}")]
    CannotCompareOrderedAndUnorderedContainers(NodeIndex, NodeIndex),
    #[error("could not find category node of kind: {0:?}")]
    CategoryNodeNotFound(CategoryNodeKind),
    // #[error("ChangeSet error: {0}")]
    // ChangeSet(#[from] ChangeSetError),
    #[error("Unable to retrieve content for ContentHash")]
    ContentMissingForContentHash,
    #[error("Action would create a graph cycle")]
    CreateGraphCycle,
    #[error("could not find the newly imported subgraph when performing updates")]
    DestinationNotUpdatedWhenImportingSubgraph,
    #[error("Edge does not exist for EdgeIndex: {0:?}")]
    EdgeDoesNotExist(EdgeIndex),
    #[error("EdgeWeight not found")]
    EdgeWeightNotFound,
    #[error("Problem during graph traversal: {0:?}")]
    GraphTraversal(petgraph::visit::DfsEvent<NodeIndex>),
    #[error("Incompatible node types")]
    IncompatibleNodeTypes,
    #[error("Invalid value graph")]
    InvalidValueGraph,
    #[error("monotonic error: {0}")]
    Monotonic(#[from] ulid::MonotonicError),
    #[error("mutex poisoning: {0}")]
    MutexPoison(String),
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("node weight not found")]
    NodeWeightNotFound,
    #[error("Node with ID {} not found", .0.to_string())]
    NodeWithIdNotFound(Ulid),
    #[error("No Prop found for NodeIndex {0:?}")]
    NoPropFound(NodeIndex),
    #[error("Ordering node {0} has id in its order for non-existent node {1}")]
    OrderingNodeHasNonexistentNodeInOrder(Ulid, Ulid),
    #[error("NodeIndex has too many Ordering children: {0:?}")]
    TooManyOrderingForNode(NodeIndex),
    #[error("NodeIndex has too many Prop children: {0:?}")]
    TooManyPropForNode(NodeIndex),
    #[error("Workspace Snapshot has conflicts and must be rebased")]
    WorkspaceNeedsRebase,
    #[error("Workspace Snapshot has conflicts")]
    WorkspacesConflict,
}

pub type WorkspaceSnapshotGraphResult<T> = Result<T, WorkspaceSnapshotGraphError>;

#[derive(Debug, Deserialize, Serialize, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize, EnumString))]
pub enum WorkspaceSnapshotGraph {
    Legacy,
    V1(DeprecatedWorkspaceSnapshotGraphV1),
    V2(WorkspaceSnapshotGraphV2),
}

impl std::ops::Deref for WorkspaceSnapshotGraph {
    type Target = WorkspaceSnapshotGraphV2;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl WorkspaceSnapshotGraph {
    /// Return a reference to the most up to date enum variant for the graph type
    pub fn inner(&self) -> &WorkspaceSnapshotGraphV2 {
        match self {
            Self::Legacy | Self::V1(_) => {
                unimplemented!("Attempted to access an unmigrated snapshot!")
            }
            Self::V2(inner) => inner,
        }
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct WorkspaceSnapshotGraphV2 {
    graph: StableDiGraph<NodeWeight, EdgeWeight>,
    node_index_by_id: HashMap<Ulid, NodeIndex>,
    node_indices_by_lineage_id: HashMap<LineageId, HashSet<NodeIndex>>,
    root_index: NodeIndex,

    #[serde(skip)]
    ulid_generator: Arc<Mutex<Generator>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RebaseBatch {
    updates: Vec<Update>,
}

impl RebaseBatch {
    pub fn new(updates: Vec<Update>) -> Self {
        Self { updates }
    }

    pub fn updates(&self) -> &[Update] {
        &self.updates
    }
}

impl std::fmt::Debug for WorkspaceSnapshotGraphV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceSnapshotGraph")
            .field("root_index", &self.root_index)
            .field("node_index_by_id", &self.node_index_by_id)
            .field("graph", &self.graph)
            .finish()
    }
}

impl WorkspaceSnapshotGraphV2 {
    pub fn new() -> WorkspaceSnapshotGraphResult<Self> {
        let mut graph: StableDiGraph<NodeWeight, EdgeWeight> =
            StableDiGraph::with_capacity(1024, 1024);
        let mut generator = Generator::new();

        let root_node = NodeWeight::new_content(
            generator.generate()?.into(),
            generator.generate()?.into(),
            ContentAddress::Root,
        );

        let node_id = root_node.id();
        let lineage_id = root_node.lineage_id();
        let root_index = graph.add_node(root_node);

        let mut result = Self {
            root_index,
            graph,
            ulid_generator: Arc::new(Mutex::new(generator)),
            ..Default::default()
        };

        result.add_node_finalize(node_id, lineage_id, root_index)?;

        Ok(result)
    }

    pub fn new_from_parts(
        graph: StableDiGraph<NodeWeight, EdgeWeight>,
        node_index_by_id: HashMap<Ulid, NodeIndex>,
        node_indices_by_lineage_id: HashMap<LineageId, HashSet<NodeIndex>>,
        root_index: NodeIndex,
    ) -> Self {
        Self {
            graph,
            node_index_by_id,
            node_indices_by_lineage_id,
            root_index,
            ulid_generator: Arc::new(Mutex::new(Generator::new())),
        }
    }

    pub fn root(&self) -> NodeIndex {
        self.root_index
    }

    /// Access the internal petgraph for this snapshot
    pub fn graph(&self) -> &StableGraph<NodeWeight, EdgeWeight> {
        &self.graph
    }

    pub fn generate_ulid(&self) -> WorkspaceSnapshotGraphResult<Ulid> {
        Ok(self
            .ulid_generator
            .lock()
            .map_err(|e| WorkspaceSnapshotGraphError::MutexPoison(e.to_string()))?
            .generate()?
            .into())
    }

    pub async fn update_node_id(
        &mut self,
        current_idx: NodeIndex,
        new_id: impl Into<Ulid>,
        new_lineage_id: LineageId,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let new_id = new_id.into();

        self.graph
            .node_weight_mut(current_idx)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?
            .set_id_and_lineage(new_id, new_lineage_id);

        self.add_node_finalize(new_id, new_lineage_id, current_idx)?;

        Ok(())
    }

    pub fn get_latest_node_idx_opt(
        &self,
        node_idx: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<NodeIndex>> {
        if !self.graph.contains_node(node_idx) {
            return Ok(None);
        }

        Ok(Some(self.get_latest_node_idx(node_idx)?))
    }

    #[inline(always)]
    pub fn get_latest_node_idx(
        &self,
        node_idx: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let node_id = self.get_node_weight(node_idx)?.id();
        self.get_node_index_by_id(node_id)
    }

    fn add_edge_inner(
        &mut self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
        cycle_check: bool,
    ) -> WorkspaceSnapshotGraphResult<EdgeIndex> {
        if cycle_check {
            self.add_temp_edge_cycle_check(from_node_index, edge_weight.clone(), to_node_index)?;
        }
        // Because outgoing edges are part of a node's identity, we create a new "from" node
        // as we are effectively writing to that node (we'll need to update the merkle tree
        // hash), and everything in the graph should be treated as copy-on-write.
        let new_from_node_index = self.copy_node_by_index(from_node_index)?;

        // Add the new edge to the new version of the "from" node.
        let new_edge_index =
            self.graph
                .update_edge(new_from_node_index, to_node_index, edge_weight);
        self.update_merkle_tree_hash(new_from_node_index)?;

        // Update the rest of the graph to reflect the new node/edge.
        self.replace_references(from_node_index)?;

        Ok(new_edge_index)
    }

    fn add_temp_edge_cycle_check(
        &mut self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let temp_edge = self
            .graph
            .update_edge(from_node_index, to_node_index, edge_weight.clone());

        let would_create_a_cycle = !self.is_acyclic_directed();
        self.graph.remove_edge(temp_edge);

        if would_create_a_cycle {
            // if you want to find out how the two nodes are already connected,
            // this will give you that info..

            // let paths: Vec<Vec<NodeIndex>> = petgraph::algo::all_simple_paths(
            //     &self.graph,
            //     to_node_index,
            //     from_node_index,
            //     0,
            //     None,
            // )
            // .collect();

            // for path in paths {
            //     for node_index in path {
            //         let node_weight = self.get_node_weight(node_index).expect("should exist");
            //         dbg!(node_weight);
            //     }
            // }

            Err(WorkspaceSnapshotGraphError::CreateGraphCycle)
        } else {
            Ok(())
        }
    }

    pub fn add_edge_with_cycle_check(
        &mut self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<EdgeIndex> {
        self.add_edge_inner(from_node_index, edge_weight, to_node_index, true)
    }

    pub fn add_edge(
        &mut self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<EdgeIndex> {
        // Temporarily add the edge to the existing tree to see if it would create a cycle.
        // Configured to run only in tests because it has a major perf impact otherwise
        #[cfg(test)]
        {
            self.add_temp_edge_cycle_check(from_node_index, edge_weight.clone(), to_node_index)?;
        }

        self.add_edge_inner(from_node_index, edge_weight, to_node_index, false)
    }

    pub(crate) fn remove_node_id(&mut self, id: impl Into<Ulid>) {
        self.node_index_by_id.remove(&id.into());
    }

    fn add_node_finalize(
        &mut self,
        node_id: Ulid,
        lineage_id: Ulid,
        node_idx: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
        // Update the accessor maps using the new index.
        self.node_index_by_id.insert(node_id, node_idx);
        self.node_indices_by_lineage_id
            .entry(lineage_id)
            .and_modify(|set| {
                set.insert(node_idx);
            })
            .or_insert_with(|| HashSet::from([node_idx]));
        self.update_merkle_tree_hash(node_idx)?;

        Ok(())
    }

    pub fn add_node(&mut self, node: NodeWeight) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let node_id = node.id();
        let lineage_id = node.lineage_id();
        let new_node_index = self.graph.add_node(node);

        self.add_node_finalize(node_id, lineage_id, new_node_index)?;

        Ok(new_node_index)
    }

    pub fn add_category_node(
        &mut self,
        id: Ulid,
        lineage_id: Ulid,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let inner_weight = CategoryNodeWeight::new(id, lineage_id, kind);
        let new_node_index = self.add_node(NodeWeight::Category(inner_weight))?;
        Ok(new_node_index)
    }

    pub fn get_category_node(
        &self,
        source: Option<Ulid>,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotGraphResult<Option<(Ulid, NodeIndex)>> {
        let source_index = match source {
            Some(provided_source) => self.get_node_index_by_id(provided_source)?,
            None => self.root_index,
        };

        // TODO(nick): ensure that two target category nodes of the same kind don't exist for the
        // same source node.
        for edgeref in self.graph.edges_directed(source_index, Outgoing) {
            let maybe_category_node_index = edgeref.target();
            let maybe_category_node_weight = self.get_node_weight(maybe_category_node_index)?;

            if let NodeWeight::Category(category_node_weight) = maybe_category_node_weight {
                if category_node_weight.kind() == kind {
                    return Ok(Some((category_node_weight.id(), maybe_category_node_index)));
                }
            }
        }

        Ok(None)
    }

    pub fn edges_directed(
        &self,
        node_index: NodeIndex,
        direction: Direction,
    ) -> Edges<'_, EdgeWeight, Directed, u32> {
        self.graph.edges_directed(node_index, direction)
    }

    pub fn find_edge(&self, from_idx: NodeIndex, to_idx: NodeIndex) -> Option<&EdgeWeight> {
        self.graph
            .find_edge(from_idx, to_idx)
            .and_then(|edge_idx| self.graph.edge_weight(edge_idx))
    }

    pub fn edges_directed_for_edge_weight_kind(
        &self,
        node_index: NodeIndex,
        direction: Direction,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> Vec<(EdgeWeight, NodeIndex, NodeIndex)> {
        self.graph
            .edges_directed(node_index, direction)
            .filter_map(|edge_ref| {
                if edge_kind == edge_ref.weight().kind().into() {
                    Some((
                        edge_ref.weight().to_owned(),
                        edge_ref.source(),
                        edge_ref.target(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn nodes(&self) -> impl Iterator<Item = (&NodeWeight, NodeIndex)> {
        self.graph.node_indices().filter_map(|node_idx| {
            self.get_node_weight_opt(node_idx)
                .map(|weight| (weight, node_idx))
        })
    }

    pub fn edges(&self) -> impl Iterator<Item = (&EdgeWeight, NodeIndex, NodeIndex)> {
        self.graph.edge_indices().filter_map(|edge_idx| {
            self.get_edge_weight_opt(edge_idx)
                .ok()
                .flatten()
                .and_then(|weight| {
                    self.graph
                        .edge_endpoints(edge_idx)
                        .map(|(source, target)| (weight, source, target))
                })
        })
    }

    // TODO(nick): fix this clippy error.
    #[allow(clippy::type_complexity)]
    pub fn add_ordered_edge(
        &mut self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<(EdgeIndex, Option<(EdgeIndex, NodeIndex, NodeIndex)>)> {
        let _start = std::time::Instant::now();
        let new_edge_index = self.add_edge(from_node_index, edge_weight, to_node_index)?;

        let from_node_index = self.get_latest_node_idx(from_node_index)?;
        let to_node_index = self.get_latest_node_idx(to_node_index)?;

        // Find the ordering node of the "container" if there is one, and add the thing pointed to
        // by the `to_node_index` to the ordering. Also point the ordering node at the thing with
        // an `Ordinal` edge, so that Ordering nodes must be touched *after* the things they order
        // in a depth first search
        let maybe_ordinal_edge_information = if let Some(container_ordering_node_index) =
            self.ordering_node_index_for_container(from_node_index)?
        {
            let ordinal_edge_index = self.add_edge(
                container_ordering_node_index,
                EdgeWeight::new(EdgeWeightKind::Ordinal),
                to_node_index,
            )?;

            let container_ordering_node_index =
                self.get_latest_node_idx(container_ordering_node_index)?;

            if let NodeWeight::Ordering(previous_container_ordering_node_weight) =
                self.get_node_weight(container_ordering_node_index)?
            {
                let element_id = self
                    .node_index_to_id(to_node_index)
                    .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;

                let mut new_container_ordering_node_weight =
                    previous_container_ordering_node_weight.clone();
                new_container_ordering_node_weight.push_to_order(element_id);
                self.add_node(NodeWeight::Ordering(new_container_ordering_node_weight))?;
                self.replace_references(container_ordering_node_index)?;
            }

            Some((
                ordinal_edge_index,
                container_ordering_node_index,
                to_node_index,
            ))
        } else {
            None
        };

        Ok((new_edge_index, maybe_ordinal_edge_information))
    }

    pub fn add_ordered_node(
        &mut self,
        node: NodeWeight,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let new_node_index = self.add_node(node)?;

        let ordering_node_id = self.generate_ulid()?;
        let ordering_node_lineage_id = self.generate_ulid()?;
        let ordering_node_index = self.add_node(NodeWeight::Ordering(OrderingNodeWeight::new(
            ordering_node_id,
            ordering_node_lineage_id,
        )))?;

        let edge_index = self.add_edge(
            new_node_index,
            EdgeWeight::new(EdgeWeightKind::Ordering),
            ordering_node_index,
        )?;
        let (source, _) = self.edge_endpoints(edge_index)?;
        Ok(source)
    }

    pub fn cleanup(&mut self) {
        let start = tokio::time::Instant::now();

        // We want to remove all of the "garbage" we've accumulated while operating on the graph.
        // Anything that is no longer reachable from the current `self.root_index` should be
        // removed as it is no longer referenced by anything in the current version of the graph.
        // Fortunately, we don't need to walk the graph to find out if something is reachable from
        // the root, since `has_path_connecting` is slow (depth-first search). Any node that does
        // *NOT* have any incoming edges (aside from the `self.root_index` node) is not reachable,
        // by definition. Finding the list of nodes with no incoming edges is very fast. If we
        // remove all nodes (that are not the `self.root_index` node) that do not have any
        // incoming edges, and we keep doing this until the only one left is the `self.root_index`
        // node, then all remaining nodes are reachable from `self.root_index`.
        let mut old_root_ids: HashSet<NodeIndex>;
        loop {
            old_root_ids = self
                .graph
                .externals(Incoming)
                .filter(|node_id| *node_id != self.root_index)
                .collect();
            if old_root_ids.is_empty() {
                break;
            }

            for stale_node_index in &old_root_ids {
                self.graph.remove_node(*stale_node_index);
            }
        }
        debug!("Removed stale NodeIndex: {:?}", start.elapsed());

        // After we retain the nodes, collect the remaining ids and indices.
        let remaining_node_ids: HashSet<Ulid> = self.graph.node_weights().map(|n| n.id()).collect();
        debug!(
            "Got remaining node IDs: {} ({:?})",
            remaining_node_ids.len(),
            start.elapsed()
        );
        let remaining_node_indices: HashSet<NodeIndex> = self.graph.node_indices().collect();
        debug!(
            "Got remaining NodeIndex: {} ({:?})",
            remaining_node_indices.len(),
            start.elapsed()
        );

        // Cleanup the node index by id map.
        self.node_index_by_id
            .retain(|id, _index| remaining_node_ids.contains(id));
        debug!("Removed stale node_index_by_id: {:?}", start.elapsed());

        // Cleanup the node indices by lineage id map.
        self.node_indices_by_lineage_id
            .iter_mut()
            .for_each(|(_lineage_id, node_indices)| {
                node_indices.retain(|node_index| remaining_node_indices.contains(node_index));
            });
        self.node_indices_by_lineage_id
            .retain(|_lineage_id, node_indices| !node_indices.is_empty());
        debug!(
            "Removed stale node_indices_by_lineage_id: {:?}",
            start.elapsed()
        );
    }

    pub fn find_equivalent_node(
        &self,
        id: Ulid,
        lineage_id: Ulid,
    ) -> WorkspaceSnapshotGraphResult<Option<NodeIndex>> {
        let maybe_equivalent_node = match self.get_node_index_by_id(id) {
            Ok(node_index) => {
                let node_indices = self.get_node_index_by_lineage(lineage_id);
                if node_indices.contains(&node_index) {
                    Some(node_index)
                } else {
                    None
                }
            }
            Err(WorkspaceSnapshotGraphError::NodeWithIdNotFound(_)) => None,
            Err(e) => return Err(e),
        };
        Ok(maybe_equivalent_node)
    }

    fn copy_node_by_index(
        &mut self,
        node_index_to_copy: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        self.add_node(self.get_node_weight(node_index_to_copy)?.clone())
    }

    pub fn detect_updates(&self, updated_graph: &Self) -> Vec<Update> {
        Detector::new(self, updated_graph).detect_updates()
    }

    #[allow(dead_code)]
    pub fn dot(&self) {
        // NOTE(nick): copy the output and execute this on macOS. It will create a file in the
        // process and open a new tab in your browser.
        // ```
        // pbpaste | dot -Tsvg -o foo.svg && open foo.svg
        // ```
        let current_root_weight = self
            .get_node_weight(self.root_index)
            .expect("this should be impossible and this code should only be used for debugging");
        println!(
            "Root Node Weight: {current_root_weight:?}\n{:?}",
            petgraph::dot::Dot::with_config(&self.graph, &[petgraph::dot::Config::EdgeNoLabel])
        );
    }

    /// Produces a subgraph of self that includes only the parent and child trees
    /// of `subgraph_root`. Useful for producing a manageable slice of the graph
    /// for debugging.
    pub fn subgraph(&self, subgraph_root: NodeIndex) -> Option<Self> {
        let mut subgraph: StableDiGraph<NodeWeight, EdgeWeight> = StableDiGraph::new();
        let mut index_map = HashMap::new();
        let mut node_index_by_id = HashMap::new();
        let mut node_indices_by_lineage_id = HashMap::new();
        let mut new_root = None;

        let mut add_node_to_idx = |node_id: Ulid, lineage_id: Ulid, node_idx: NodeIndex| {
            node_index_by_id.insert(node_id, node_idx);
            node_indices_by_lineage_id
                .entry(lineage_id)
                .and_modify(|set: &mut HashSet<NodeIndex>| {
                    set.insert(node_idx);
                })
                .or_insert_with(|| HashSet::from([node_idx]));
        };

        let mut parent_q = VecDeque::from([subgraph_root]);
        let node_weight = self.graph.node_weight(subgraph_root)?.to_owned();
        let sub_id = node_weight.id();
        let sub_lineage_id = node_weight.lineage_id();
        let new_subgraph_root = subgraph.add_node(node_weight);
        add_node_to_idx(sub_id, sub_lineage_id, new_subgraph_root);
        index_map.insert(subgraph_root, new_subgraph_root);

        // Walk to parent
        while let Some(node_idx) = parent_q.pop_front() {
            let mut has_parents = false;
            for edge_ref in self.edges_directed(node_idx, Incoming) {
                has_parents = true;
                let source_idx = edge_ref.source();
                let source_node_weight = self.graph.node_weight(source_idx)?.to_owned();
                let id = source_node_weight.id();
                let lineage_id = source_node_weight.lineage_id();
                let new_source_idx = match index_map.get(&source_idx).copied() {
                    Some(node_idx) => node_idx,
                    None => {
                        let new_source_idx = subgraph.add_node(source_node_weight);
                        index_map.insert(source_idx, new_source_idx);
                        node_index_by_id.insert(id, new_source_idx);
                        node_indices_by_lineage_id
                            .entry(lineage_id)
                            .and_modify(|set: &mut HashSet<NodeIndex>| {
                                set.insert(new_source_idx);
                            })
                            .or_insert_with(|| HashSet::from([new_source_idx]));

                        new_source_idx
                    }
                };

                let edge_weight = edge_ref.weight().to_owned();

                let current_node_idx_in_sub = index_map.get(&node_idx).copied()?;
                if subgraph
                    .find_edge(new_source_idx, current_node_idx_in_sub)
                    .is_none()
                {
                    subgraph.add_edge(new_source_idx, current_node_idx_in_sub, edge_weight);
                }

                parent_q.push_back(source_idx);
            }
            if !has_parents {
                new_root = Some(index_map.get(&node_idx).copied()?);
            }
        }

        // Walk to leaves from subgraph_root
        let mut child_q: VecDeque<NodeIndex> = VecDeque::from([subgraph_root]);
        while let Some(node_idx) = child_q.pop_front() {
            for edge_ref in self.edges_directed(node_idx, Outgoing) {
                let target_idx = edge_ref.target();
                let target_node_weight = self.graph.node_weight(target_idx)?.to_owned();
                let id = target_node_weight.id();
                let lineage_id = target_node_weight.lineage_id();
                let new_target_idx = match index_map.get(&target_idx).copied() {
                    Some(node_idx) => node_idx,
                    None => {
                        let new_target_idx = subgraph.add_node(target_node_weight);
                        index_map.insert(target_idx, new_target_idx);
                        node_index_by_id.insert(id, new_target_idx);
                        node_indices_by_lineage_id
                            .entry(lineage_id)
                            .and_modify(|set: &mut HashSet<NodeIndex>| {
                                set.insert(new_target_idx);
                            })
                            .or_insert_with(|| HashSet::from([new_target_idx]));

                        new_target_idx
                    }
                };

                let edge_weight = edge_ref.weight().to_owned();
                let current_node_idx_in_sub = index_map.get(&edge_ref.source()).copied()?;
                if subgraph
                    .find_edge(current_node_idx_in_sub, new_target_idx)
                    .is_none()
                {
                    subgraph.add_edge(current_node_idx_in_sub, new_target_idx, edge_weight);
                }

                child_q.push_back(target_idx);
            }
        }

        Some(Self {
            graph: subgraph,
            node_index_by_id,
            node_indices_by_lineage_id,
            root_index: new_root?,
            ..Default::default()
        })
    }

    /// Write the graph to disk. This *MAY PANIC*. Use only for debugging.
    #[allow(clippy::disallowed_methods)]
    pub fn write_to_disk(&self, file_suffix: &str) {
        let serialized = serialize::to_vec(self).expect("unable to serialize");
        let filename = format!("{}-{}", Ulid::new(), file_suffix);

        let home_env = std::env::var("HOME").expect("No HOME environment variable set");
        let home = std::path::Path::new(&home_env);
        let mut file = File::create(home.join(&filename)).expect("could not create file");
        file.write_all(&serialized).expect("could not write file");

        println!("Wrote graph to {}", home.join(&filename).display());
    }

    #[allow(clippy::disallowed_methods)]
    pub fn tiny_dot_to_file(&self, suffix: Option<&str>) {
        let suffix = suffix.unwrap_or("dot");
        // NOTE(nick): copy the output and execute this on macOS. It will create a file in the
        // process and open a new tab in your browser.
        // ```
        // GRAPHFILE=<filename-without-extension>; cat $GRAPHFILE.txt | dot -Tsvg -o processed-$GRAPHFILE.svg; open processed-$GRAPHFILE.svg
        // ```
        let dot = petgraph::dot::Dot::with_attr_getters(
            &self.graph,
            &[
                petgraph::dot::Config::NodeNoLabel,
                petgraph::dot::Config::EdgeNoLabel,
            ],
            &|_, edgeref| {
                let discrim: EdgeWeightKindDiscriminants = edgeref.weight().kind().into();
                let color = match discrim {
                    EdgeWeightKindDiscriminants::Action => "black",
                    EdgeWeightKindDiscriminants::ActionPrototype => "black",
                    EdgeWeightKindDiscriminants::AuthenticationPrototype => "black",
                    EdgeWeightKindDiscriminants::Contain => "blue",
                    EdgeWeightKindDiscriminants::FrameContains => "black",
                    EdgeWeightKindDiscriminants::Ordering => "gray",
                    EdgeWeightKindDiscriminants::Ordinal => "gray",
                    EdgeWeightKindDiscriminants::Prop => "orange",
                    EdgeWeightKindDiscriminants::Prototype => "green",
                    EdgeWeightKindDiscriminants::PrototypeArgument => "green",
                    EdgeWeightKindDiscriminants::PrototypeArgumentValue => "green",
                    EdgeWeightKindDiscriminants::Socket => "red",
                    EdgeWeightKindDiscriminants::SocketValue => "purple",
                    EdgeWeightKindDiscriminants::Proxy => "gray",
                    EdgeWeightKindDiscriminants::Root => "black",
                    EdgeWeightKindDiscriminants::Use => "black",
                    EdgeWeightKindDiscriminants::ValidationOutput => "darkcyan",
                };

                match edgeref.weight().kind() {
                    EdgeWeightKind::Contain(key) => {
                        let key = key
                            .as_deref()
                            .map(|key| format!(" ({key}"))
                            .unwrap_or("".into());
                        format!(
                            "label = \"{discrim:?}{key}\"\nfontcolor = {color}\ncolor = {color}"
                        )
                    }
                    _ => format!("label = \"{discrim:?}\"\nfontcolor = {color}\ncolor = {color}"),
                }
            },
            &|_, (node_index, node_weight)| {
                let (label, color) = match node_weight {
                    NodeWeight::Action(_) => ("Action".to_string(), "cyan"),
                    NodeWeight::ActionPrototype(_) => ("Action Prototype".to_string(), "cyan"),
                    NodeWeight::Content(weight) => {
                        let discrim = ContentAddressDiscriminants::from(weight.content_address());
                        let color = match discrim {
                            // Some of these should never happen as they have their own top-level
                            // NodeWeight variant.
                            ContentAddressDiscriminants::ActionPrototype => "green",
                            ContentAddressDiscriminants::AttributePrototype => "green",
                            ContentAddressDiscriminants::Component => "black",
                            ContentAddressDiscriminants::DeprecatedAction => "green",
                            ContentAddressDiscriminants::DeprecatedActionBatch => "green",
                            ContentAddressDiscriminants::DeprecatedActionRunner => "green",
                            ContentAddressDiscriminants::OutputSocket => "red",
                            ContentAddressDiscriminants::Func => "black",
                            ContentAddressDiscriminants::FuncArg => "black",
                            ContentAddressDiscriminants::InputSocket => "red",
                            ContentAddressDiscriminants::JsonValue => "fuchsia",
                            ContentAddressDiscriminants::Module => "yellow",
                            ContentAddressDiscriminants::Prop => "orange",
                            ContentAddressDiscriminants::Root => "black",
                            ContentAddressDiscriminants::Schema => "black",
                            ContentAddressDiscriminants::SchemaVariant => "black",
                            ContentAddressDiscriminants::Secret => "black",
                            ContentAddressDiscriminants::StaticArgumentValue => "green",
                            ContentAddressDiscriminants::ValidationPrototype => "black",
                            ContentAddressDiscriminants::ValidationOutput => "darkcyan",
                        };
                        (discrim.to_string(), color)
                    }
                    NodeWeight::AttributePrototypeArgument(apa) => (
                        format!(
                            "Attribute Prototype Argument{}",
                            apa.targets()
                                .map(|targets| format!(
                                    "\nsource: {}\nto: {}",
                                    targets.source_component_id, targets.destination_component_id
                                ))
                                .unwrap_or("".to_string())
                        ),
                        "green",
                    ),
                    NodeWeight::AttributeValue(_) => ("Attribute Value".to_string(), "blue"),
                    NodeWeight::Category(category_node_weight) => match category_node_weight.kind()
                    {
                        CategoryNodeKind::Action => ("Actions (Category)".to_string(), "black"),
                        CategoryNodeKind::Component => {
                            ("Components (Category)".to_string(), "black")
                        }
                        CategoryNodeKind::DeprecatedActionBatch => {
                            ("Action Batches (Category)".to_string(), "black")
                        }
                        CategoryNodeKind::Func => ("Funcs (Category)".to_string(), "black"),
                        CategoryNodeKind::Schema => ("Schemas (Category)".to_string(), "black"),
                        CategoryNodeKind::Secret => ("Secrets (Category)".to_string(), "black"),
                        CategoryNodeKind::Module => ("Modules (Category)".to_string(), "black"),
                        CategoryNodeKind::DependentValueRoots => {
                            ("Dependent Values (Category)".into(), "black")
                        }
                    },
                    NodeWeight::Component(component) => (
                        "Component".to_string(),
                        if component.to_delete() {
                            "gray"
                        } else {
                            "black"
                        },
                    ),
                    NodeWeight::Func(func_node_weight) => {
                        (format!("Func\n{}", func_node_weight.name()), "black")
                    }
                    NodeWeight::FuncArgument(func_arg_node_weight) => (
                        format!("Func Arg\n{}", func_arg_node_weight.name()),
                        "black",
                    ),
                    NodeWeight::Ordering(_) => {
                        (NodeWeightDiscriminants::Ordering.to_string(), "gray")
                    }
                    NodeWeight::Prop(prop_node_weight) => {
                        (format!("Prop\n{}", prop_node_weight.name()), "orange")
                    }
                    NodeWeight::Secret(secret_node_weight) => (
                        format!("Secret\n{}", secret_node_weight.encrypted_secret_key()),
                        "black",
                    ),
                    NodeWeight::DependentValueRoot(node_weight) => (
                        format!("DependentValue\n{}", node_weight.value_id()),
                        "purple",
                    ),
                };
                let color = color.to_string();
                let id = node_weight.id();
                format!(
                    "label = \"\n\n{label}\n{node_index:?}\n{id}\n\n{:?}\n{:?}\"\nfontcolor = {color}\ncolor = {color}", node_weight.merkle_tree_hash(), node_weight.node_hash(),
                )
            },
        );
        let filename_no_extension = format!("{}-{}", Ulid::new(), suffix);

        let home_str = std::env::var("HOME").expect("could not find home directory via env");
        let home = std::path::Path::new(&home_str);

        let mut file = File::create(home.join(format!("{filename_no_extension}.txt")))
            .expect("could not create file");
        file.write_all(format!("{dot:?}").as_bytes())
            .expect("could not write file");
        println!("dot output stored in file (filename without extension: {filename_no_extension})");
    }

    #[inline(always)]
    pub(crate) fn get_node_index_by_id(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let id = id.into();

        self.node_index_by_id
            .get(&id)
            .copied()
            .ok_or(WorkspaceSnapshotGraphError::NodeWithIdNotFound(id))
    }

    #[inline(always)]
    pub(crate) fn get_node_index_by_id_opt(&self, id: impl Into<Ulid>) -> Option<NodeIndex> {
        let id = id.into();

        self.node_index_by_id.get(&id).copied()
    }

    fn get_node_index_by_lineage(&self, lineage_id: Ulid) -> HashSet<NodeIndex> {
        self.node_indices_by_lineage_id
            .get(&lineage_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn node_index_to_id(&self, node_idx: NodeIndex) -> Option<Ulid> {
        self.graph
            .node_weight(node_idx)
            .map(|node_weight| node_weight.id())
    }

    pub fn get_node_weight_opt(&self, node_index: NodeIndex) -> Option<&NodeWeight> {
        self.graph.node_weight(node_index)
    }

    pub fn get_node_weight(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<&NodeWeight> {
        self.get_node_weight_opt(node_index)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)
    }

    pub fn get_node_weight_by_id_opt(&self, id: impl Into<Ulid>) -> Option<&NodeWeight> {
        self.get_node_index_by_id_opt(id)
            .and_then(|index| self.get_node_weight_opt(index))
    }

    fn get_node_weight_mut(
        &mut self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<&mut NodeWeight> {
        self.graph
            .node_weight_mut(node_index)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)
    }

    pub fn get_edge_weight_opt(
        &self,
        edge_index: EdgeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<&EdgeWeight>> {
        Ok(self.graph.edge_weight(edge_index))
    }

    fn has_path_to_root(&self, node: NodeIndex) -> bool {
        algo::has_path_connecting(&self.graph, self.root_index, node, None)
    }

    pub fn import_component_subgraph(
        &mut self,
        other: &WorkspaceSnapshotGraphV2,
        component_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
        // * DFS event-based traversal.
        //   * DfsEvent::Discover(attribute_prototype_argument_node_index, _):
        //     If APA has targets, skip & return Control::Prune, since we don't want to bring in
        //     Components other than the one specified. Only arguments linking Inout & Output
        //     Sockets will have targets (the source & destination ComponentIDs).
        //   * DfsEvent::Discover(func_node_index, _):
        //     Add edge from Funcs Category node to imported Func node.
        let mut edges_by_tail = HashMap::new();
        petgraph::visit::depth_first_search(&other.graph, Some(component_node_index), |event| {
            self.import_component_subgraph_process_dfs_event(other, &mut edges_by_tail, event)
        })?;

        Ok(())
    }

    /// This assumes that the SchemaVariant for the Component is already present in [`self`][Self].
    fn import_component_subgraph_process_dfs_event(
        &mut self,
        other: &WorkspaceSnapshotGraphV2,
        edges_by_tail: &mut HashMap<NodeIndex, Vec<(NodeIndex, EdgeWeight)>>,
        event: DfsEvent<NodeIndex>,
    ) -> WorkspaceSnapshotGraphResult<petgraph::visit::Control<()>> {
        match event {
            // We only check to see if we can prune graph traversal in the node discovery event.
            // The "real" work is done in the node finished event.
            DfsEvent::Discover(other_node_index, _) => {
                let other_node_weight = other.get_node_weight(other_node_index)?;

                // AttributePrototypeArguments with targets connect Input & Output Sockets, and we
                // don't want to import either the Component on the other end of the connection, or
                // the connection itself. Unfortunately, we can't prune when looking at the
                // relevant edge, as that would prune _all remaining edges_ outgoing from the
                // AttributePrototype.
                if NodeWeightDiscriminants::AttributePrototypeArgument == other_node_weight.into() {
                    let apa_node_weight =
                        other_node_weight.get_attribute_prototype_argument_node_weight()?;
                    if apa_node_weight.targets().is_some() {
                        return Ok(petgraph::visit::Control::Prune);
                    }
                }

                // When we hit something that already exists, we're pretty much guaranteed to have
                // left "the component" and have gone into already-existing Funcs or the Schema
                // Variant.
                if self
                    .find_equivalent_node(other_node_weight.id(), other_node_weight.lineage_id())?
                    .is_some()
                {
                    return Ok(petgraph::visit::Control::Prune);
                }

                Ok(petgraph::visit::Control::Continue)
            }
            // We wait to do the "real" import work in the Finish event as these happen in
            // post-order, so we are guaranteed that all of the targets of the outgoing edges
            // have been imported.
            DfsEvent::Finish(other_node_index, _) => {
                // See if we already have the node from other in self.
                let other_node_weight = other.get_node_weight(other_node_index)?;
                // Even though we prune when the equivalent node is_some() in the Discover event,
                // we will still get a Finish event for the node that returned Control::Prune in
                // its Discover event.
                if self
                    .find_equivalent_node(other_node_weight.id(), other_node_weight.lineage_id())?
                    .is_none()
                {
                    // AttributePrototypeArguments for cross-component connections will still have
                    // their DfsEvent::Finish fire, and won't already exist in self, but we do not
                    // want to import them.
                    if let NodeWeight::AttributePrototypeArgument(
                        attribute_prototype_argument_node_weight,
                    ) = other_node_weight
                    {
                        if attribute_prototype_argument_node_weight.targets().is_some() {
                            return Ok(petgraph::visit::Control::Prune);
                        }
                    }

                    // Import the node.
                    self.add_node(other_node_weight.clone())?;

                    // Create all edges with this node as the tail.
                    if let Entry::Occupied(edges) = edges_by_tail.entry(other_node_index) {
                        for (other_head_node_index, edge_weight) in edges.get() {
                            // Need to get this on every iteration as the node index changes as we
                            // add edges.
                            let self_node_index =
                                self.get_node_index_by_id(other_node_weight.id())?;
                            let other_head_node_weight =
                                other.get_node_weight(*other_head_node_index)?;
                            let self_head_node_index =
                                self.get_node_index_by_id(other_head_node_weight.id())?;
                            self.add_edge(
                                self_node_index,
                                edge_weight.clone(),
                                self_head_node_index,
                            )?;
                        }
                    }

                    // Funcs and Components have incoming edges from their Category nodes that we
                    // want to exist, but won't be discovered by the graph traversal on its own.
                    let category_kind = if NodeWeightDiscriminants::Func == other_node_weight.into()
                    {
                        Some(CategoryNodeKind::Func)
                    } else if NodeWeightDiscriminants::Component == other_node_weight.into() {
                        Some(CategoryNodeKind::Component)
                    } else {
                        None
                    };

                    if let Some(category_node_kind) = category_kind {
                        let self_node_index = self.get_node_index_by_id(other_node_weight.id())?;
                        let (_, category_node_idx) = self
                            .get_category_node(None, category_node_kind)?
                            .ok_or(WorkspaceSnapshotGraphError::CategoryNodeNotFound(
                                CategoryNodeKind::Func,
                            ))?;
                        self.add_edge(
                            category_node_idx,
                            EdgeWeight::new(EdgeWeightKind::new_use()),
                            self_node_index,
                        )?;
                    }
                }

                Ok(petgraph::visit::Control::Continue)
            }
            DfsEvent::BackEdge(tail_node_index, head_node_index)
            | DfsEvent::CrossForwardEdge(tail_node_index, head_node_index)
            | DfsEvent::TreeEdge(tail_node_index, head_node_index) => {
                // We'll keep track of the edges we encounter, instead of doing something with them
                // right away as the common case (TreeEdge) is that we'll encounter the edge before
                // we encounter the node for the head (target) of the edge. We can't actually do
                // anything about importing the edge until we've also imported the head node.
                for edgeref in other
                    .graph
                    .edges_connecting(tail_node_index, head_node_index)
                {
                    edges_by_tail
                        .entry(tail_node_index)
                        .and_modify(|entry| {
                            entry.push((head_node_index, edgeref.weight().clone()));
                        })
                        .or_insert_with(|| vec![(head_node_index, edgeref.weight().clone())]);
                }
                Ok(petgraph::visit::Control::Continue)
            }
        }
    }

    pub fn is_acyclic_directed(&self) -> bool {
        // Using this because "is_cyclic_directed" is recursive.
        algo::toposort(&self.graph, None).is_ok()
    }

    #[allow(dead_code)]
    fn is_on_path_between(&self, start: NodeIndex, end: NodeIndex, node: NodeIndex) -> bool {
        algo::has_path_connecting(&self.graph, start, node, None)
            && algo::has_path_connecting(&self.graph, node, end, None)
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns an `Option<Vec<NodeIndex>>`. If there is an ordering node, then the return will be a
    /// [`Some`], where the [`Vec`] is populated with the [`NodeIndex`] of the nodes specified by
    /// the ordering node, in the order defined by the ordering node. If there is not an ordering
    /// node, then the return will be [`None`].
    pub fn ordered_children_for_node(
        &self,
        container_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<Vec<NodeIndex>>> {
        let mut ordered_child_indexes = Vec::new();
        if let Some(container_ordering_index) =
            self.ordering_node_index_for_container(container_node_index)?
        {
            if let NodeWeight::Ordering(ordering_weight) =
                self.get_node_weight(container_ordering_index)?
            {
                for ordered_id in ordering_weight.order() {
                    if let Some(child_index) = self.node_index_by_id.get(ordered_id).copied() {
                        ordered_child_indexes.push(child_index);
                    }
                }
            }
        } else {
            return Ok(None);
        }

        Ok(Some(ordered_child_indexes))
    }

    pub fn ordering_node_for_container(
        &self,
        container_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<OrderingNodeWeight>> {
        Ok(
            match self.ordering_node_index_for_container(container_node_index)? {
                Some(ordering_node_idx) => match self.get_node_weight_opt(ordering_node_idx) {
                    Some(node_weight) => Some(node_weight.get_ordering_node_weight()?.clone()),
                    None => None,
                },
                None => None,
            },
        )
    }

    pub fn ordering_node_index_for_container(
        &self,
        container_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<NodeIndex>> {
        let onto_ordering_node_indexes =
            ordering_node_indexes_for_node_index(self, container_node_index);
        if onto_ordering_node_indexes.len() > 1 {
            error!(
                "Too many ordering nodes found for container NodeIndex {:?}",
                container_node_index
            );
            return Err(WorkspaceSnapshotGraphError::TooManyOrderingForNode(
                container_node_index,
            ));
        }
        Ok(onto_ordering_node_indexes.first().copied())
    }

    pub fn prop_node_index_for_node_index(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<NodeIndex>> {
        let prop_node_indexes = prop_node_indexes_for_node_index(self, node_index);
        if prop_node_indexes.len() > 1 {
            error!("Too many prop nodes found for NodeIndex {:?}", node_index);
            return Err(WorkspaceSnapshotGraphError::TooManyPropForNode(node_index));
        }
        Ok(prop_node_indexes.first().copied())
    }

    pub fn remove_node(&mut self, node_index: NodeIndex) {
        self.graph.remove_node(node_index);
    }

    /// [`StableGraph`] guarantees the stability of [`NodeIndex`] across removals, however there
    /// are **NO** guarantees around the stability of [`EdgeIndex`] across removals. If
    /// [`Self::cleanup()`] has been called, then any [`EdgeIndex`] found before
    /// [`Self::cleanup()`] has run should be considered invalid.
    pub fn remove_edge(
        &mut self,
        source_node_index: NodeIndex,
        target_node_index: NodeIndex,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotGraphResult<()> {
        self.remove_edge_inner(source_node_index, target_node_index, edge_kind)
    }

    /// Removes an edge from `source_node_index` to `target_node_index`, and
    /// also handles removing an edge from the Ordering node if one exists for
    /// the node at `source_node_index`.
    fn remove_edge_inner(
        &mut self,
        source_node_index: NodeIndex,
        target_node_index: NodeIndex,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let source_node_index = self.get_latest_node_idx(source_node_index)?;
        let target_node_index = self.get_latest_node_idx(target_node_index)?;

        self.copy_node_by_index(source_node_index)?;
        self.replace_references(source_node_index)?;
        // replace references may copy the node again to a new index
        let source_node_index = self.get_latest_node_idx(source_node_index)?;

        self.remove_edge_of_kind(source_node_index, target_node_index, edge_kind);

        if let Some(previous_container_ordering_node_index) =
            self.ordering_node_index_for_container(source_node_index)?
        {
            let element_id = self
                .node_index_to_id(target_node_index)
                .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;

            if let NodeWeight::Ordering(previous_container_ordering_node_weight) =
                self.get_node_weight(previous_container_ordering_node_index)?
            {
                let mut new_container_ordering_node_weight =
                    previous_container_ordering_node_weight.clone();

                // We only want to update the ordering of the container if we removed an edge to
                // one of the ordered relationships.
                if new_container_ordering_node_weight.remove_from_order(element_id) {
                    self.remove_edge_of_kind(
                        previous_container_ordering_node_index,
                        target_node_index,
                        EdgeWeightKindDiscriminants::Ordinal,
                    );

                    self.add_node(NodeWeight::Ordering(new_container_ordering_node_weight))?;
                    self.replace_references(previous_container_ordering_node_index)?;
                }
            }
        }

        let source_node_index = self.get_latest_node_idx(source_node_index)?;
        let mut work_queue = VecDeque::from([source_node_index]);

        while let Some(node_index) = work_queue.pop_front() {
            self.update_merkle_tree_hash(
                // If we updated the ordering node, that means we've invalidated the container's
                // NodeIndex (new_source_node_index), so we need to find the new NodeIndex to be able
                // to update the container's merkle tree hash.
                node_index,
            )?;

            for edge_ref in self.graph.edges_directed(node_index, Incoming) {
                work_queue.push_back(edge_ref.source());
            }
        }

        Ok(())
    }

    fn remove_edge_of_kind(
        &mut self,
        source_node_index: NodeIndex,
        target_node_index: NodeIndex,
        edge_kind: EdgeWeightKindDiscriminants,
    ) {
        let mut edges_to_remove = vec![];
        for edgeref in self
            .graph
            .edges_connecting(source_node_index, target_node_index)
        {
            if edge_kind == edgeref.weight().kind().into() {
                edges_to_remove.push(edgeref.id());
            }
        }
        for edge_to_remove in edges_to_remove {
            self.graph.remove_edge(edge_to_remove);
        }
    }

    pub fn edge_endpoints(
        &self,
        edge_index: EdgeIndex,
    ) -> WorkspaceSnapshotGraphResult<(NodeIndex, NodeIndex)> {
        let (source, destination) = self
            .graph
            .edge_endpoints(edge_index)
            .ok_or(WorkspaceSnapshotGraphError::EdgeDoesNotExist(edge_index))?;
        Ok((source, destination))
    }

    /// Replace references should be called when a node has been changed and copied into the graph.
    /// It will use the original_node_index to find the most up to date version of the new node,
    /// and replace all edges that point to that old node with edges pointing to the new node.
    /// Because the graph is treated as an immutable, copy-on-write structure, this means walking
    /// up the graph to the root and copying all nodes that have edges that point to the
    /// original_node_index, and all nodes that have edges that point to *those* parent nodes,
    /// etc, until we've processed the entire parent tree of the original node.
    pub fn replace_references(
        &mut self,
        original_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let mut work_q = VecDeque::from([original_node_index]);
        let mut seen_list = HashSet::new();

        while let Some(old_node_index) = work_q.pop_front() {
            if seen_list.contains(&old_node_index) {
                continue;
            }
            seen_list.insert(old_node_index);

            for edge_ref in self.edges_directed(old_node_index, Direction::Incoming) {
                work_q.push_back(edge_ref.source())
            }

            let latest_node_idx = self.get_latest_node_idx(old_node_index)?;
            let new_node_index = if latest_node_idx != old_node_index {
                latest_node_idx
            } else {
                self.copy_node_by_index(old_node_index)?
            };

            // Find all outgoing edges weights and find the edge targets.
            let mut edges_to_create = Vec::new();
            for edge_ref in self.graph.edges_directed(old_node_index, Outgoing) {
                edges_to_create.push((edge_ref.weight().clone(), edge_ref.target(), edge_ref.id()));
            }

            // Make copies of these edges where the source is the new node index and the
            // destination is one of the following...
            // - If an entry exists in `old_to_new_node_indices` for the destination node index,
            //   use the value of the entry (the destination was affected by the replacement,
            //   and needs to use the new node index to reflect this).
            // - There is no entry in `old_to_new_node_indices`; use the same destination node
            //   index as the old edge (the destination was *NOT* affected by the replacement,
            //   and does not have any new information to reflect).
            for (edge_weight, destination_node_index, edge_idx) in edges_to_create {
                // Need to directly add the edge, without going through `self.add_edge` to avoid
                // infinite recursion, and because we're the place doing all the book keeping
                // that we'd be interested in happening from `self.add_edge`.
                let destination_node_index = self.get_latest_node_idx(destination_node_index)?;

                self.graph.remove_edge(edge_idx);

                self.graph
                    .update_edge(new_node_index, destination_node_index, edge_weight);
            }

            self.update_merkle_tree_hash(new_node_index)?;
        }

        // Use the new version of the old root node as our root node.
        self.root_index = self.get_latest_node_idx(self.root_index)?;

        Ok(())
    }

    pub fn update_content(
        &mut self,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let original_node_index = self.get_node_index_by_id(id)?;
        let new_node_index = self.copy_node_by_index(original_node_index)?;
        let node_weight = self.get_node_weight_mut(new_node_index)?;
        node_weight.new_content_hash(new_content_hash)?;

        self.replace_references(original_node_index)?;
        Ok(())
    }

    pub fn update_order(
        &mut self,
        container_id: Ulid,
        new_order: Vec<Ulid>,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let original_node_index = self
            .ordering_node_index_for_container(self.get_node_index_by_id(container_id)?)?
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;
        let new_node_index = self.copy_node_by_index(original_node_index)?;
        let node_weight = self.get_node_weight_mut(new_node_index)?;
        node_weight.set_order(new_order)?;

        self.replace_references(original_node_index)?;
        Ok(())
    }

    fn update_merkle_tree_hash(
        &mut self,
        node_index_to_update: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let mut hasher = MerkleTreeHash::hasher();
        hasher.update(
            self.get_node_weight(node_index_to_update)?
                .node_hash()
                .to_string()
                .as_bytes(),
        );

        // Need to make sure that ordered containers have their ordered children in the
        // order specified by the ordering graph node.
        let explicitly_ordered_children = self
            .ordered_children_for_node(node_index_to_update)?
            .unwrap_or_default();

        // Need to make sure the unordered neighbors are added to the hash in a stable order to
        // ensure the merkle tree hash is identical for identical trees.
        let mut unordered_neighbors = Vec::new();
        for neighbor_node in self
            .graph
            .neighbors_directed(node_index_to_update, Outgoing)
        {
            // Only add the neighbor if it's not one of the ones with an explicit ordering.
            if !explicitly_ordered_children.contains(&neighbor_node) {
                let neighbor_id = self.get_node_weight(neighbor_node)?.id();
                unordered_neighbors.push((neighbor_id, neighbor_node));
            }
        }
        // We'll sort the neighbors by the ID in the NodeWeight, as that will result in more stable
        // results than if we sorted by the NodeIndex itself.
        unordered_neighbors.sort_by_cached_key(|(id, _index)| *id);
        // It's not important whether the explicitly ordered children are first or last, as long as
        // they are always in that position, and are always in the sequence specified by the
        // container's Ordering node.
        let mut ordered_neighbors =
            Vec::with_capacity(explicitly_ordered_children.len() + unordered_neighbors.len());
        ordered_neighbors.extend(explicitly_ordered_children);
        ordered_neighbors.extend::<Vec<NodeIndex>>(
            unordered_neighbors
                .iter()
                .map(|(_id, index)| *index)
                .collect(),
        );

        for neighbor_node in ordered_neighbors {
            hasher.update(
                self.get_node_weight(neighbor_node)?
                    .merkle_tree_hash()
                    .to_string()
                    .as_bytes(),
            );

            // The edge(s) between `node_index_to_update`, and `neighbor_node` potentially encode
            // important information related to the "identity" of `node_index_to_update`.
            for connecting_edgeref in self
                .graph
                .edges_connecting(node_index_to_update, neighbor_node)
            {
                match connecting_edgeref.weight().kind() {
                    // This is the key for an entry in a map.
                    EdgeWeightKind::Contain(Some(key)) => hasher.update(key.as_bytes()),

                    EdgeWeightKind::Use { is_default } => {
                        hasher.update(is_default.to_string().as_bytes())
                    }

                    // This is the key representing an element in a container type corresponding
                    // to an AttributePrototype
                    EdgeWeightKind::Prototype(Some(key)) => hasher.update(key.as_bytes()),

                    // Nothing to do, as these EdgeWeightKind do not encode extra information
                    // in the edge itself.
                    EdgeWeightKind::AuthenticationPrototype
                    | EdgeWeightKind::Action
                    | EdgeWeightKind::ActionPrototype
                    | EdgeWeightKind::Contain(None)
                    | EdgeWeightKind::FrameContains
                    | EdgeWeightKind::PrototypeArgument
                    | EdgeWeightKind::PrototypeArgumentValue
                    | EdgeWeightKind::Socket
                    | EdgeWeightKind::Ordering
                    | EdgeWeightKind::Ordinal
                    | EdgeWeightKind::Prop
                    | EdgeWeightKind::Prototype(None)
                    | EdgeWeightKind::Proxy
                    | EdgeWeightKind::Root
                    | EdgeWeightKind::SocketValue
                    | EdgeWeightKind::ValidationOutput => {}
                }
            }
        }

        let new_node_weight = self
            .graph
            .node_weight_mut(node_index_to_update)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;
        new_node_weight.set_merkle_tree_hash(hasher.finalize());

        Ok(())
    }

    /// Does a depth first post-order walk to recalculate the entire merkle tree
    /// hash. This operation can be expensive, so should be done only when we
    /// know it needs to be (like when migrating snapshots between versions)
    pub fn recalculate_entire_merkle_tree_hash(&mut self) -> WorkspaceSnapshotGraphResult<()> {
        let mut dfs = petgraph::visit::DfsPostOrder::new(&self.graph, self.root_index);

        while let Some(node_index) = dfs.next(&self.graph) {
            self.update_merkle_tree_hash(node_index)?;
        }

        Ok(())
    }

    /// Perform [`Updates`](Update) using [`self`](WorkspaceSnapshotGraph) as the "to rebase" graph
    /// and a provided graph as the "onto" graph.
    pub fn perform_updates(&mut self, updates: &[Update]) -> WorkspaceSnapshotGraphResult<()> {
        for update in updates {
            match update {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } => {
                    let updated_source = self.get_node_index_by_id_opt(source.id);
                    let destination = self.get_node_index_by_id_opt(destination.id);

                    if let (Some(updated_source), Some(destination)) = (updated_source, destination)
                    {
                        self.add_edge(updated_source, edge_weight.clone(), destination)?;
                    }
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } => {
                    let updated_source = self.get_node_index_by_id_opt(source.id);
                    let destination = self.get_node_index_by_id_opt(destination.id);

                    if let (Some(updated_source), Some(destination)) = (updated_source, destination)
                    {
                        self.remove_edge_inner(updated_source, destination, *edge_kind)?;
                    }
                }
                Update::NewNode { node_weight } => {
                    if self.get_node_index_by_id_opt(node_weight.id()).is_none() {
                        self.add_node(node_weight.to_owned())?;
                    }
                }
                Update::ReplaceNode { node_weight } => {
                    let updated_to_rebase = self.get_node_index_by_id_opt(node_weight.id());
                    if let Some(updated_to_rebase) = updated_to_rebase {
                        self.add_node(node_weight.to_owned())?;
                        self.replace_references(updated_to_rebase)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Update node weight in place with a lambda. Use with caution. Generally
    /// we treat node weights as immutable and replace them by creating a new
    /// node with a new node weight and replacing references to point to the new
    /// node.
    pub(crate) fn update_node_weight<L>(
        &mut self,
        node_idx: NodeIndex,
        lambda: L,
    ) -> WorkspaceSnapshotGraphResult<()>
    where
        L: FnOnce(&mut NodeWeight) -> WorkspaceSnapshotGraphResult<()>,
    {
        let node_weight = self
            .graph
            .node_weight_mut(node_idx)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;

        lambda(node_weight)?;

        Ok(())
    }
}

fn ordering_node_indexes_for_node_index(
    snapshot: &WorkspaceSnapshotGraphV2,
    node_index: NodeIndex,
) -> Vec<NodeIndex> {
    snapshot
        .graph
        .edges_directed(node_index, Outgoing)
        .filter_map(|edge_reference| {
            if edge_reference.weight().kind() == &EdgeWeightKind::Ordering
                && matches!(
                    snapshot.get_node_weight(edge_reference.target()),
                    Ok(NodeWeight::Ordering(_))
                )
            {
                return Some(edge_reference.target());
            }

            None
        })
        .collect()
}

fn prop_node_indexes_for_node_index(
    snapshot: &WorkspaceSnapshotGraphV2,
    node_index: NodeIndex,
) -> Vec<NodeIndex> {
    snapshot
        .graph
        .edges_directed(node_index, Outgoing)
        .filter_map(|edge_reference| {
            if edge_reference.weight().kind() == &EdgeWeightKind::Prop
                && matches!(
                    snapshot.get_node_weight(edge_reference.target()),
                    Ok(NodeWeight::Prop(_))
                )
            {
                return Some(edge_reference.target());
            }
            None
        })
        .collect()
}
