use chrono::Utc;
use content_store::{ContentHash, Store, StoreError};
use petgraph::stable_graph::Edges;
use petgraph::{algo, prelude::*, visit::DfsEvent};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::Write;

use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::change_set_pointer::{ChangeSetPointer, ChangeSetPointerError};
use crate::workspace_snapshot::vector_clock::VectorClockId;
use crate::workspace_snapshot::{
    conflict::Conflict,
    content_address::ContentAddress,
    edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants},
    node_weight::{NodeWeight, NodeWeightError, OrderingNodeWeight},
    update::Update,
};

use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{CategoryNodeWeight, NodeWeightDiscriminants};

use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
/// Ensure [`NodeIndex`] is usable by external crates.
pub use petgraph::graph::NodeIndex;
pub use petgraph::Direction;

mod tests;

pub type NodeIndexMap = HashMap<NodeIndex, NodeIndex>;
pub type LineageId = Ulid;

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum WorkspaceSnapshotGraphError {
    #[error("Cannot compare ordering of container elements between ordered, and un-ordered container: {0:?}, {1:?}")]
    CannotCompareOrderedAndUnorderedContainers(NodeIndex, NodeIndex),
    #[error("could not find category node used by node with index {0:?}")]
    CategoryNodeNotFound(NodeIndex),
    #[error("ChangeSet error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("Unable to retrieve content for ContentHash")]
    ContentMissingForContentHash,
    #[error("Content store error: {0}")]
    ContentStore(#[from] StoreError),
    #[error("Action would create a graph cycle")]
    CreateGraphCycle,
    #[error("could not find the newly imported subgraph when performing updates")]
    DestinationNotUpdatedWhenImportingSubgraph,
    #[error("Edge does not exist for EdgeIndex: {0:?}")]
    EdgeDoesNotExist(EdgeIndex),
    #[error("EdgeWeight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("EdgeWeight not found")]
    EdgeWeightNotFound,
    #[error("Problem during graph traversal: {0:?}")]
    GraphTraversal(petgraph::visit::DfsEvent<NodeIndex>),
    #[error("Incompatible node types")]
    IncompatibleNodeTypes,
    #[error("Invalid value graph")]
    InvalidValueGraph,
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("node weight not found")]
    NodeWeightNotFound,
    #[error("Node with ID {} not found", .0.to_string())]
    NodeWithIdNotFound(Ulid),
    #[error("No Prop found for NodeIndex {0:?}")]
    NoPropFound(NodeIndex),
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

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct WorkspaceSnapshotGraph {
    graph: StableDiGraph<NodeWeight, EdgeWeight>,
    node_index_by_id: HashMap<Ulid, NodeIndex>,
    node_indices_by_lineage_id: HashMap<LineageId, HashSet<NodeIndex>>,
    root_index: NodeIndex,
}

impl std::fmt::Debug for WorkspaceSnapshotGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceSnapshotGraph")
            .field("root_index", &self.root_index)
            .field("node_index_by_id", &self.node_index_by_id)
            .field("graph", &self.graph)
            .finish()
    }
}

impl WorkspaceSnapshotGraph {
    pub fn new(change_set: &ChangeSetPointer) -> WorkspaceSnapshotGraphResult<Self> {
        let mut graph: StableDiGraph<NodeWeight, EdgeWeight> = StableDiGraph::with_capacity(1, 0);
        let root_index = graph.add_node(NodeWeight::new_content(
            change_set,
            change_set.generate_ulid()?,
            ContentAddress::Root,
        )?);

        Ok(Self {
            root_index,
            graph,
            ..Default::default()
        })
    }

    pub fn root(&self) -> NodeIndex {
        self.root_index
    }

