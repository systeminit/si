use petgraph::prelude::*;
use petgraph::{algo, stable_graph::EdgeReference};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::workspace_snapshot::{
    change_set::ChangeSet,
    conflict::{Conflict, ConflictLocation},
    edge_weight::{EdgeWeight, EdgeWeightKind},
    node_weight::{ContentAddress, NodeWeight, NodeWeightError, OrderingNodeWeight},
    WorkspaceSnapshotError, WorkspaceSnapshotResult,
};
use crate::ContentHash;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum WorkspaceSnapshotGraphError {
    #[error("Cannot compare ordering of container elements between ordered, and un-ordered container: {0:?}, {1:?}")]
    CannotCompareOrderedAndUnorderedContainers(NodeIndex, NodeIndex),
    #[error("Problem during graph traversal: {0:?}")]
    GraphTraversal(petgraph::visit::DfsEvent<NodeIndex>),
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("node weight not found")]
    NodeWeightNotFound,
    #[error("NodeIndex has too many Ordering children: {0:?}")]
    TooManyOrderingForNode(NodeIndex),
}

pub type WorkspaceSnapshotGraphResult<T> = Result<T, WorkspaceSnapshotGraphError>;

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct WorkspaceSnapshotGraph {
    root_index: NodeIndex,
    graph: StableDiGraph<NodeWeight, EdgeWeight>,
}

impl std::fmt::Debug for WorkspaceSnapshotGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceSnapshotGraph")
            .field("root_index", &self.root_index)
            .field("graph", &self.graph)
            .finish()
    }
}

impl WorkspaceSnapshotGraph {
    pub fn new(change_set: &ChangeSet) -> WorkspaceSnapshotGraphResult<Self> {
        let mut graph: StableDiGraph<NodeWeight, EdgeWeight> = StableDiGraph::with_capacity(1, 0);
        let root_index = graph.add_node(NodeWeight::new_content_with_seen_vector_clock(
            change_set,
            ContentAddress::Root,
        )?);

        Ok(Self { root_index, graph })
    }

    fn is_acyclic_directed(&self) -> bool {
        // Using this because "is_cyclic_directed" is recursive.
        algo::toposort(&self.graph, None).is_ok()
    }

    pub fn cleanup(&mut self) {
        self.graph.retain_nodes(|frozen_graph, current_node| {
            // We cannot use "has_path_to_root" because we need to use the Frozen<StableGraph<...>>.
            algo::has_path_connecting(&*frozen_graph, self.root_index, current_node, None)
        });
    }

