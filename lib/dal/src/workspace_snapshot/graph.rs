use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::Write;

use chrono::Utc;
/// Ensure [`NodeIndex`] is usable by external crates.
pub use petgraph::graph::NodeIndex;
use petgraph::stable_graph::Edges;
pub use petgraph::Direction;
use petgraph::{algo, prelude::*, visit::DfsEvent};
use serde::{Deserialize, Serialize};
use si_events::merkle_tree_hash::MerkleTreeHash;
use si_events::{ulid::Ulid, ContentHash};
use thiserror::Error;

use telemetry::prelude::*;

use crate::change_set::{ChangeSet, ChangeSetError};
use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{CategoryNodeWeight, NodeWeightDiscriminants};
use crate::workspace_snapshot::vector_clock::VectorClockId;
use crate::workspace_snapshot::{
    conflict::Conflict,
    content_address::ContentAddress,
    edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants},
    node_weight::{NodeWeight, NodeWeightError, OrderingNodeWeight},
    update::Update,
};

mod tests;

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
    ChangeSet(#[from] ChangeSetError),
    #[error("Unable to retrieve content for ContentHash")]
    ContentMissingForContentHash,
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
    pub fn new(change_set: &ChangeSet) -> WorkspaceSnapshotGraphResult<Self> {
        let mut graph: StableDiGraph<NodeWeight, EdgeWeight> = StableDiGraph::with_capacity(1, 0);
        let root_node = NodeWeight::new_content(
            change_set,
            change_set.generate_ulid()?,
            ContentAddress::Root,
        )?;

        let node_id = root_node.id();
        let lineage_id = root_node.lineage_id();
        let root_index = graph.add_node(root_node);

        let mut result = Self {
            root_index,
            graph,
            ..Default::default()
        };

        result.add_node_finalize(node_id, lineage_id, root_index)?;

        Ok(result)
    }

    pub fn root(&self) -> NodeIndex {
        self.root_index
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
        change_set: &ChangeSet,
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
            let maybe_category_node_weight = self.get_node_weight(maybe_category_node_index)?;

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
                .ok()
                .flatten()
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
        change_set: &ChangeSet,
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
                EdgeWeight::new(change_set, EdgeWeightKind::Ordinal)?,
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
                new_container_ordering_node_weight.push_to_order(change_set, element_id)?;
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
        change_set: &ChangeSet,
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
    #[instrument(level = "info", skip_all)]
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
                    error!(
                        "Unable to get NodeWeight for onto NodeIndex {:?}: {}",
                        onto_node_index, err,
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
                }

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
                            onto_node_weight.merkle_tree_hash(),
                            to_rebase_node_weight.merkle_tree_hash()
                        );
                        continue;
                    }
                    any_content_with_lineage_has_changed = true;

                    // Check if there's a difference in the node itself (and whether it is a
                    // conflict if there is a difference).
                    if onto_node_weight.node_hash() != to_rebase_node_weight.node_hash() {
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

                            if let (
                                NodeWeight::Ordering(onto_ordering),
                                NodeWeight::Ordering(to_rebase_ordering),
                            ) = (onto_node_weight, to_rebase_node_weight)
                            {
                                // TODO Checking if two ordering arrays are non conflicting
                                // (if the common elements between two ordering have the same relative positions)
                                // is logic that could be extracted into its own thing. The following code does that

                                // Both `onto` and `to_rebase` have changes that the other has not incorporated. We
                                // need to find out what the changes are to see what needs to be updated, and what
                                // conflicts.
                                let onto_ordering_set: HashSet<Ulid> =
                                    onto_ordering.order().iter().copied().collect();
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
                                        to_rebase: to_rebase_node_index,
                                        onto: onto_node_index,
                                    });
                                }
                            } else {
                                conflicts.push(Conflict::NodeContent {
                                    to_rebase: to_rebase_node_index,
                                    onto: onto_node_index,
                                });
                            }
                        }
                    }

                    let (container_conflicts, container_updates) = self
                        .find_container_membership_conflicts_and_updates(
                            to_rebase_vector_clock_id,
                            to_rebase_node_index,
                            &[],
                            onto,
                            onto_vector_clock_id,
                            onto_node_index,
                            &[],
                        )
                        .map_err(|err| {
                            error!("Unable to find container membership conflicts and updates for onto container NodeIndex {:?} and to_rebase container NodeIndex {:?}: {}", onto_node_index, to_rebase_node_index, err);
                            event
                        })?;

                    updates.extend(container_updates);
                    conflicts.extend(container_conflicts);
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
        let current_root_weight = self
            .get_node_weight(self.root_index)
            .expect("this should be impossible and this code should only be used for debugging");
        println!(
            "Root Node Weight: {current_root_weight:?}\n{:?}",
            petgraph::dot::Dot::with_config(&self.graph, &[petgraph::dot::Config::EdgeNoLabel])
        );
    }

    #[allow(dead_code, clippy::disallowed_methods)]
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
                    EdgeWeightKindDiscriminants::ActionRunsAfter => "blue",
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
                };
                let color = color.to_string();
                let id = node_weight.id();
                format!(
                    "label = \"\n\n{label}\n{node_index:?}\n{id}\n\n{:?}\"\nfontcolor = {color}\ncolor = {color}", node_weight.merkle_tree_hash()
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

    #[allow(clippy::too_many_arguments)]
    fn find_container_membership_conflicts_and_updates(
        &self,
        to_rebase_vector_clock_id: VectorClockId,
        to_rebase_container_index: NodeIndex,
        to_rebase_container_order: &[Ulid],
        onto: &WorkspaceSnapshotGraph,
        onto_vector_clock_id: VectorClockId,
        onto_container_index: NodeIndex,
        onto_container_order: &[Ulid],
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

            if to_rebase_container_order.contains(&target_node_weight.id()) {
                continue;
            }

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

            if onto_container_order.contains(&target_node_weight.id()) {
                continue;
            }

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
                .get_edge_weight_opt(only_to_rebase_edge_info.edge_index)?
                .ok_or(WorkspaceSnapshotGraphError::EdgeWeightNotFound)?;
            let to_rebase_item_weight =
                self.get_node_weight(only_to_rebase_edge_info.target_node_index)?;

            if to_rebase_edge_weight
                .vector_clock_first_seen()
                .entry_for(to_rebase_vector_clock_id)
                <= onto_last_saw_to_rebase
            {
                // Onto has seen this edge before, but the edge doesn't exist in
                // onto. If the container has been modified since we last saw
                // it, then we have a conflict. Otherwise, we should remove the
                // edge.
                if to_rebase_item_weight
                    .vector_clock_write()
                    .entry_for(to_rebase_vector_clock_id)
                    > onto_last_saw_to_rebase
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
                // Onto has never seen this edge, so it was added by another
                // rebase. Conflict here if the node weight specifies this edge
                // as exclusive to us.
                let container_weight = self.get_node_weight(to_rebase_container_index)?;
                let edge_kind = only_to_rebase_edge_info.edge_kind;
                if container_weight.is_exclusive_outgoing_edge(edge_kind) {
                    conflicts.push(Conflict::ExclusiveEdgeMismatch {
                        source: only_to_rebase_edge_info.source_node_index,
                        destination: only_to_rebase_edge_info.target_node_index,
                        edge_kind,
                    });
                }
            }
        }

        // - Items unique to `onto`:
        for only_onto_edge_info in only_onto_edges.values() {
            let onto_edge_weight = onto
                .get_edge_weight_opt(only_onto_edge_info.edge_index)?
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

    #[inline(always)]
    pub(crate) fn try_get_node_index_by_id(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotGraphResult<Option<NodeIndex>> {
        let id = id.into();

        Ok(self.node_index_by_id.get(&id).copied())
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

    pub fn get_node_weight_opt(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<&NodeWeight>> {
        Ok(self.graph.node_weight(node_index))
    }

    pub fn get_node_weight(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<&NodeWeight> {
        self.get_node_weight_opt(node_index)?
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

    pub fn get_edge_weight_opt(
        &self,
        edge_index: EdgeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<&EdgeWeight>> {
        Ok(self.graph.edge_weight(edge_index))
    }

    fn has_path_to_root(&self, node: NodeIndex) -> bool {
        algo::has_path_connecting(&self.graph, self.root_index, node, None)
    }

    pub fn import_subgraph(
        &mut self,
        other: &WorkspaceSnapshotGraph,
        root_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
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

                    self.replace_references(equivalent_node_index)?;
                    self.get_latest_node_idx(new_node_index)?
                }
            } else {
                self.add_node(node_weight_to_copy)?
            };

            for edge in other.graph.edges_directed(node_index_to_copy, Outgoing) {
                let target_id = other.get_node_weight(edge.target())?.id();
                let latest_target = self.get_node_index_by_id(target_id)?;
                self.graph
                    .update_edge(node_index, latest_target, edge.weight().clone());
            }
        }

        Ok(())
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
                    ordered_child_indexes.push(
                        *self
                            .node_index_by_id
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

    pub fn ordering_node_for_container(
        &self,
        container_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<OrderingNodeWeight>> {
        Ok(
            match self.ordering_node_index_for_container(container_node_index)? {
                Some(ordering_node_idx) => match self.get_node_weight_opt(ordering_node_idx)? {
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

    pub(crate) fn remove_node(&mut self, node_index: NodeIndex) {
        self.graph.remove_node(node_index);
    }

    /// [`StableGraph`] guarantees the stability of [`NodeIndex`] across removals, however there
    /// are **NO** guarantees around the stability of [`EdgeIndex`] across removals. If
    /// [`Self::cleanup()`] has been called, then any [`EdgeIndex`] found before
    /// [`Self::cleanup()`] has run should be considered invalid.
    pub(crate) fn remove_edge(
        &mut self,
        change_set: &ChangeSet,
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

        self.inner_remove_edge(source_node_index, target_node_index, edge_kind);

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
                if new_container_ordering_node_weight.remove_from_order(change_set, element_id)? {
                    self.inner_remove_edge(
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

    fn inner_remove_edge(
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
        change_set: &ChangeSet,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let original_node_index = self.get_node_index_by_id(id)?;
        let new_node_index = self.copy_node_by_index(original_node_index)?;
        let node_weight = self.get_node_weight_mut(new_node_index)?;
        node_weight.increment_vector_clock(change_set)?;
        node_weight.new_content_hash(new_content_hash)?;

        self.replace_references(original_node_index)?;
        Ok(())
    }

    pub fn update_order(
        &mut self,
        change_set: &ChangeSet,
        container_id: Ulid,
        new_order: Vec<Ulid>,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let original_node_index = self
            .ordering_node_index_for_container(self.get_node_index_by_id(container_id)?)?
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;
        let new_node_index = self.copy_node_by_index(original_node_index)?;
        let node_weight = self.get_node_weight_mut(new_node_index)?;
        node_weight.set_order(change_set, new_order)?;

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
                    | EdgeWeightKind::ActionRunsAfter
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

    /// Perform [`Updates`](Update) using [`self`](WorkspaceSnapshotGraph) as the "to rebase" graph
    /// and a provided graph as the "onto" graph.
    pub fn perform_updates(
        &mut self,
        to_rebase_change_set: &ChangeSet,
        onto: &WorkspaceSnapshotGraph,
        updates: &[Update],
    ) -> WorkspaceSnapshotGraphResult<()> {
        for update in updates {
            match update {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } => {
                    let updated_source = self.get_latest_node_idx(*source)?;
                    let destination = self.find_in_self_or_create_using_onto(*destination, onto)?;

                    self.add_edge(updated_source, edge_weight.clone(), destination)?;
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } => {
                    let updated_source = self.get_latest_node_idx(*source)?;
                    let destination = self.get_latest_node_idx(*destination)?;
                    self.remove_edge(
                        to_rebase_change_set,
                        updated_source,
                        destination,
                        *edge_kind,
                    )?;
                }
                Update::ReplaceSubgraph {
                    onto: onto_subgraph_root,
                    to_rebase: to_rebase_subgraph_root,
                } => {
                    let updated_to_rebase = self.get_latest_node_idx(*to_rebase_subgraph_root)?;
                    self.find_in_self_or_create_using_onto(*onto_subgraph_root, onto)?;
                    self.replace_references(updated_to_rebase)?;
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

    /// Given the node index for a node in other, find if a node exists in self that has the same
    /// id as the node found in other.
    fn find_latest_idx_in_self_from_other_idx(
        &mut self,
        other: &WorkspaceSnapshotGraph,
        other_idx: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<NodeIndex>> {
        let other_id = other.get_node_weight(other_idx)?.id();

        Ok(self.get_node_index_by_id(other_id).ok())
    }

    /// Find in self where self is the "to rebase" side or create using "onto".
    fn find_in_self_or_create_using_onto(
        &mut self,
        unchecked: NodeIndex,
        onto: &WorkspaceSnapshotGraph,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let unchecked_node_weight = onto.get_node_weight(unchecked)?;

        let found_or_created = {
            let equivalent_node = if let Some(found) =
                self.find_latest_idx_in_self_from_other_idx(onto, unchecked)?
            {
                Some(found)
            } else {
                self.find_equivalent_node(
                    unchecked_node_weight.id(),
                    unchecked_node_weight.lineage_id(),
                )?
            };

            match equivalent_node {
                Some(found_equivalent_node) => {
                    let found_equivalent_node_weight =
                        self.get_node_weight(found_equivalent_node)?;
                    if found_equivalent_node_weight.merkle_tree_hash()
                        != unchecked_node_weight.merkle_tree_hash()
                    {
                        self.import_subgraph(onto, unchecked)?;
                        self.find_latest_idx_in_self_from_other_idx(onto, unchecked)?
                            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?
                    } else {
                        found_equivalent_node
                    }
                }
                None => {
                    self.import_subgraph(onto, unchecked)?;
                    self.find_latest_idx_in_self_from_other_idx(onto, unchecked)?
                        .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?
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