    pub fn add_edge(
        &mut self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<EdgeIndex> {
        // Temporarily add the edge to the existing tree to see if it would create a cycle.
        // Configured to run only in tests because it has a major perf impact otherwise
        //#[cfg(test)]
        {
            let temp_edge =
                self.graph
                    .update_edge(from_node_index, to_node_index, edge_weight.clone());

            let would_create_a_cycle = !self.is_acyclic_directed();
            self.graph.remove_edge(temp_edge);
            if would_create_a_cycle {
                return Err(WorkspaceSnapshotGraphError::CreateGraphCycle);
            }
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
        self.replace_references(from_node_index, new_from_node_index)?;

        Ok(new_edge_index)
    }

    pub(crate) fn remove_node_id(&mut self, id: impl Into<Ulid>) {
        self.node_index_by_id.remove(&id.into());
    }

    pub fn add_node(&mut self, node: NodeWeight) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        // Cache the node id and the lineage id;
        let node_id = node.id();
        let lineage_id = node.lineage_id();

        // Create the node and cache the index.
        let new_node_index = self.graph.add_node(node);

        // Update the accessor maps using the new index.
        self.node_index_by_id.insert(node_id, new_node_index);
        self.node_indices_by_lineage_id
            .entry(lineage_id)
            .and_modify(|set| {
                set.insert(new_node_index);
            })
            .or_insert_with(|| HashSet::from([new_node_index]));
        self.update_merkle_tree_hash(new_node_index)?;

        Ok(new_node_index)
    }

    pub fn add_category_node(
        &mut self,
        change_set: &ChangeSetPointer,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let inner_weight = CategoryNodeWeight::new(change_set, kind)?;
        let new_node_index = self.add_node(NodeWeight::Category(inner_weight))?;
        Ok(new_node_index)
    }

    pub fn get_category_node(
        &self,
        source: Option<Ulid>,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotGraphResult<(Ulid, NodeIndex)> {
        let source_index = match source {
            Some(provided_source) => self.get_node_index_by_id(provided_source)?,
            None => self.root_index,
        };

        // TODO(nick): ensure that two target category nodes of the same kind don't exist for the
        // same source node.
        for edgeref in self.graph.edges_directed(source_index, Outgoing) {
            let maybe_category_node_index = edgeref.target();
            let maybe_category_node_weight = self
                .graph
                .node_weight(maybe_category_node_index)
                .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;

            if let NodeWeight::Category(category_node_weight) = maybe_category_node_weight {
                if category_node_weight.kind() == kind {
                    return Ok((category_node_weight.id(), maybe_category_node_index));
                }
            }
        }

        Err(WorkspaceSnapshotGraphError::CategoryNodeNotFound(
            source_index,
        ))
    }

    pub fn edges_directed(
        &self,
        node_index: NodeIndex,
        direction: Direction,
    ) -> Edges<'_, EdgeWeight, Directed, u32> {
        self.graph.edges_directed(node_index, direction)
    }

    pub fn nodes(&self) -> impl Iterator<Item = (&NodeWeight, NodeIndex)> {
        self.graph.node_indices().filter_map(|node_idx| {
            self.graph
                .node_weight(node_idx)
                .map(|weight| (weight, node_idx))
        })
    }

    pub fn edges(&self) -> impl Iterator<Item = (&EdgeWeight, NodeIndex, NodeIndex)> {
        self.graph.edge_indices().filter_map(|edge_idx| {
            self.graph.edge_weight(edge_idx).and_then(|weight| {
                self.graph
                    .edge_endpoints(edge_idx)
                    .map(|(source, target)| (weight, source, target))
            })
        })
    }

    pub fn add_ordered_edge(
        &mut self,
        change_set: &ChangeSetPointer,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<EdgeIndex> {
        let _start = std::time::Instant::now();
        // info!("begin adding edge: {:?}", start.elapsed());
        let new_edge_index = self.add_edge(from_node_index, edge_weight, to_node_index)?;
        // info!("added edge: {:?}", start.elapsed());

        let (new_from_node_index, _) = self
            .graph
            .edge_endpoints(new_edge_index)
            .ok_or(WorkspaceSnapshotGraphError::EdgeWeightNotFound)?;

        // Find the ordering node of the "container" if there is one, and add the thing pointed to
        // by the `to_node_index` to the ordering.
        // info!(
        //     "begin ordering node index for container: {:?}",
        //     start.elapsed()
        // );
        if let Some(container_ordering_node_index) =
            self.ordering_node_index_for_container(new_from_node_index)?
        {
            // info!(
            //     "got ordering node index for container: {:?}",
            //     start.elapsed()
            // );
            if let NodeWeight::Ordering(previous_container_ordering_node_weight) = self
                .graph
                .node_weight(container_ordering_node_index)
                .ok_or_else(|| WorkspaceSnapshotGraphError::NodeWeightNotFound)?
            {
                let element_node_weight = self
                    .graph
                    .node_weight(to_node_index)
                    .ok_or_else(|| WorkspaceSnapshotGraphError::NodeWeightNotFound)?;
                let mut new_container_ordering_node_weight =
                    previous_container_ordering_node_weight.clone();
                let mut new_order =
                    Vec::with_capacity(previous_container_ordering_node_weight.order().len() + 1);
                new_order.extend(previous_container_ordering_node_weight.order());
                new_order.push(element_node_weight.id());
                new_container_ordering_node_weight.set_order(change_set, new_order)?;

                let new_container_ordering_node_index =
                    self.add_node(NodeWeight::Ordering(new_container_ordering_node_weight))?;
                self.replace_references(
                    container_ordering_node_index,
                    new_container_ordering_node_index,
                )?;
            }
        }

        Ok(new_edge_index)
    }

    pub fn add_ordered_node(
        &mut self,
        change_set: &ChangeSetPointer,
        node: NodeWeight,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let new_node_index = self.add_node(node)?;
        let ordering_node_index =
            self.add_node(NodeWeight::Ordering(OrderingNodeWeight::new(change_set)?))?;
        let edge_index = self.add_edge(
            new_node_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Ordering)?,
            ordering_node_index,
        )?;
        let (source, _) = self.edge_endpoints(edge_index)?;
        Ok(source)
    }

    pub async fn attribute_value_view(
        &self,
        content_store: &mut impl Store,
        root_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<serde_json::Value> {
        let mut view = serde_json::json![{}];
        let mut nodes_to_add = VecDeque::from([(root_index, "".to_string())]);

        while let Some((current_node_index, write_location)) = nodes_to_add.pop_front() {
            let current_node_weight = self.get_node_weight(current_node_index)?;
            let current_node_content: serde_json::Value = content_store
                .get(&current_node_weight.content_hash())
                .await?
                .ok_or(WorkspaceSnapshotGraphError::ContentMissingForContentHash)?;
            // We don't need to care what kind the prop is, since assigning a value via
            // `pointer_mut` completely overwrites the existing value, regardless of any
            // pre-existing data types.
            let view_pointer = match view.pointer_mut(&write_location) {
                Some(pointer) => {
                    *pointer = current_node_content.clone();
                    pointer
                }
                None => {
                    // This is an error, and really shouldn't ever happen.
                    dbg!(view, write_location, current_node_content);
                    todo!();
                }
            };

            if current_node_content.is_null() {
                // If the value we just inserted is "null", then there shouldn't be any child
                // values, so don't bother looking for them in the graph to be able to add
                // them to the work queue.
                continue;
            }

            // Find the ordering if there is one, so we can add the children in the proper order.
            if let Some(child_ordering) = self.ordered_children_for_node(current_node_index)? {
                for (child_position_index, &child_node_index) in child_ordering.iter().enumerate() {
                    // `.enumerate()` gives us 1-indexed, but we need 0-indexed.

                    // We insert a JSON `Null` as a "place holder" for the write location. We need
                    // it to exist to be able to get a `pointer_mut` to it on the next time around,
                    // but we don't really care what it is, since we're going to completely
                    // overwrite it anyway.
                    for edge in self
                        .graph
                        .edges_connecting(current_node_index, child_node_index)
                    {
                        let child_position = match edge.weight().kind() {
                            EdgeWeightKind::Contain(Some(key)) => {
                                view_pointer
                                    .as_object_mut()
                                    .ok_or(WorkspaceSnapshotGraphError::InvalidValueGraph)?
                                    .insert(key.clone(), serde_json::json![null]);
                                key.clone()
                            }
                            EdgeWeightKind::Contain(None) => {
                                if current_node_content.is_array() {
                                    view_pointer
                                        .as_array_mut()
                                        .ok_or(WorkspaceSnapshotGraphError::InvalidValueGraph)?
                                        .push(serde_json::json![null]);
                                    child_position_index.to_string()
                                } else {
                                    // Get prop name
                                    if let NodeWeight::Prop(prop_weight) = self.get_node_weight(
                                        self.prop_node_index_for_node_index(child_node_index)?
                                            .ok_or(WorkspaceSnapshotGraphError::NoPropFound(
                                                child_node_index,
                                            ))?,
                                    )? {
                                        view_pointer
                                            .as_object_mut()
                                            .ok_or(WorkspaceSnapshotGraphError::InvalidValueGraph)?
                                            .insert(
                                                prop_weight.name().to_string(),
                                                serde_json::json![null],
                                            );
                                        prop_weight.name().to_string()
                                    } else {
                                        return Err(WorkspaceSnapshotGraphError::InvalidValueGraph);
                                    }
                                }
                            }
                            _ => continue,
                        };
                        let child_write_location = format!("{}/{}", write_location, child_position);
                        nodes_to_add.push_back((child_node_index, child_write_location));
                    }
                }
            } else {
                // The child nodes aren't explicitly ordered, so we'll need to come up with one of
                // our own. We'll sort the nodes by their `NodeIndex`, which means that when a
                // write last happened to them (or anywhere further towards the leaves) will
                // determine their sorting in oldest to most recent order.
                let mut child_index_to_position = HashMap::new();
                let mut child_indexes = Vec::new();
                let outgoing_edges = self.graph.edges_directed(current_node_index, Outgoing);
                for edge_ref in outgoing_edges {
                    match edge_ref.weight().kind() {
                        EdgeWeightKind::Contain(Some(key)) => {
                            view_pointer
                                .as_object_mut()
                                .ok_or(WorkspaceSnapshotGraphError::InvalidValueGraph)?
                                .insert(key.clone(), serde_json::json![null]);
                            child_index_to_position.insert(edge_ref.target(), key.clone());
                            child_indexes.push(edge_ref.target());
                        }
                        EdgeWeightKind::Contain(None) => {
                            child_indexes.push(edge_ref.target());
                            if current_node_content.is_array() {
                                view_pointer
                                    .as_array_mut()
                                    .ok_or(WorkspaceSnapshotGraphError::InvalidValueGraph)?
                                    .push(serde_json::json![null]);
                            } else {
                                // Get prop name
                                if let NodeWeight::Prop(prop_weight) = self.get_node_weight(
                                    self.prop_node_index_for_node_index(edge_ref.target())?
                                        .ok_or(WorkspaceSnapshotGraphError::NoPropFound(
                                            edge_ref.target(),
                                        ))?,
                                )? {
                                    view_pointer
                                        .as_object_mut()
                                        .ok_or(WorkspaceSnapshotGraphError::InvalidValueGraph)?
                                        .insert(
                                            prop_weight.name().to_string(),
                                            serde_json::json![null],
                                        );
                                    child_index_to_position
                                        .insert(edge_ref.target(), prop_weight.name().to_string());
                                    child_indexes.push(edge_ref.target());
                                } else {
                                    return Err(WorkspaceSnapshotGraphError::InvalidValueGraph);
                                }
                            }
                        }
                        _ => continue,
                    }
                }
                child_indexes.sort();

                for (child_position_index, child_node_index) in child_indexes.iter().enumerate() {
                    if let Some(key) = child_index_to_position.get(child_node_index) {
                        nodes_to_add
                            .push_back((*child_node_index, format!("{}/{}", write_location, key)));
                    } else {
                        nodes_to_add.push_back((
                            *child_node_index,
                            format!("{}/{}", write_location, child_position_index),
                        ));
                    }
                }
            }
        }

        Ok(view)
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
        info!("Removed stale NodeIndex: {:?}", start.elapsed());

        // After we retain the nodes, collect the remaining ids and indices.
        let remaining_node_ids: HashSet<Ulid> = self.graph.node_weights().map(|n| n.id()).collect();
        info!(
            "Got remaining node IDs: {} ({:?})",
            remaining_node_ids.len(),
            start.elapsed()
        );
        let remaining_node_indices: HashSet<NodeIndex> = self.graph.node_indices().collect();
        info!(
            "Got remaining NodeIndex: {} ({:?})",
            remaining_node_indices.len(),
            start.elapsed()
        );

        // Cleanup the node index by id map.
        self.node_index_by_id
            .retain(|id, _index| remaining_node_ids.contains(id));
        info!("Removed stale node_index_by_id: {:?}", start.elapsed());

        // Cleanup the node indices by lineage id map.
        self.node_indices_by_lineage_id
            .iter_mut()
            .for_each(|(_lineage_id, node_indices)| {
                node_indices.retain(|node_index| remaining_node_indices.contains(node_index));
            });
        self.node_indices_by_lineage_id
            .retain(|_lineage_id, node_indices| !node_indices.is_empty());
        info!(
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

    pub fn detect_conflicts_and_updates(
        &self,
        to_rebase_vector_clock_id: VectorClockId,
        onto: &WorkspaceSnapshotGraph,
        onto_vector_clock_id: VectorClockId,
    ) -> WorkspaceSnapshotGraphResult<(Vec<Conflict>, Vec<Update>)> {
        let mut conflicts: Vec<Conflict> = Vec::new();
        let mut updates: Vec<Update> = Vec::new();
        if let Err(traversal_error) =
            petgraph::visit::depth_first_search(&onto.graph, Some(onto.root_index), |event| {
                self.detect_conflicts_and_updates_process_dfs_event(
                    to_rebase_vector_clock_id,
                    onto,
                    onto_vector_clock_id,
                    event,
                    &mut conflicts,
                    &mut updates,
                )
            })
        {
            return Err(WorkspaceSnapshotGraphError::GraphTraversal(traversal_error));
        };

        Ok((conflicts, updates))
    }

    fn detect_conflicts_and_updates_process_dfs_event(
        &self,
        to_rebase_vector_clock_id: VectorClockId,
        onto: &WorkspaceSnapshotGraph,
        onto_vector_clock_id: VectorClockId,
        event: DfsEvent<NodeIndex>,
        conflicts: &mut Vec<Conflict>,
        updates: &mut Vec<Update>,
    ) -> Result<petgraph::visit::Control<()>, petgraph::visit::DfsEvent<NodeIndex>> {
        match event {
            DfsEvent::Discover(onto_node_index, _) => {
                let onto_node_weight = onto.get_node_weight(onto_node_index).map_err(|err| {
                    dbg!(
                        "Unable to get NodeWeight for onto NodeIndex {:?}: {}",
                        onto_node_index,
                        err,
                    );
                    event
                })?;

                let mut to_rebase_node_indexes = HashSet::new();
                if onto_node_index == onto.root_index {
                    // There can only be one (valid/current) `ContentAddress::Root` at any
                    // given moment, and the `lineage_id` isn't really relevant as it's not
                    // globally stable (even though it is locally stable). This matters as we
                    // may be dealing with a `WorkspaceSnapshotGraph` that is coming to us
                    // externally from a module that we're attempting to import. The external
                    // `WorkspaceSnapshotGraph` will be `self`, and the "local" one will be
                    // `onto`.
                    to_rebase_node_indexes.insert(self.root_index);
                } else {
                    // Only retain node indexes... or indices... if they are part of the current
                    // graph. There may still be garbage from previous updates to the graph
                    // laying around.
                    let mut potential_to_rebase_node_indexes =
                        self.get_node_index_by_lineage(onto_node_weight.lineage_id());
                    potential_to_rebase_node_indexes
                        .retain(|node_index| self.has_path_to_root(*node_index));
                    to_rebase_node_indexes.extend(potential_to_rebase_node_indexes);

                    // TODO(nick): detect category nodes with a different lineage. We will likely
                    // need to check incoming edges in one graph and then look for outgoing edges in
                    // the other graph.
                    // // Since category nodes may be created from scratch from a different workspace,
                    // // they may have different lineage ids. We still want to consider the same
                    // // category kind as an equivalent node, even though it might have a different
                    // // lineage id.
                    // if let NodeWeight::Category(onto_category_node_weight) = onto_node_weight {
                    //     onto_category_node_weight
                    // }
                    //     let category_node_kind = onto_category_node_weight.kind();
                    //     let (_, to_rebase_category_node_index) =
                    //         self.get_category_node(Some(onto_category_node_weight.id()), category_node_kind).map_err(|err| {
                    //             error!(
                    //                 "Unable to get to rebase Category node for kind {:?} from onto {:?}: {}",
                    //                 onto_category_node_weight.kind(), onto, err,
                    //             );
                    //             event
                    //         })?;
                    //     to_rebase_node_indexes.insert(to_rebase_category_node_index);
                    // }
                }

                // We'll lazily populate these, since we don't know if we'll need it at all, and
                // we definitely don't want to be re-fetching this information inside the loop
                // below, as it will be identical every time.
                let mut onto_ordering_node_index = None;

                // If everything with the same `lineage_id` is identical, then we can prune the
                // graph traversal, and avoid unnecessary lookups/comparisons.
                let mut any_content_with_lineage_has_changed = false;

                for to_rebase_node_index in to_rebase_node_indexes {
                    let to_rebase_node_weight =
                        self.get_node_weight(to_rebase_node_index).map_err(|err| {
                            error!(
                                "Unable to get to_rebase NodeWeight for NodeIndex {:?}: {}",
                                to_rebase_node_index, err,
                            );
                            event
                        })?;

                    if onto_node_weight.merkle_tree_hash()
                        == to_rebase_node_weight.merkle_tree_hash()
                    {
                        // If the merkle tree hashes are the same, then the entire sub-graph is
                        // identical, and we don't need to check any further.
                        debug!(
                            "onto {} and to rebase {} merkle tree hashes are the same",
                            onto_node_weight.id(),
                            to_rebase_node_weight.id()
                        );
                        continue;
                    }
                    any_content_with_lineage_has_changed = true;

                    // Check if there's a difference in the node itself (and whether it is a
                    // conflict if there is a difference).
                    if onto_node_weight.content_hash() != to_rebase_node_weight.content_hash() {
                        if to_rebase_node_weight
                            .vector_clock_write()
                            .is_newer_than(onto_node_weight.vector_clock_write())
                        {
                            // The existing node (`to_rebase`) has changes, but has already seen
                            // all of the changes in `onto`. There is no conflict, and there is
                            // nothing to update.
                        } else if onto_node_weight
                            .vector_clock_write()
                            .is_newer_than(to_rebase_node_weight.vector_clock_write())
                        {
                            // `onto` has changes, but has already seen all of the changes in
                            // `to_rebase`. There is no conflict, and we should update to use the
                            // `onto` node.
                            updates.push(Update::ReplaceSubgraph {
                                onto: onto_node_index,
                                to_rebase: to_rebase_node_index,
                            });
                        } else {
                            // There are changes on both sides that have not
                            // been seen by the other side; this is a conflict.
                            // There may also be other conflicts in the outgoing
                            // relationships, the downstream nodes, or both.

                            // If the nodes in question are ordering nodes, the
                            // conflict we care about is the ChildOrder
                            // conflict, and will have already been detected.
                            // The content on the ordering node is just the
                            // ordering of the edges, so what matters if there
                            // is a conflict in order, not if the hashes differ
                            // because there is an extra edge (but the rest of
                            // the edges are ordered the same)
                            if !matches!(
                                (onto_node_weight, to_rebase_node_weight),
                                (
                                    NodeWeight::Ordering(OrderingNodeWeight { .. }),
                                    NodeWeight::Ordering(OrderingNodeWeight { .. })
                                )
                            ) {
                                conflicts.push(Conflict::NodeContent {
                                    to_rebase: to_rebase_node_index,
                                    onto: onto_node_index,
                                });
                            }
                        }
                    }

                    if onto_ordering_node_index.is_none() {
                        onto_ordering_node_index = onto
                            .ordering_node_index_for_container(onto_node_index)
                            .map_err(|_| event)?;
                    }
                    let to_rebase_ordering_node_index = self
                        .ordering_node_index_for_container(to_rebase_node_index)
                        .map_err(|_| event)?;

                    match (to_rebase_ordering_node_index, onto_ordering_node_index) {
                        (None, None) => {
                            // Neither is ordered. The potential conflict could be because one
                            // or more elements changed, because elements were added/removed,
                            // or a combination of these.
                            //
                            // We need to check for all of these using the outgoing edges from
                            // the containers, since we can't rely on an ordering child to
                            // contain all the information to determine ordering/addition/removal.
                            //
                            // Eventually, this will only happen on the root node itself, since
                            // Objects, Maps, and Arrays should all have an ordering, for at
                            // least display purposes.
                            debug!(
                                "Found what appears to be two unordered containers: onto {:?}, to_rebase {:?}",
                                onto_node_index, to_rebase_node_index,
                            );
                            debug!(
                                "Comparing unordered containers: {:?}, {:?}",
                                onto_node_index, to_rebase_node_index
                            );

                            let (container_conflicts, container_updates) = self
                                .find_unordered_container_membership_conflicts_and_updates(
                                    to_rebase_vector_clock_id,
                                    to_rebase_node_index,
                                    onto,
                                    onto_vector_clock_id,
                                    onto_node_index,
                                )
                                .map_err(|err| {
                                    error!("Unable to find unordered container membership conflicts and updates for onto container NodeIndex {:?} and to_rebase container NodeIndex {:?}: {}", onto_node_index, to_rebase_node_index, err);
                                    event
                                })?;

                            updates.extend(container_updates);
                            conflicts.extend(container_conflicts);
                        }
                        (None, Some(_)) | (Some(_), None) => {
                            // We're trying to compare an ordered container with an unordered one,
                            // which isn't something that logically makes sense, so we've likely
                            // started comparing incompatible things.
                            warn!(
                                "Attempting to compare an ordered, and an unordered container: onto {:?}, to_rebase {:?}",
                                onto_node_index, to_rebase_node_index,
                            );
                            return Err(event);
                        }
                        (Some(to_rebase_ordering_node_index), Some(onto_ordering_node_index)) => {
                            debug!(
                                "Comparing ordered containers: {:?}, {:?}",
                                onto_node_index, to_rebase_node_index
                            );
                            let (container_conflicts, container_updates) = self
                                .find_ordered_container_membership_conflicts_and_updates(
                                    to_rebase_vector_clock_id,
                                    to_rebase_node_index,
                                    to_rebase_ordering_node_index,
                                    onto,
                                    onto_vector_clock_id,
                                    onto_node_index,
                                    onto_ordering_node_index,
                                )
                                .map_err(|_| event)?;

                            updates.extend(container_updates);
                            conflicts.extend(container_conflicts);

                            return Ok(petgraph::visit::Control::Continue);
                        }
                    }
                }

                if any_content_with_lineage_has_changed {
                    // There was at least one thing with a merkle tree hash difference, so we need
                    // to examine further down the tree to see where the difference(s) are, and
                    // where there are conflicts, if there are any.
                    Ok(petgraph::visit::Control::Continue)
                } else {
                    // Everything to be rebased is identical, so there's no need to examine the
                    // rest of the tree looking for differences & conflicts that won't be there.
                    Ok(petgraph::visit::Control::Prune)
                }
            }
            DfsEvent::TreeEdge(_, _)
            | DfsEvent::BackEdge(_, _)
            | DfsEvent::CrossForwardEdge(_, _)
            | DfsEvent::Finish(_, _) => {
                // These events are all ignored, since we handle looking at edges as we encounter
                // the node(s) the edges are coming from (Outgoing edges).
                Ok(petgraph::visit::Control::Continue)
            }
        }
    }

    #[allow(dead_code)]
    pub fn dot(&self) {
        // NOTE(nick): copy the output and execute this on macOS. It will create a file in the
        // process and open a new tab in your browser.
        // ```
        // pbpaste | dot -Tsvg -o foo.svg && open foo.svg
        // ```
        let current_root_weight = self.get_node_weight(self.root_index).unwrap();
        println!(
            "Root Node Weight: {current_root_weight:?}\n{:?}",
            petgraph::dot::Dot::with_config(&self.graph, &[petgraph::dot::Config::EdgeNoLabel])
        );
    }

    #[allow(dead_code)]
    pub fn tiny_dot_to_file(&self) {
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
                    EdgeWeightKindDiscriminants::ActionPrototype => "black",
                    EdgeWeightKindDiscriminants::Contain => "blue",
                    EdgeWeightKindDiscriminants::Ordering => "gray",
                    EdgeWeightKindDiscriminants::InterComponent => "green",
                    EdgeWeightKindDiscriminants::Prop => "orange",
                    EdgeWeightKindDiscriminants::Prototype => "green",
                    EdgeWeightKindDiscriminants::PrototypeArgument => "green",
                    EdgeWeightKindDiscriminants::PrototypeArgumentValue => "green",
                    EdgeWeightKindDiscriminants::Provider => "red",
                    EdgeWeightKindDiscriminants::Proxy => "gray",
                    EdgeWeightKindDiscriminants::Use => "black",
                    EdgeWeightKindDiscriminants::Root => "black",
                };
                format!("label = \"{discrim:?}\"\nfontcolor = {color}\ncolor = {color}")
            },
            &|_, (node_index, node_weight)| {
                let (label, color) = match node_weight {
                    NodeWeight::Content(weight) => {
                        let discrim = ContentAddressDiscriminants::from(weight.content_address());
                        let color = match discrim {
                            ContentAddressDiscriminants::ActionPrototype => "green",
                            ContentAddressDiscriminants::AttributePrototype => "green",
                            ContentAddressDiscriminants::AttributePrototypeArgument => "green",
                            ContentAddressDiscriminants::AttributeValue => "blue",
                            ContentAddressDiscriminants::Component => "black",
                            ContentAddressDiscriminants::ExternalProvider => "red",
                            ContentAddressDiscriminants::Func => "black",
                            ContentAddressDiscriminants::FuncArg => "black",
                            ContentAddressDiscriminants::InternalProvider => "red",
                            ContentAddressDiscriminants::Prop => "orange",
                            ContentAddressDiscriminants::Root => "black",
                            ContentAddressDiscriminants::Schema => "black",
                            ContentAddressDiscriminants::SchemaVariant => "black",
                            ContentAddressDiscriminants::StaticArgumentValue => "green",
                            ContentAddressDiscriminants::ValidationPrototype => "black",
                        };
                        (discrim.to_string(), color)
                    }
                    NodeWeight::Category(category_node_weight) => match category_node_weight.kind()
                    {
                        CategoryNodeKind::Component => {
                            ("Components (Category)".to_string(), "black")
                        }
                        CategoryNodeKind::Func => ("Funcs (Category)".to_string(), "black"),
                        CategoryNodeKind::Schema => ("Schemas (Category)".to_string(), "black"),
                    },
                    NodeWeight::Func(func_node_weight) => {
                        (format!("Func\n{}", func_node_weight.name()), "black")
                    }
                    NodeWeight::Ordering(_) => {
                        (NodeWeightDiscriminants::Ordering.to_string(), "gray")
                    }
                    NodeWeight::Prop(prop_node_weight) => {
                        (format!("Prop\n{}", prop_node_weight.name()), "orange")
                    }
                };
                let color = color.to_string();
                format!(
                    "label = \"\n\n{label}\n{node_index:?}\n\n\n\"\nfontcolor = {color}\ncolor = {color}",
                )
            },
        );
        let filename_no_extension = format!("dot-{}", Ulid::new().to_string());
        let mut file = File::create(format!("/home/zacharyhamm/{filename_no_extension}.txt"))
            .expect("could not create file");
        file.write_all(format!("{dot:?}").as_bytes())
            .expect("could not write file");
        println!("dot output stored in file (filename without extension: {filename_no_extension})");
    }

    #[allow(clippy::too_many_arguments)]
    fn find_ordered_container_membership_conflicts_and_updates(
        &self,
        to_rebase_vector_clock_id: VectorClockId,
        to_rebase_container_index: NodeIndex,
        to_rebase_ordering_index: NodeIndex,
        onto: &WorkspaceSnapshotGraph,
        onto_vector_clock_id: VectorClockId,
        onto_container_index: NodeIndex,
        onto_ordering_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<(Vec<Conflict>, Vec<Update>)> {
        let mut updates = Vec::new();
        let mut conflicts = Vec::new();

        let onto_ordering = match onto.get_node_weight(onto_ordering_index)? {
            NodeWeight::Ordering(ordering) => ordering,
            _ => return Err(WorkspaceSnapshotGraphError::IncompatibleNodeTypes),
        };
        let to_rebase_ordering = match self.get_node_weight(to_rebase_ordering_index)? {
            NodeWeight::Ordering(ordering) => ordering,
            _ => return Err(WorkspaceSnapshotGraphError::IncompatibleNodeTypes),
        };

        if onto_ordering.order() == to_rebase_ordering.order() {
            // Both contain the same items, in the same order. No conflicts, and nothing
            // to update.
            return Ok((conflicts, updates));
        } else if onto_ordering
            .vector_clock_write()
            .is_newer_than(to_rebase_ordering.vector_clock_write())
        {
            // info!("onto_ordering_clock_newer");
            let onto_ordering_set: HashSet<Ulid> = onto_ordering.order().iter().copied().collect();
            let to_rebase_ordering_set: HashSet<Ulid> =
                to_rebase_ordering.order().iter().copied().collect();
            let new_items: HashSet<Ulid> = onto_ordering_set
                .difference(&to_rebase_ordering_set)
                .copied()
                .collect();
            let removed_items: HashSet<Ulid> = to_rebase_ordering_set
                .difference(&onto_ordering_set)
                .copied()
                .collect();

            // Find which `other` container items have the new ordering IDs so we can add edges
            // from the `to_rebase` container to them (and create them in `to_rebase` if they don't
            // already exist).
            for onto_container_item_index in onto
                .graph
                .neighbors_directed(onto_container_index, Outgoing)
            {
                let onto_container_item_weight = onto.get_node_weight(onto_container_item_index)?;
                if new_items.contains(&onto_container_item_weight.id()) {
                    for edge in onto
                        .graph
                        .edges_connecting(onto_container_index, onto_container_item_index)
                    {
                        updates.push(Update::NewEdge {
                            source: to_rebase_container_index,
                            destination: onto_container_item_index,
                            edge_weight: edge.weight().clone(),
                        });
                    }
                }
            }

            // Remove the edges from the `to_rebase` container to the items removed in `onto`. We
            // don't need to worry about removing the items themselves as they will be garbage
            // collected when we drop all items that are not reachable from `to_rebase.root_index`
            // if they are no longer referenced by anything.
            for to_rebase_container_item_index in self
                .graph
                .neighbors_directed(to_rebase_container_index, Outgoing)
            {
                let to_rebase_container_item_weight =
                    self.get_node_weight(to_rebase_container_item_index)?;
                if removed_items.contains(&to_rebase_container_item_weight.id()) {
                    for edgeref in self
                        .graph
                        .edges_connecting(to_rebase_container_index, to_rebase_container_item_index)
                    {
                        updates.push(Update::RemoveEdge {
                            source: edgeref.source(),
                            destination: edgeref.target(),
                            edge_kind: edgeref.weight().kind().into(),
                        });
                    }
                }
            }
        } else if to_rebase_ordering
            .vector_clock_write()
            .is_newer_than(onto_ordering.vector_clock_write())
        {
            // We already have everything in `onto` as part of `to_rebase`. Nothing needs
            // updating, and there are no conflicts.
        } else {
            // Both `onto` and `to_rebase` have changes that the other has not incorporated. We
            // need to find out what the changes are to see what needs to be updated, and what
            // conflicts.
            let onto_ordering_set: HashSet<Ulid> = onto_ordering.order().iter().copied().collect();
            let to_rebase_ordering_set: HashSet<Ulid> =
                to_rebase_ordering.order().iter().copied().collect();

            // Make sure that both `onto` and `to_rebase` have the same relative ordering for the
            // nodes they have in common. If they don't, then that means that the order changed on
            // at least one of them.
            let common_items: HashSet<Ulid> = onto_ordering_set
                .intersection(&to_rebase_ordering_set)
                .copied()
                .collect();
            let common_onto_items = {
                let mut items = onto_ordering.order().clone();
                items.retain(|i| common_items.contains(i));
                items
            };
            let common_to_rebase_items = {
                let mut items = to_rebase_ordering.order().clone();
                items.retain(|i| common_items.contains(i));
                items
            };
            if common_onto_items != common_to_rebase_items {
                conflicts.push(Conflict::ChildOrder {
                    onto: onto_ordering_index,
                    to_rebase: to_rebase_ordering_index,
                });
            }

            let only_onto_items: HashSet<Ulid> = onto_ordering_set
                .difference(&to_rebase_ordering_set)
                .copied()
                .collect();
            let only_to_rebase_items: HashSet<Ulid> = to_rebase_ordering_set
                .difference(&onto_ordering_set)
                .copied()
                .collect();

            let mut only_to_rebase_item_indexes = HashMap::new();
            for to_rebase_edgeref in self
                .graph
                .edges_directed(to_rebase_container_index, Outgoing)
            {
                let dest_node_weight = self.get_node_weight(to_rebase_edgeref.target())?;
                if only_to_rebase_items.contains(&dest_node_weight.id()) {
                    only_to_rebase_item_indexes
                        .insert(dest_node_weight.id(), to_rebase_edgeref.target());
                }
            }

            for only_to_rebase_item in only_to_rebase_items {
                let only_to_rebase_item_index = *only_to_rebase_item_indexes
                    .get(&only_to_rebase_item)
                    .ok_or(WorkspaceSnapshotGraphError::NodeWithIdNotFound(
                        only_to_rebase_item,
                    ))?;
                for to_rebase_edgeref in self
                    .graph
                    .edges_connecting(to_rebase_container_index, only_to_rebase_item_index)
                {
                    if to_rebase_edgeref
                        .weight()
                        .vector_clock_first_seen()
                        .entry_for(onto_vector_clock_id)
                        .is_none()
                    {
                        // `only_to_rebase_item` is new: Edge in `to_rebase` does not have a "First Seen" for `onto`.
                    } else if self
                        .get_node_weight(only_to_rebase_item_index)?
                        .vector_clock_write()
                        .entry_for(to_rebase_vector_clock_id)
                        .is_some()
                    {
                        // Entry was deleted in `onto`. If we have also modified the entry, then
                        // there's a conflict.
                        conflicts.push(Conflict::ModifyRemovedItem(only_to_rebase_item_index));
                    } else {
                        // Entry was deleted in `onto`, and has not been modified in `to_rebase`:
                        // Remove the edge.
                        updates.push(Update::RemoveEdge {
                            source: to_rebase_edgeref.source(),
                            destination: to_rebase_edgeref.target(),
                            edge_kind: to_rebase_edgeref.weight().kind().into(),
                        });
                    }
                }
            }

            let mut only_onto_item_indexes = HashMap::new();
            for onto_edgeref in onto.graph.edges_directed(onto_container_index, Outgoing) {
                let dest_node_weight = onto.get_node_weight(onto_edgeref.target())?;
                if only_onto_items.contains(&dest_node_weight.id()) {
                    only_onto_item_indexes.insert(dest_node_weight.id(), onto_edgeref.target());
                }
            }

            let onto_root_seen_as_of = self
                .get_node_weight(self.root_index)?
                .vector_clock_recently_seen()
                .entry_for(onto_vector_clock_id);
            for only_onto_item in only_onto_items {
                let only_onto_item_index = *only_onto_item_indexes.get(&only_onto_item).ok_or(
                    WorkspaceSnapshotGraphError::NodeWithIdNotFound(only_onto_item),
                )?;
                for onto_edgeref in onto
                    .graph
                    .edges_connecting(onto_container_index, only_onto_item_index)
                {
                    // `only_onto_item` is new:
                    //   - "First seen" of edge for `onto` > "Seen As Of" on root for `onto` in
                    //     `to_rebase`.
                    if let Some(onto_first_seen) = onto_edgeref
                        .weight()
                        .vector_clock_first_seen()
                        .entry_for(onto_vector_clock_id)
                    {
                        if let Some(root_seen_as_of) = onto_root_seen_as_of {
                            if onto_first_seen > root_seen_as_of {
                                // The edge for the item was created more recently than the last
                                // state we knew of from `onto`, which means that the item is
                                // "new". We can't have removed something that we didn't know
                                // existed in the first place.
                                updates.push(Update::NewEdge {
                                    source: to_rebase_container_index,
                                    destination: onto_edgeref.target(),
                                    edge_weight: onto_edgeref.weight().clone(),
                                });
                            }
                        }
                    } else if let Ok(onto_item_node_weight) =
                        onto.get_node_weight(only_onto_item_index)
                    {
                        if let Some(root_seen_as_of) = onto_root_seen_as_of {
                            if onto_item_node_weight
                                .vector_clock_write()
                                .has_entries_newer_than(root_seen_as_of)
                            {
                                // The item removed in `to_rebase` has been modified in `onto`
                                // since we last knew the state of `onto`: This is a conflict, as
                                // we don't know if the removal is still intended given the new
                                // state of the item.
                                conflicts.push(Conflict::RemoveModifiedItem {
                                    container: to_rebase_container_index,
                                    removed_item: only_onto_item_index,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok((conflicts, updates))
    }

    fn find_unordered_container_membership_conflicts_and_updates(
        &self,
        to_rebase_vector_clock_id: VectorClockId,
        to_rebase_container_index: NodeIndex,
        onto: &WorkspaceSnapshotGraph,
        onto_vector_clock_id: VectorClockId,
        onto_container_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<(Vec<Conflict>, Vec<Update>)> {
        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        struct UniqueEdgeInfo {
            pub kind: EdgeWeightKind,
            pub target_lineage: Ulid,
        }

        #[derive(Debug, Copy, Clone)]
        struct EdgeInfo {
            pub source_node_index: NodeIndex,
            pub target_node_index: NodeIndex,
            pub edge_kind: EdgeWeightKindDiscriminants,
            pub edge_index: EdgeIndex,
        }

        let mut updates = Vec::new();
        let mut conflicts = Vec::new();

        let mut to_rebase_edges = HashMap::<UniqueEdgeInfo, EdgeInfo>::new();
        for edgeref in self
            .graph
            .edges_directed(to_rebase_container_index, Outgoing)
        {
            let target_node_weight = self.get_node_weight(edgeref.target())?;
            to_rebase_edges.insert(
                UniqueEdgeInfo {
                    kind: edgeref.weight().kind().clone(),
                    target_lineage: target_node_weight.lineage_id(),
                },
                EdgeInfo {
                    source_node_index: edgeref.source(),
                    target_node_index: edgeref.target(),
                    edge_kind: edgeref.weight().kind().into(),
                    edge_index: edgeref.id(),
                },
            );
        }

        let mut onto_edges = HashMap::<UniqueEdgeInfo, EdgeInfo>::new();
        for edgeref in onto.graph.edges_directed(onto_container_index, Outgoing) {
            let target_node_weight = onto.get_node_weight(edgeref.target())?;
            onto_edges.insert(
                UniqueEdgeInfo {
                    kind: edgeref.weight().kind().clone(),
                    target_lineage: target_node_weight.lineage_id(),
                },
                EdgeInfo {
                    source_node_index: edgeref.source(),
                    target_node_index: edgeref.target(),
                    edge_kind: edgeref.weight().kind().into(),
                    edge_index: edgeref.id(),
                },
            );
        }

        let only_to_rebase_edges = {
            let mut unique_edges = to_rebase_edges.clone();
            for key in onto_edges.keys() {
                unique_edges.remove(key);
            }
            unique_edges
        };
        let only_onto_edges = {
            let mut unique_edges = onto_edges.clone();
            for key in to_rebase_edges.keys() {
                unique_edges.remove(key);
            }
            unique_edges
        };

        debug!("only to rebase edges: {:?}", &only_to_rebase_edges);
        debug!("only onto edges: {:?}", &only_onto_edges);

        let root_seen_as_of_onto = self
            .get_node_weight(self.root_index)?
            .vector_clock_recently_seen()
            .entry_for(onto_vector_clock_id);

        let onto_last_saw_to_rebase = onto
            .get_node_weight(onto.root_index)?
            .vector_clock_recently_seen()
            .entry_for(to_rebase_vector_clock_id);

        for only_to_rebase_edge_info in only_to_rebase_edges.values() {
            let to_rebase_edge_weight = self
                .graph
                .edge_weight(only_to_rebase_edge_info.edge_index)
                .ok_or(WorkspaceSnapshotGraphError::EdgeWeightNotFound)?;
            let to_rebase_item_weight =
                self.get_node_weight(only_to_rebase_edge_info.target_node_index)?;

            // If `onto` has never seen this edge, then it's new, and there are no conflicts, and
            // no updates.
            if to_rebase_edge_weight
                .vector_clock_first_seen()
                .entry_for(to_rebase_vector_clock_id)
                <= onto_last_saw_to_rebase
            {
                if to_rebase_item_weight
                    .vector_clock_write()
                    .entry_for(to_rebase_vector_clock_id)
                    >= onto_last_saw_to_rebase
                {
                    // Item has been modified in `onto` (`onto` item write vector clock > "seen as
                    // of" for `onto` entry in `to_rebase` root): Conflict (ModifyRemovedItem)
                    conflicts.push(Conflict::ModifyRemovedItem(
                        only_to_rebase_edge_info.target_node_index,
                    ))
                } else {
                    // Item not modified & removed by `onto`: No conflict; Update::RemoveEdge
                    updates.push(Update::RemoveEdge {
                        source: only_to_rebase_edge_info.source_node_index,
                        destination: only_to_rebase_edge_info.target_node_index,
                        edge_kind: only_to_rebase_edge_info.edge_kind,
                    });
                }
            } else {
                debug!(
                    "edge weight entry for to rebase vector clock id {:?} is older than onto last saw {:?}", to_rebase_edge_weight.vector_clock_first_seen().entry_for(to_rebase_vector_clock_id), onto_last_saw_to_rebase);
            }
        }

        // - Items unique to `onto`:
        for only_onto_edge_info in only_onto_edges.values() {
            let onto_edge_weight = onto
                .graph
                .edge_weight(only_onto_edge_info.edge_index)
                .ok_or(WorkspaceSnapshotGraphError::EdgeWeightNotFound)?;
            let onto_item_weight = onto.get_node_weight(only_onto_edge_info.target_node_index)?;

            if let Some(onto_first_seen) = onto_edge_weight
                .vector_clock_first_seen()
                .entry_for(onto_vector_clock_id)
            {
                // From "onto_first_seen", we know "when was the first time onto saw this edge?".
                match root_seen_as_of_onto {
                    Some(root_seen_as_of) if onto_first_seen <= root_seen_as_of => {}
                    _ => {
                        // Edge first seen by `onto` > "seen as of" on `to_rebase` graph for `onto`'s entry on
                        // root node: Item is new.
                        // Other case where item is new: the `to_rebase` has never seen anything from
                        // the `onto` change set. All the items are new.
                        updates.push(Update::NewEdge {
                            source: to_rebase_container_index,
                            destination: only_onto_edge_info.target_node_index,
                            edge_weight: onto_edge_weight.clone(),
                        });
                    }
                }
            } else if let Some(root_seen_as_of) = root_seen_as_of_onto {
                if onto_item_weight
                    .vector_clock_write()
                    .has_entries_newer_than(root_seen_as_of)
                {
                    // Item write vector clock has entries > "seen as of" on `to_rebase` graph for
                    // `onto`'s entry on root node: Conflict (RemoveModifiedItem)
                    conflicts.push(Conflict::RemoveModifiedItem {
                        container: to_rebase_container_index,
                        removed_item: only_onto_edge_info.target_node_index,
                    });
                }
            }
            // Item removed by `to_rebase`: No conflict & no update necessary.
        }

        // - Sets same: No conflicts/updates
        Ok((conflicts, updates))
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

    fn get_node_index_by_lineage(&self, lineage_id: Ulid) -> HashSet<NodeIndex> {
        self.node_indices_by_lineage_id
            .get(&lineage_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_node_weight(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<&NodeWeight> {
        self.graph
            .node_weight(node_index)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)
    }

    fn get_node_weight_mut(
        &mut self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<&mut NodeWeight> {
        self.graph
            .node_weight_mut(node_index)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)
    }

    fn has_path_to_root(&self, node: NodeIndex) -> bool {
        algo::has_path_connecting(&self.graph, self.root_index, node, None)
    }

    pub fn import_subgraph(
        &mut self,
        other: &WorkspaceSnapshotGraph,
        root_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<HashMap<NodeIndex, NodeIndex>> {
        let mut updated = HashMap::new();
        let mut dfs = petgraph::visit::DfsPostOrder::new(&other.graph, root_index);
        while let Some(node_index_to_copy) = dfs.next(&other.graph) {
            let node_weight_to_copy = other.get_node_weight(node_index_to_copy)?.clone();
            let node_weight_id = node_weight_to_copy.id();
            let node_weight_lineage_id = node_weight_to_copy.lineage_id();

            // The following assumes there are no conflicts between "self" and "other". If there
            // are conflicts between them, we shouldn't be running updates.
            let node_index = if let Some(equivalent_node_index) =
                self.find_equivalent_node(node_weight_id, node_weight_lineage_id)?
            {
                let equivalent_node_weight = self.get_node_weight(equivalent_node_index)?;
                if equivalent_node_weight
                    .vector_clock_write()
                    .is_newer_than(node_weight_to_copy.vector_clock_write())
                {
                    equivalent_node_index
                } else {
                    let new_node_index = self.add_node(node_weight_to_copy)?;
                    updated.extend(self.replace_references(equivalent_node_index, new_node_index)?);

                    new_node_index
                }
            } else {
                self.add_node(node_weight_to_copy)?
            };

            updated.insert(node_index_to_copy, node_index);

            for edge in other.graph.edges_directed(node_index_to_copy, Outgoing) {
                self.graph.update_edge(
                    node_index,
                    updated
                        .get(&edge.target())
                        .copied()
                        .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?,
                    edge.weight().clone(),
                );
            }
        }
        Ok(updated)
    }

    #[allow(dead_code)]
    fn is_acyclic_directed(&self) -> bool {
        // Using this because "is_cyclic_directed" is recursive.
        algo::toposort(&self.graph, None).is_ok()
    }

    fn is_on_path_between(&self, start: NodeIndex, end: NodeIndex, node: NodeIndex) -> bool {
        algo::has_path_connecting(&self.graph, start, node, None)
            && algo::has_path_connecting(&self.graph, node, end, None)
    }

    pub fn mark_graph_seen(
        &mut self,
        vector_clock_id: VectorClockId,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let seen_at = Utc::now();
        for edge in self.graph.edge_weights_mut() {
            edge.mark_seen_at(vector_clock_id, seen_at);
        }
        for node in self.graph.node_weights_mut() {
            node.mark_seen_at(vector_clock_id, seen_at);
        }

        Ok(())
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns an `Option<Vec<NodeInde>>`. If there is an ordering node, then the return will be a
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
                let mut node_index_by_id = HashMap::new();
                for neighbor_index in self
                    .graph
                    .neighbors_directed(container_node_index, Outgoing)
                {
                    let neighbor_weight = self.get_node_weight(neighbor_index)?;
                    node_index_by_id.insert(neighbor_weight.id(), neighbor_index);
                }
                for ordered_id in ordering_weight.order() {
                    ordered_child_indexes.push(
                        *node_index_by_id
                            .get(ordered_id)
                            .ok_or(WorkspaceSnapshotGraphError::NodeWithIdNotFound(*ordered_id))?,
                    );
                }
            }
        } else {
            return Ok(None);
        }

        Ok(Some(ordered_child_indexes))
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
        Ok(onto_ordering_node_indexes.get(0).copied())
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
        Ok(prop_node_indexes.get(0).copied())
    }

    pub(crate) fn remove_node(&mut self, node_index: NodeIndex) {
        self.graph.remove_node(node_index);
    }

    /// [`StableGraph`] guarantees the stability of [`NodeIndex`] across removals, however there
    /// are **NO** guarantees around the stability of [`EdgeIndex`] across removals. If
    /// [`Self::cleanup()`] has been called, then any [`EdgeIndex`] found before
    /// [`Self::cleanup()`] has run should be considered invalid.
    pub(crate) fn remove_edge(
        &mut self,
        change_set: &ChangeSetPointer,
        source_node_index: NodeIndex,
        target_node_index: NodeIndex,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotGraphResult<HashMap<NodeIndex, NodeIndex>> {
        let mut updated = NodeIndexMap::new();

        let mut edges_to_remove = Vec::new();
        let new_source_node_index = self.copy_node_by_index(source_node_index)?;
        updated.extend(self.replace_references(source_node_index, new_source_node_index)?);

        for edgeref in self
            .graph
            .edges_connecting(new_source_node_index, target_node_index)
        {
            if edge_kind == edgeref.weight().kind().into() {
                edges_to_remove.push(edgeref.id());
            }
        }
        for edge_to_remove in edges_to_remove {
            self.graph.remove_edge(edge_to_remove);
        }

        if let Some(previous_container_ordering_node_index) =
            self.ordering_node_index_for_container(new_source_node_index)?
        {
            let old_target_node_weight = self
                .graph
                .node_weight(target_node_index)
                .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;
            if let NodeWeight::Ordering(previous_container_ordering_node_weight) = self
                .graph
                .node_weight(previous_container_ordering_node_index)
                .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?
            {
                let mut new_container_ordering_node_weight =
                    previous_container_ordering_node_weight.clone();
                let old_target_id = old_target_node_weight.id();
                let mut new_order = new_container_ordering_node_weight.order().clone();
                new_order.retain(|id| *id != old_target_id);

                // We only want to update the ordering of the container if we removed an edge to
                // one of the ordered relationships.
                if &new_order != previous_container_ordering_node_weight.order() {
                    new_container_ordering_node_weight.set_order(change_set, new_order)?;

                    let new_container_ordering_node_index =
                        self.add_node(NodeWeight::Ordering(new_container_ordering_node_weight))?;
                    updated.extend(self.replace_references(
                        previous_container_ordering_node_index,
                        new_container_ordering_node_index,
                    )?);
                }
            }
        }

        let new_source_node_index =
            self.get_node_index_by_id(self.get_node_weight(new_source_node_index)?.id())?;

        let mut work_queue = VecDeque::from([new_source_node_index]);

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

        Ok(updated)
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

    pub fn replace_references(
        &mut self,
        original_node_index: NodeIndex,
        new_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<HashMap<NodeIndex, NodeIndex>> {
        let mut old_to_new_node_indices: HashMap<NodeIndex, NodeIndex> = HashMap::new();
        old_to_new_node_indices.insert(original_node_index, new_node_index);

        // Create a post order traversal work queue by starting with the original
        // node index in front.
        let mut work_queue = VecDeque::new();
        work_queue.push_front(original_node_index);

        // Push all sources of incoming edges that have a path to the root to the back to ensure
        // post order traversal.
        for edgeref in self.edges_directed(original_node_index, Incoming) {
            let source_for_incoming_edge = edgeref.source();
            if self.is_on_path_between(
                self.root_index,
                original_node_index,
                source_for_incoming_edge,
            ) {
                work_queue.push_back(source_for_incoming_edge);
            }
        }

        while let Some(old_node_index) = work_queue.pop_front() {
            // Only process nodes that have a path to root.
            if !self.has_path_to_root(old_node_index) {
                continue;
            }

            // Check if we have incoming edges and push them to the back to ensure we
            // continue to perform post order traversal.
            for edgeref in self.edges_directed(old_node_index, Direction::Incoming) {
                work_queue.push_back(edgeref.source());
            }

            // Copy the node if we have not seen it or grab it if we have. Only the first node in DFS post order
            // traversal should already exist since it was created before we entered `replace_references`, and
            // is the reason we're updating things in the first place.
            let new_node_index = match old_to_new_node_indices.get(&old_node_index) {
                Some(found_new_node_index) => *found_new_node_index,
                None => {
                    let new_node_index = self.copy_node_by_index(old_node_index)?;
                    old_to_new_node_indices.insert(old_node_index, new_node_index);
                    new_node_index
                }
            };

            // Find all outgoing edges weights and find the edge targets.
            let mut edges_to_create: Vec<(EdgeWeight, NodeIndex)> = Vec::new();
            for edge_reference in self.graph.edges_directed(old_node_index, Outgoing) {
                edges_to_create.push((edge_reference.weight().clone(), edge_reference.target()));
            }

            // Make copies of these edges where the source is the new node index and the
            // destination is one of the following...
            // - If an entry exists in `old_to_new_node_indices` for the destination node index,
            //   use the value of the entry (the destination was affected by the replacement,
            //   and needs to use the new node index to reflect this).
            // - There is no entry in `old_to_new_node_indices`; use the same destination node
            //   index as the old edge (the destination was *NOT* affected by the replacement,
            //   and does not have any new information to reflect).
            for (edge_weight, destination_node_index) in edges_to_create {
                // Need to directly add the edge, without going through `self.add_edge` to avoid
                // infinite recursion, and because we're the place doing all the book keeping
                // that we'd be interested in happening from `self.add_edge`.
                self.graph.update_edge(
                    new_node_index,
                    old_to_new_node_indices
                        .get(&destination_node_index)
                        .copied()
                        .unwrap_or(destination_node_index),
                    edge_weight,
                );
            }

            self.update_merkle_tree_hash(new_node_index)?;
        }

        // Use the new version of the old root node as our root node.
        if let Some(new_root_node_index) = old_to_new_node_indices.get(&self.root_index) {
            self.root_index = *new_root_node_index;
        }

        // Before returning, remove the root from the map because we should always "ask" what the
        // root is rather than relying on a potentially stale reference.
        old_to_new_node_indices.remove(&self.root_index);

        Ok(old_to_new_node_indices)
    }

    pub fn update_content(
        &mut self,
        change_set: &ChangeSetPointer,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let original_node_index = self.get_node_index_by_id(id)?;
        let new_node_index = self.copy_node_by_index(original_node_index)?;
        let node_weight = self.get_node_weight_mut(new_node_index)?;
        node_weight.increment_vector_clock(change_set)?;
        node_weight.new_content_hash(new_content_hash)?;

        self.replace_references(original_node_index, new_node_index)?;
        Ok(())
    }

    pub fn update_order(
        &mut self,
        change_set: &ChangeSetPointer,
        container_id: Ulid,
        new_order: Vec<Ulid>,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let original_node_index = self
            .ordering_node_index_for_container(self.get_node_index_by_id(container_id)?)?
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;
        let new_node_index = self.copy_node_by_index(original_node_index)?;
        let node_weight = self.get_node_weight_mut(new_node_index)?;
        node_weight.set_order(change_set, new_order)?;

        self.replace_references(original_node_index, new_node_index)?;
        Ok(())
    }

    fn update_merkle_tree_hash(
        &mut self,
        node_index_to_update: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let mut hasher = ContentHash::hasher();
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
                self.graph
                    .node_weight(neighbor_node)
                    .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?
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

                    // This is the kind of the action.
                    EdgeWeightKind::ActionPrototype(kind) => {
                        hasher.update(kind.to_string().as_bytes())
                    }

                    // This is the key representing an element in a container type corresponding
                    // to an AttributePrototype
                    EdgeWeightKind::Prototype(Some(key)) => hasher.update(key.as_bytes()),

                    // Nothing to do, as these EdgeWeightKind do not encode extra information
                    // in the edge itself.
                    EdgeWeightKind::Contain(None)
                    | EdgeWeightKind::InterComponent
                    | EdgeWeightKind::PrototypeArgument
                    | EdgeWeightKind::PrototypeArgumentValue
                    | EdgeWeightKind::Provider
                    | EdgeWeightKind::Ordering
                    | EdgeWeightKind::Prop
                    | EdgeWeightKind::Prototype(None)
                    | EdgeWeightKind::Proxy
                    | EdgeWeightKind::Root
                    | EdgeWeightKind::Use => {}
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

    /// Perform [`Updates`](Update) using [`self`](WorkspaceSnapshotGraph) as the "to rebase" graph
    /// and a provided graph as the "onto" graph.
    pub fn perform_updates(
        &mut self,
        to_rebase_change_set: &ChangeSetPointer,
        onto: &WorkspaceSnapshotGraph,
        updates: &[Update],
    ) -> WorkspaceSnapshotGraphResult<()> {
        let mut updated = NodeIndexMap::new();
        for update in updates {
            match update {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } => {
                    let updated_source = *updated.get(source).unwrap_or(source);
                    let destination =
                        self.find_in_self_or_create_using_onto(*destination, &mut updated, onto)?;
                    let new_edge_index =
                        self.add_edge(updated_source, edge_weight.clone(), destination)?;
                    let (new_source, _) = self.edge_endpoints(new_edge_index)?;
                    updated.insert(*source, new_source);
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } => {
                    let updated_source = *updated.get(source).unwrap_or(source);
                    let destination = *updated.get(destination).unwrap_or(destination);
                    updated.extend(self.remove_edge(
                        to_rebase_change_set,
                        updated_source,
                        destination,
                        *edge_kind,
                    )?);
                    if let Some(source_remapped) = updated.get(&updated_source).copied() {
                        updated.insert(*source, source_remapped);
                    }
                }
                Update::ReplaceSubgraph {
                    onto: onto_subgraph_root,
                    to_rebase: to_rebase_subgraph_root,
                } => {
                    let updated_to_rebase = *updated
                        .get(to_rebase_subgraph_root)
                        .unwrap_or(to_rebase_subgraph_root);
                    let new_subgraph_root = self.find_in_self_or_create_using_onto(
                        *onto_subgraph_root,
                        &mut updated,
                        onto,
                    )?;
                    updated.extend(self.replace_references(updated_to_rebase, new_subgraph_root)?);
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

    /// Find in self where self is the "to rebase" side or create using "onto".
    fn find_in_self_or_create_using_onto(
        &mut self,
        unchecked: NodeIndex,
        updated: &mut NodeIndexMap,
        onto: &WorkspaceSnapshotGraph,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let found_or_created = match updated.get(&unchecked) {
            Some(found) => *found,
            None => {
                let unchecked_node_weight = onto.get_node_weight(unchecked)?;
                match self.find_equivalent_node(
                    unchecked_node_weight.id(),
                    unchecked_node_weight.lineage_id(),
                )? {
                    Some(found_equivalent_node) => {
                        let found_equivalent_node_weight =
                            self.get_node_weight(found_equivalent_node)?;
                        if found_equivalent_node_weight.merkle_tree_hash()
                            != unchecked_node_weight.merkle_tree_hash()
                        {
                            updated.extend(self.import_subgraph(onto, unchecked)?);

                            *updated.get(&unchecked).ok_or(
                                WorkspaceSnapshotGraphError::DestinationNotUpdatedWhenImportingSubgraph,
                            )?
                        } else {
                            updated.insert(unchecked, found_equivalent_node);

                            found_equivalent_node
                        }
                    }
                    None => {
                        updated.extend(self.import_subgraph(onto, unchecked)?);
                        *updated.get(&unchecked).ok_or(
                            WorkspaceSnapshotGraphError::DestinationNotUpdatedWhenImportingSubgraph,
                        )?
                    }
                }
            }
        };
        Ok(found_or_created)
    }
}

fn ordering_node_indexes_for_node_index(
    snapshot: &WorkspaceSnapshotGraph,
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
    snapshot: &WorkspaceSnapshotGraph,
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