    fn dot(&self) {
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

    fn add_node(&mut self, node: NodeWeight) -> WorkspaceSnapshotResult<NodeIndex> {
        let new_node_index = self.graph.add_node(node);
        self.update_merkle_tree_hash(new_node_index)?;

        Ok(new_node_index)
    }

    pub fn add_edge(
        &mut self,
        change_set: &ChangeSet,
        from_node_index: NodeIndex,
        mut edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<EdgeIndex> {
        // Temporarily add the edge to the existing tree to see if it would create a cycle.
        let temp_edge = self
            .graph
            .update_edge(from_node_index, to_node_index, edge_weight.clone());
        let would_create_a_cycle = !self.is_acyclic_directed();
        self.graph.remove_edge(temp_edge);
        if would_create_a_cycle {
            return Err(WorkspaceSnapshotError::CreateGraphCycle);
        }

        // Ensure the vector clocks of the edge are up-to-date.
        edge_weight.increment_vector_clocks(change_set)?;

        // Because outgoing edges are part of a node's identity, we create a new "from" node
        // as we are effectively writing to that node (we'll need to update the merkle tree
        // hash), and everything in the graph should be treated as copy-on-write.
        let new_from_node_index = self.copy_node_index(change_set, from_node_index)?;

        // Add the new edge to the new version of the "from" node.
        let new_edge_index =
            self.graph
                .update_edge(new_from_node_index, to_node_index, edge_weight);
        self.update_merkle_tree_hash(new_from_node_index)?;

        // Update the rest of the graph to reflect the new node/edge.
        self.replace_references(change_set, from_node_index, new_from_node_index)?;

        Ok(new_edge_index)
    }

    fn get_node_index_by_id(&self, id: Ulid) -> WorkspaceSnapshotResult<NodeIndex> {
        for node_index in self.graph.node_indices() {
            // It's possible that there are multiple nodes in the petgraph that have the
            // same ID as the one we're interested in, as we may not yet have cleaned up
            // nodes/edges representing "old" versions when we're making changes. There
            // should only be one in the sub-graph starting at `self.root_index`,
            // however, and this represents the current state of the workspace after all
            // changes have been made.
            if self.has_path_to_root(node_index) {
                let node_weight = self.get_node_weight(node_index)?;
                if node_weight.id() == id {
                    return Ok(node_index);
                }
            }
        }

        Err(WorkspaceSnapshotError::NodeWithIdNotFound(id))
    }

    fn copy_node_index(
        &mut self,
        change_set: &ChangeSet,
        node_index_to_copy: NodeIndex,
    ) -> WorkspaceSnapshotResult<NodeIndex> {
        let new_node_index = self.graph.add_node(
            self.get_node_weight(node_index_to_copy)?
                .new_with_incremented_vector_clocks(change_set)?,
        );

        Ok(new_node_index)
    }

    pub fn update_content(
        &mut self,
        change_set: &ChangeSet,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotResult<()> {
        let original_node_index = self.get_node_index_by_id(id)?;
        let new_node_index = self.copy_node_index(change_set, original_node_index)?;
        let node_weight = self.get_node_weight_mut(new_node_index)?;
        node_weight.new_content_hash(new_content_hash)?;

        self.replace_references(change_set, original_node_index, new_node_index)
    }

    fn has_path_to_root(&self, node: NodeIndex) -> bool {
        algo::has_path_connecting(&self.graph, self.root_index, node, None)
    }

    fn is_on_path_between(&self, start: NodeIndex, end: NodeIndex, node: NodeIndex) -> bool {
        algo::has_path_connecting(&self.graph, start, node, None)
            && algo::has_path_connecting(&self.graph, node, end, None)
    }

    fn get_node_weight(&self, node_index: NodeIndex) -> WorkspaceSnapshotGraphResult<&NodeWeight> {
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

    fn replace_references(
        &mut self,
        change_set: &ChangeSet,
        original_node_index: NodeIndex,
        new_node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<()> {
        let mut old_to_new_node_indices: HashMap<NodeIndex, NodeIndex> = HashMap::new();
        old_to_new_node_indices.insert(original_node_index, new_node_index);

        let mut dfspo = DfsPostOrder::new(&self.graph, self.root_index);
        while let Some(old_node_index) = dfspo.next(&self.graph) {
            // All nodes that exist between the root and the `original_node_index` are affected by the replace, and only
            // those nodes are affected, because the replacement affects their merkel tree hashes.
            if self.is_on_path_between(self.root_index, original_node_index, old_node_index) {
                // Copy the node if we have not seen it or grab it if we have. Only the first node in DFS post order
                // traversal should already exist since it was created before we entered `replace_references`, and
                // is the reason we're updating things in the first place.
                let new_node_index = match old_to_new_node_indices.get(&old_node_index) {
                    Some(found_new_node_index) => *found_new_node_index,
                    None => {
                        let new_node_index = self.copy_node_index(change_set, old_node_index)?;
                        old_to_new_node_indices.insert(old_node_index, new_node_index);
                        new_node_index
                    }
                };

                // Find all outgoing edges. From those outgoing edges and find their destinations.
                // If they do not have destinations, then there is no work to do (i.e. stale edge
                // reference, which should only happen if an edge was removed after we got the
                // edge ref, but before we asked about the edge's endpoints).
                let mut edges_to_create: Vec<(EdgeWeight, NodeIndex)> = Vec::new();
                for edge_reference in self.graph.edges_directed(old_node_index, Outgoing) {
                    let edge_weight = edge_reference.weight();
                    if let Some((_, destination_node_index)) =
                        self.graph.edge_endpoints(edge_reference.id())
                    {
                        edges_to_create.push((
                            edge_weight.new_with_incremented_vector_clocks(change_set)?,
                            destination_node_index,
                        ));
                    }
                }

                // Make copies of these edges where the source is the new node index and the
                // destination is one of the following...
                // - If an entry exists in `old_to_new_node_indicies` for the destination node index,
                //   use the value of the entry (the destination was affected by the replacement,
                //   and needs to use the new node index to reflect this).
                // - There is no entry in `old_to_new_node_indicies`; use the same destination node
                //   index as the old edge (the destination was *NOT* affected by the replacemnt,
                //   and does not have any new information to reflect).
                for (edge_weight, destination_node_index) in edges_to_create {
                    // Need to directly add the edge, without going through `self.add_edge` to avoid
                    // infinite recursion, and because we're the place doing all the book keeping
                    // that we'd be interested in happening from `self.add_edge`.
                    self.graph.update_edge(
                        new_node_index,
                        *old_to_new_node_indices
                            .get(&destination_node_index)
                            .unwrap_or(&destination_node_index),
                        edge_weight,
                    );
                }

                self.update_merkle_tree_hash(new_node_index)?;

                // Use the new version of the old root node as our root node.
                if let Some(new_root_node_index) = old_to_new_node_indices.get(&self.root_index) {
                    self.root_index = *new_root_node_index;
                }
            }
        }

        Ok(())
    }

    fn update_merkle_tree_hash(
        &mut self,
        node_index_to_update: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let mut hasher = ContentHash::hasher();
        hasher.update(
            self.get_node_weight(node_index_to_update)?
                .content_hash()
                .to_string()
                .as_bytes(),
        );

        // Need to make sure the neighbors are added to the hash in a stable order to ensure the
        // merkle tree hash is identical for identical trees.
        let mut ordered_neighbors = Vec::new();
        for neighbor_node in self
            .graph
            .neighbors_directed(node_index_to_update, Outgoing)
        {
            ordered_neighbors.push(neighbor_node);
        }
        ordered_neighbors.sort();

        for neighbor_node in ordered_neighbors {
            hasher.update(
                self.graph
                    .node_weight(neighbor_node)
                    .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?
                    .merkle_tree_hash()
                    .to_string()
                    .as_bytes(),
            );
        }

        let new_node_weight = self
            .graph
            .node_weight_mut(node_index_to_update)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;
        new_node_weight.set_merkle_tree_hash(hasher.finalize());

        Ok(())
    }

    pub fn merge(
        &mut self,
        change_set: &ChangeSet,
        other: &WorkspaceSnapshotGraph,
        other_change_set: &ChangeSet,
    ) -> WorkspaceSnapshotResult<()> {
        let local_root_write_vector_clock =
            self.get_node_weight(self.root_index)?.vector_clock_write();
        let remote_root_write_vector_clock = other
            .get_node_weight(other.root_index)?
            .vector_clock_write();

        if local_root_write_vector_clock.is_newer_than(remote_root_write_vector_clock) {
            // We're strictly newer than `other`, which means that we can fast forward by incorporating new
            // vector clock entries in `other` (which should only be in the "seen" clock), and incrementing
            // our "seen" vector clock entry.
            //
            // Create a new root node with merged "seen" vector clocks, incrementing the entry for `change_set`.
            // Do *NOT* increment the "write" vector clock, since we have not changed anything about the content
            // of the graph.
            let mut new_root_node_weight = self.get_node_weight(self.root_index)?.clone();
            new_root_node_weight
                .merge_clocks(change_set, other.get_node_weight(other.root_index)?)?;
            new_root_node_weight.increment_seen_vector_clock(change_set)?;
            let new_root_index = self.add_node(new_root_node_weight)?;

            self.replace_references(change_set, self.root_index, new_root_index)?;
        } else if remote_root_write_vector_clock.is_newer_than(local_root_write_vector_clock) {
            // `local` is not "newer" than `other` and `other` *IS* "newer" than `local`, which means that we
            // can fast-forward `local` to the state of `other`, incorporate the `local` vector clock entries
            // into `other` (which should only be in the "seen" clock), and increment our "seen" vector clock
            // entry.
            let mut new_root_node_weight = other.get_node_weight(other.root_index)?.clone();
            new_root_node_weight
                .merge_clocks(change_set, self.get_node_weight(self.root_index)?)?;
            new_root_node_weight.increment_seen_vector_clock(change_set)?;

            self.graph = other.graph.clone();
            self.root_index = other.root_index;

            let new_root_node_index = self.add_node(new_root_node_weight)?;
            self.replace_references(change_set, self.root_index, new_root_node_index)?;
        } else if !self
            .find_conflicts(change_set, other, other_change_set)?
            .is_empty()
        {
            return Err(WorkspaceSnapshotError::WorkspaceNeedsRebase);
        } else {
            // No conflicts; do the merge!

            todo!();
        }

        // `other` has newer/more entries than we have:
        //   - If we also have newer/more entries than `other`, both sides have changed, and we need to see if there's a conflict.

        Ok(())
    }

    pub fn find_conflicts(
        &self,
        change_set: &ChangeSet,
        other: &WorkspaceSnapshotGraph,
        other_change_set: &ChangeSet,
    ) -> WorkspaceSnapshotGraphResult<Vec<Conflict>> {
        // `self`/`local` is the base change set. `other` is the change set to merge in.
        // Both `local` and `other` have (write) entries that the other does not. Figure out if the
        // trees can be merged together without any conflicts.
        let mut conflicts: Vec<Conflict> = Vec::new();
        let result =
            petgraph::visit::depth_first_search(&other.graph, Some(other.root_index), |event| {
                match event {
                    petgraph::visit::DfsEvent::Discover(to_merge_node_index, _) => {
                        // Check if `base` has a node with the same `ID`
                        let to_merge_node_weight = other
                            .get_node_weight(to_merge_node_index)
                            .map_err(|_| event)?;
                        let base_node_index =
                            match self.get_node_index_by_id(to_merge_node_weight.id()) {
                                Ok(ni) => ni,
                                // If there isn't a node with the same ID in `base` then it's a "new" node.
                                Err(WorkspaceSnapshotError::NodeWithIdNotFound(_)) => {
                                    // TODO: This isn't right. We should be checking why it's only in `other` (the branch to merge). Was it deleted in `base`, and if so, has it been modified at all in `other`?
                                    return Ok(petgraph::visit::Control::Continue);
                                }
                                // Something else went wrong; we should probably bail out.
                                Err(_) => return Err(event),
                            };
                        let base_node_weight =
                            self.get_node_weight(base_node_index).map_err(|_| event)?;

                        if to_merge_node_weight.merkle_tree_hash()
                            == base_node_weight.merkle_tree_hash()
                        {
                            // These (sub-)graphs have the same content, so there can be no conflicts
                            // from this point towards the leaves.
                            return Ok(petgraph::visit::Control::Prune);
                        }

                        if let (
                            NodeWeight::Content(base_content_weight),
                            NodeWeight::Content(to_merge_content_weight),
                        ) = (base_node_weight, to_merge_node_weight)
                        {
                            // If the content of the node itself is the same in `base` and `other`, then that
                            // means that something has changed about the node's children. This could be an
                            // added or removed relationship, a change in the ordering of the children, or
                            // that one of the child nodes itself (or one of its descendents) has changed,
                            // or all of these. If there are added/removed relationships, or re-ordered
                            // children on both sides, then that should be considered as a conflict on the
                            // container node itself.
                            if to_merge_content_weight.content_address()
                                == base_content_weight.content_address()
                            {
                                if let Some(container_membership_conflict) = self
                                    .has_container_membership_conflict(
                                        base_node_index,
                                        other,
                                        to_merge_node_index,
                                    )
                                    .map_err(|_| event)?
                                {
                                    conflicts.push(container_membership_conflict);
                                }
                            } else {
                                // There's a difference in the content of the node itself, so we need to see
                                // if that difference is because of a conflict.
                                if base_content_weight
                                    .vector_clock_write()
                                    .is_newer_than(to_merge_content_weight.vector_clock_write())
                                    || to_merge_content_weight
                                        .vector_clock_write()
                                        .is_newer_than(base_content_weight.vector_clock_write())
                                {
                                    // One side already contains all of the changes from the other, so it's not a conflict.
                                    return Ok(petgraph::visit::Control::Continue);
                                } else {
                                    // Both sides have changes the other does not know about: We've
                                    // got a conflict.
                                    conflicts.push(Conflict::NodeContent(ConflictLocation {
                                        base: base_node_index,
                                        other: to_merge_node_index,
                                    }));
                                }
                            }
                        }

                        Ok(petgraph::visit::Control::Continue)
                    }
                    // TODO: Remove this. Only here to get type checking/guessing to work.
                    // Not actually what we want, since this is about finishing all edges from a node,
                    // not about finishing the graph.
                    petgraph::visit::DfsEvent::Finish(_, _) => {
                        Ok(petgraph::visit::Control::Break(()))
                    }
                    _ => Ok(petgraph::visit::Control::Continue),
                    // petgraph::visit::DfsEvent::TreeEdge(_, _) => todo!(),
                    // petgraph::visit::DfsEvent::BackEdge(_, _) => todo!(),
                    // petgraph::visit::DfsEvent::CrossForwardEdge(_, _) => todo!(),
                    // petgraph::visit::DfsEvent::Finish(_, _) => todo!(),
                }
            });
        if let Err(traversal_error) = result {
            return Err(WorkspaceSnapshotGraphError::GraphTraversal(traversal_error));
        }

        Ok(conflicts)
    }

    fn has_container_membership_conflict(
        &self,
        base_container_node_index: NodeIndex,
        to_merge: &WorkspaceSnapshotGraph,
        to_merge_container_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<Conflict>> {
        let base_ordering_node_indexes =
            ordering_node_indexes_for_node_index(self, base_container_node_index);
        if base_ordering_node_indexes.len() > 1 {
            return Err(WorkspaceSnapshotGraphError::TooManyOrderingForNode(
                base_container_node_index,
            ));
        }
        let to_merge_ordering_node_indexes =
            ordering_node_indexes_for_node_index(to_merge, to_merge_container_node_index);
        if to_merge_ordering_node_indexes.len() > 1 {
            return Err(WorkspaceSnapshotGraphError::TooManyOrderingForNode(
                base_container_node_index,
            ));
        }

        let (base_order_index, to_merge_order_index) = match (
            base_ordering_node_indexes.get(0),
            to_merge_ordering_node_indexes.get(0),
        ) {
            (Some(base_order_index), Some(to_merge_order_index)) => {
                (*base_order_index, *to_merge_order_index)
            }
            (Some(_), None) | (None, Some(_)) => {
                return Err(
                    WorkspaceSnapshotGraphError::CannotCompareOrderedAndUnorderedContainers(
                        base_container_node_index,
                        to_merge_container_node_index,
                    ),
                );
            }
            (None, None) => {
                // Neither is ordered. The potential conflict could be because one
                // or more elements changed, because elements were added/removed,
                // or a combination of these.
                //
                // We need to check for all of these using the outgoing edges from
                // the containers, since we can't rely on an ordering child to
                // contain all the information to determine ordering/addition/removal.
                //
                // TODO: Eventually, this shouldn't ever happen, since Objects, Maps, and Arrays should all have an ordering, for at least display ordering purposes.
                warn!(
                    "Found what appears to be two unordered containers: {:?}, {:?}",
                    base_container_node_index, to_merge_container_node_index
                );

                todo!();
            }
        };

        let base_order = match self.get_node_weight(base_order_index)? {
            NodeWeight::Content(_) => unreachable!(),
            NodeWeight::Ordering(o) => o,
        };
        let to_merge_order = match to_merge.get_node_weight(to_merge_order_index)? {
            NodeWeight::Content(_) => unreachable!(),
            NodeWeight::Ordering(o) => o,
        };

        if base_order.order() == to_merge_order.order() {
            // Set membership same on both sides & order the same: No child conflict
            return Ok(None);
        }

        let base_order_set: HashSet<Ulid> = base_order.order().iter().copied().collect();
        let to_merge_order_set: HashSet<Ulid> = to_merge_order.order().iter().copied().collect();
        if base_order_set == to_merge_order_set {
            // Set membership same on both sides & only one side changed ordering: No child conflict
            if base_order
                .vector_clock_write()
                .is_newer_than(to_merge_order.vector_clock_write())
                || to_merge_order
                    .vector_clock_write()
                    .is_newer_than(base_order.vector_clock_write())
            {
                return Ok(None);
            }

            // Set membership same on both sides & both sides changed ordering: Conflict::ChildOrder
            return Ok(Some(Conflict::ChildOrder(ConflictLocation {
                base: base_order_index,
                other: to_merge_order_index,
            })));
        } else if base_order_set
            .difference(&to_merge_order_set)
            .next()
            .is_some()
            && to_merge_order_set
                .difference(&base_order_set)
                .next()
                .is_some()
        {
            // Set membership different between sides & each side has entries the other does not: Conflict::ChildMembership
            return Ok(Some(Conflict::ChildMembership(ConflictLocation {
                base: base_container_node_index,
                other: to_merge_container_node_index,
            })));
        }

        // Set membership different between sides & only one side has entries the other does not, there
        // can still be a conflict if one side has also changed ordering (both sides will have written
        // to the order for different reasons).
        if !base_order
            .vector_clock_write()
            .is_newer_than(to_merge_order.vector_clock_write())
            && !to_merge_order
                .vector_clock_write()
                .is_newer_than(base_order.vector_clock_write())
        {
            // By comparing the ordering using only the elements from the intersection of the two sets
            // we can help narrow down whether the conflict is an ordering conflict, or a membership
            // conflict. If the ordering of the intersection is the same between both, then it's a membership
            // conflict.
            let common_element_ids: HashSet<Ulid> = base_order_set
                .intersection(&to_merge_order_set)
                .copied()
                .collect();
            let mut base_common_order = base_order.order().clone();
            base_common_order.retain(|id| common_element_ids.contains(id));
            let mut to_merge_common_order = to_merge_order.order().clone();
            to_merge_common_order.retain(|id| common_element_ids.contains(id));
            if base_common_order == to_merge_common_order {
                return Ok(Some(Conflict::ChildMembership(ConflictLocation {
                    base: base_container_node_index,
                    other: to_merge_container_node_index,
                })));
            }

            // TODO: It's still possible that this is an ordering conflict, but we're not checking at that level of detail yet.
            //
            // We can probably tell whether it's a membership, or an ordering conflict by comparing the
            // ordering using only the intersection of the two sets.
            return Ok(Some(Conflict::ChildMembership(ConflictLocation {
                base: base_container_node_index,
                other: to_merge_container_node_index,
            })));
        }

        Ok(None)
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
            if edge_reference.weight().kind == EdgeWeightKind::Ordering {
                if let Some((_, destination_node_index)) =
                    snapshot.graph.edge_endpoints(edge_reference.id())
                {
                    if matches!(
                        snapshot.get_node_weight(destination_node_index),
                        Ok(NodeWeight::Ordering(_))
                    ) {
                        return Some(destination_node_index);
                    }
                }
            }

            None
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ComponentId, ContentHash, FuncId, PropId, SchemaId, SchemaVariantId};

    #[test]
    fn new() {
        let change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        assert!(graph.is_acyclic_directed());
    }

    #[test]
    fn add_nodes_and_edges() {
        let change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let schema_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        let component_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let component_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::new(
                        ComponentId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");

        graph
            .add_edge(
                change_set,
                graph.root_index,
                EdgeWeight::default(),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                change_set,
                graph.root_index,
                EdgeWeight::default(),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::default(),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::default(),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let func_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let func_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    func_id,
                    ContentAddress::Func(ContentHash::new(
                        FuncId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add func");
        let prop_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let prop_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    prop_id,
                    ContentAddress::Prop(ContentHash::new(
                        PropId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add prop");

        graph
            .add_edge(
                change_set,
                graph.root_index,
                EdgeWeight::default(),
                func_index,
            )
            .expect("Unable to add root -> func edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::default(),
                prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::default(),
                graph
                    .get_node_index_by_id(func_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add prop -> func edge");

        assert!(graph.is_acyclic_directed());
    }

    #[test]
    fn cyclic_failure() {
        let change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let initial_schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let initial_schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        let component_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let initial_component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::new(
                        ComponentId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");

        graph
            .add_edge(
                change_set,
                graph.root_index,
                EdgeWeight::default(),
                initial_component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                change_set,
                graph.root_index,
                EdgeWeight::default(),
                initial_schema_node_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Cannot find NodeIndex"),
                EdgeWeight::default(),
                initial_schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Cannot find NodeIndex"),
                EdgeWeight::default(),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot find NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let pre_cycle_root_index = graph.root_index;

        // This should cause a cycle.
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot find NodeIndex"),
                EdgeWeight::default(),
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Cannot find NodeIndex"),
            )
            .expect_err("Created a cycle");

        assert_eq!(pre_cycle_root_index, graph.root_index,);
    }

    #[test]
    fn update_content() {
        let change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let schema_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        let component_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let component_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::new(
                        component_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");

        graph
            .add_edge(
                change_set,
                graph.root_index,
                EdgeWeight::default(),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                change_set,
                graph.root_index,
                EdgeWeight::default(),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::default(),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::default(),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        graph.dot();

        // TODO: This is meant to simulate "modifying" the existing component, instead of swapping in a completely independent component.
        graph
            .update_content(
                change_set,
                component_id.into(),
                ContentHash::new("new_content".as_bytes()),
            )
            .expect("Unable to update Component content hash");

        graph.dot();

        graph.cleanup();

        graph.dot();

        panic!();

        // TODO(nick,jacob): do something here
    }

    #[test]
    fn update_content_from_new_change_set() {
        let change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let schema_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        let component_id = change_set.generate_ulid().expect("Cannot generate Ulid");
        let component_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::new(
                        component_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");

        graph
            .add_edge(
                change_set,
                graph.root_index,
                EdgeWeight::default(),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                change_set,
                graph.root_index,
                EdgeWeight::default(),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::default(),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::default(),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        graph.dot();

        let update_change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        graph
            .update_content(
                &update_change_set,
                component_id.into(),
                ContentHash::new("new_content".as_bytes()),
            )
            .expect("Unable to update Component content hash");

        graph.dot();

        graph.cleanup();

        graph.dot();

        panic!();

        // TODO(nick,jacob): do something here
    }

    #[test]
    fn compare_snapshots_purely_new_content() {
        let initial_change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let initial_change_set = &initial_change_set;
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let schema_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::new("Schema A".as_bytes())),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = initial_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let schema_variant_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::new("Schema Variant A".as_bytes())),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        initial_graph
            .add_edge(
                initial_change_set,
                initial_graph.root_index,
                EdgeWeight::default(),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::default(),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        initial_graph.dot();

        let new_change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = initial_graph.clone();

        let component_id = new_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let component_index = new_graph
            .add_node(
                NodeWeight::new_content(
                    new_change_set,
                    component_id,
                    ContentAddress::Schema(ContentHash::new("Component A".as_bytes())),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        new_graph
            .add_edge(
                new_change_set,
                new_graph.root_index,
                EdgeWeight::default(),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        new_graph
            .add_edge(
                new_change_set,
                new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::default(),
                new_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        new_graph.dot();

        panic!();
    }
}
