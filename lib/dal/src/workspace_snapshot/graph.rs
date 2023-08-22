use petgraph::visit::NodeCount;
use petgraph::{algo, prelude::*, visit::DfsEvent};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::{
    workspace_snapshot::{
        change_set::{ChangeSet, ChangeSetError},
        conflict::Conflict,
        edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind},
        node_weight::{ContentAddress, NodeWeight, NodeWeightError},
        update::Update,
    },
    ContentHash,
};

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum WorkspaceSnapshotGraphError {
    #[error("Cannot compare ordering of container elements between ordered, and un-ordered container: {0:?}, {1:?}")]
    CannotCompareOrderedAndUnorderedContainers(NodeIndex, NodeIndex),
    #[error("ChangeSet error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("Action would create a graph cycle")]
    CreateGraphCycle,
    #[error("EdgeWeight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("EdgeWeight not found")]
    EdgeWeightNotFound,
    #[error("Problem during graph traversal: {0:?}")]
    GraphTraversal(petgraph::visit::DfsEvent<NodeIndex>),
    #[error("Incompatible node types")]
    IncompatibleNodeTypes,
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("node weight not found")]
    NodeWeightNotFound,
    #[error("Node with ID {0} not found")]
    NodeWithIdNotFound(Ulid),
    #[error("NodeIndex has too many Ordering children: {0:?}")]
    TooManyOrderingForNode(NodeIndex),
    #[error("Workspace Snapshot has conflicts and must be rebased")]
    WorkspaceNeedsRebase,
    #[error("Workspace Snapshot has conflicts")]
    WorkspacesConflict,
}

pub type WorkspaceSnapshotGraphResult<T> = Result<T, WorkspaceSnapshotGraphError>;

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct WorkspaceSnapshotGraph {
    graph: StableDiGraph<NodeWeight, EdgeWeight>,
    root_index: NodeIndex,
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
        let root_index = graph.add_node(NodeWeight::new_content(
            change_set,
            change_set.generate_ulid()?,
            ContentAddress::Root,
        )?);

        Ok(Self { root_index, graph })
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn add_edge(
        &mut self,
        change_set: &ChangeSet,
        from_node_index: NodeIndex,
        mut edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<EdgeIndex> {
        // Temporarily add the edge to the existing tree to see if it would create a cycle.
        let temp_edge = self
            .graph
            .update_edge(from_node_index, to_node_index, edge_weight.clone());
        let would_create_a_cycle = !self.is_acyclic_directed();
        self.graph.remove_edge(temp_edge);
        if would_create_a_cycle {
            return Err(WorkspaceSnapshotGraphError::CreateGraphCycle);
        }

        // Ensure the vector clocks of the edge are up-to-date.
        edge_weight.increment_vector_clocks(change_set)?;

        // Because outgoing edges are part of a node's identity, we create a new "from" node
        // as we are effectively writing to that node (we'll need to update the merkle tree
        // hash), and everything in the graph should be treated as copy-on-write.
        let new_from_node_index = self.copy_node_index(from_node_index)?;

        // Add the new edge to the new version of the "from" node.
        let new_edge_index =
            self.graph
                .update_edge(new_from_node_index, to_node_index, edge_weight);
        self.update_merkle_tree_hash(new_from_node_index)?;

        // Update the rest of the graph to reflect the new node/edge.
        self.replace_references(change_set, from_node_index, new_from_node_index)?;

        Ok(new_edge_index)
    }

    fn add_node(&mut self, node: NodeWeight) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let new_node_index = self.graph.add_node(node);
        self.update_merkle_tree_hash(new_node_index)?;

        Ok(new_node_index)
    }

    pub fn cleanup(&mut self) {
        self.graph.retain_nodes(|frozen_graph, current_node| {
            // We cannot use "has_path_to_root" because we need to use the Frozen<StableGraph<...>>.
            algo::has_path_connecting(&*frozen_graph, self.root_index, current_node, None)
        });
    }

    fn copy_node_index(
        &mut self,
        node_index_to_copy: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let new_node_index = self
            .graph
            .add_node(self.get_node_weight(node_index_to_copy)?.clone());
        Ok(new_node_index)
    }

    fn detect_conflicts_and_updates(
        &self,
        to_rebase_change_set: &ChangeSet,
        onto: &WorkspaceSnapshotGraph,
        onto_change_set: &ChangeSet,
    ) -> WorkspaceSnapshotGraphResult<(Vec<Conflict>, Vec<Update>)> {
        let mut conflicts: Vec<Conflict> = Vec::new();
        let mut updates: Vec<Update> = Vec::new();
        if let Err(traversal_error) =
            petgraph::visit::depth_first_search(&onto.graph, Some(onto.root_index), |event| {
                self.detect_conflicts_and_updates_process_dfs_event(
                    to_rebase_change_set,
                    onto,
                    onto_change_set,
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
        to_rebase_change_set: &ChangeSet,
        onto: &WorkspaceSnapshotGraph,
        onto_change_set: &ChangeSet,
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
                let mut to_rebase_node_indexes = Vec::new();
                if let NodeWeight::Content(onto_content_weight) = onto_node_weight {
                    if onto_content_weight.content_address() == ContentAddress::Root {
                        // There can only be one (valid/current) `ContentAddress::Root` at any
                        // given moment, and the `lineage_id` isn't really relevant as it's not
                        // globally stable (even though it is locally stable). This matters as we
                        // may be dealing with a `WorkspaceSnapshotGraph` that is coming to us
                        // externally from a module that we're attempting to import. The external
                        // `WorkspaceSnapshotGraph` will be `self`, and the "local" one will be
                        // `onto`.
                        to_rebase_node_indexes.push(self.root_index);
                    } else {
                        // Only retain node indexes... or indices... if they are part of the current
                        // graph. There may still be garbage from previous updates to the graph
                        // laying around.
                        let mut potential_to_rebase_node_indexes = self
                            .get_node_index_by_lineage(onto_node_weight.lineage_id())
                            .map_err(|err| {
                                error!(
                                    "Unable to find NodeIndex(es) for lineage_id {}: {}",
                                    onto_node_weight.lineage_id(),
                                    err,
                                );
                                event
                            })?;
                        potential_to_rebase_node_indexes
                            .retain(|node_index| self.has_path_to_root(*node_index));
                        to_rebase_node_indexes.extend(potential_to_rebase_node_indexes);
                    }
                }

                // We'll lazily populate these, since we don't know if we'll need it at all, and
                // we definitely don't want to be re-fetching this information inside the loop
                // below, as it will be identical every time.
                let mut onto_edges = None;
                let mut onto_ordering_node_index = None;
                let mut onto_order_set: Option<HashSet<Ulid>> = None;

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
                                new: onto_node_index,
                                old: to_rebase_node_index,
                            });
                        } else {
                            // There are changes on both sides that have not been seen by the other
                            // side; this is a conflict. There may also be other conflicts in the
                            // outgoing relationships, the downstream nodes, or both.
                            conflicts.push(Conflict::NodeContent {
                                to_rebase: to_rebase_node_index,
                                onto: onto_node_index,
                            });
                        }
                    }

                    if onto_ordering_node_index.is_none() {
                        let onto_ordering_node_indexes =
                            ordering_node_indexes_for_node_index(onto, onto_node_index);
                        if onto_ordering_node_indexes.len() > 1 {
                            error!(
                                "Too many ordering nodes found for onto NodeIndex {:?}",
                                onto_node_index
                            );
                            return Err(event);
                        }
                        onto_ordering_node_index = onto_ordering_node_indexes.get(0).copied();
                    }
                    let to_rebase_ordering_node_indexes =
                        ordering_node_indexes_for_node_index(self, to_rebase_node_index);
                    if to_rebase_ordering_node_indexes.len() > 1 {
                        error!(
                            "Too many ordering nodes found for to_rebase NodeIndex {:?}",
                            to_rebase_node_index
                        );
                        return Err(event);
                    }
                    let to_rebase_ordering_node_index =
                        to_rebase_ordering_node_indexes.get(0).copied();

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
                            warn!(
                                "Found what appears to be two unordered containers: onto {:?}, to_rebase {:?}",
                                onto_node_index, to_rebase_node_index,
                            );
                            println!(
                                "Comparing unordered containers: {:?}, {:?}",
                                onto_node_index, to_rebase_node_index
                            );

                            let onto_edges = onto_edges.get_or_insert_with(|| {
                                onto.graph.edges_directed(onto_node_index, Outgoing)
                            });
                            let to_rebase_edges =
                                self.graph.edges_directed(to_rebase_node_index, Outgoing);

                            let (container_conflicts, container_updates) = self
                                .find_unordered_container_membership_conflicts_and_updates(
                                    to_rebase_change_set,
                                    to_rebase_node_index,
                                    onto,
                                    onto_change_set,
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
                            println!(
                                "Comparing ordered containers: {:?}, {:?}",
                                onto_node_index, to_rebase_node_index
                            );
                            if onto_order_set.is_none() {
                                if let NodeWeight::Ordering(onto_order_weight) = onto
                                    .get_node_weight(onto_ordering_node_index)
                                    .map_err(|_| event)?
                                {
                                    onto_order_set =
                                        Some(onto_order_weight.order().iter().copied().collect());
                                };
                            }
                            let (container_conflicts, container_updates) = self
                                .find_ordered_container_membership_conflicts_and_updates(
                                    to_rebase_change_set,
                                    to_rebase_node_index,
                                    to_rebase_ordering_node_index,
                                    onto,
                                    onto_change_set,
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
                    return Ok(petgraph::visit::Control::Continue);
                } else {
                    // Everything to be rebased is identical, so there's no need to examine the
                    // rest of the tree looking for differences & conflicts that won't be there.
                    return Ok(petgraph::visit::Control::Prune);
                }
            }
            DfsEvent::TreeEdge(_, _)
            | DfsEvent::BackEdge(_, _)
            | DfsEvent::CrossForwardEdge(_, _)
            | DfsEvent::Finish(_, _) => {
                // These events are all ignored, since we handle looking at edges as we encounter
                // the node(s) the edges are coming from (Outgoing edges).
                return Ok(petgraph::visit::Control::Continue);
            }
        }
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

    pub fn update_content(
        &mut self,
        change_set: &ChangeSet,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let original_node_index = self.get_node_index_by_id(id)?;
        let new_node_index = self.copy_node_index(original_node_index)?;
        let node_weight = self.get_node_weight_mut(new_node_index)?;
        node_weight.increment_vector_clock(change_set)?;
        node_weight.new_content_hash(new_content_hash)?;

        self.replace_references(change_set, original_node_index, new_node_index)
    }

    fn find_ordered_container_membership_conflicts_and_updates(
        &self,
        to_rebase_change_set: &ChangeSet,
        to_rebase_container_index: NodeIndex,
        to_rebase_ordering_index: NodeIndex,
        onto: &WorkspaceSnapshotGraph,
        onto_change_set: &ChangeSet,
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
                    for edge in self
                        .graph
                        .edges_connecting(to_rebase_container_index, to_rebase_container_item_index)
                    {
                        updates.push(Update::RemoveEdge(edge.id()));
                    }
                }
            }

            // Use the ordering from `other` in `to_rebase`.
            updates.push(Update::ReplaceSubgraph {
                new: onto_ordering_index,
                old: to_rebase_ordering_index,
            });
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
                        .entry_for(onto_change_set)
                        .is_none()
                    {
                        // `only_to_rebase_item` is new: Edge in `to_rebase` does not have a "First Seen" for `onto`.
                    } else if self
                        .get_node_weight(only_to_rebase_item_index)?
                        .vector_clock_write()
                        .entry_for(to_rebase_change_set)
                        .is_some()
                    {
                        // Entry was deleted in `onto`. If we have also modified the entry, then
                        // there's a conflict.
                        conflicts.push(Conflict::ModifyRemovedItem(only_to_rebase_item_index));
                    } else {
                        // Entry was deleted in `onto`, and has not been modified in `to_rebase`:
                        // Remove the edge.
                        updates.push(Update::RemoveEdge(to_rebase_edgeref.id()));
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
                .entry_for(onto_change_set);
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
                        .entry_for(onto_change_set)
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
                    } else if let Some(onto_item_node_weight) =
                        onto.get_node_weight(only_onto_item_index).ok()
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
        to_rebase_change_set: &ChangeSet,
        to_rebase_container_index: NodeIndex,
        onto: &WorkspaceSnapshotGraph,
        onto_change_set: &ChangeSet,
        onto_container_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<(Vec<Conflict>, Vec<Update>)> {
        #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
        struct UniqueEdgeInfo {
            pub kind: EdgeWeightKind,
            pub target_lineage: Ulid,
        }

        #[derive(Debug, Copy, Clone)]
        struct EdgeInfo {
            pub target_node_index: NodeIndex,
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
                    kind: edgeref.weight().kind(),
                    target_lineage: target_node_weight.lineage_id(),
                },
                EdgeInfo {
                    target_node_index: edgeref.target(),
                    edge_index: edgeref.id(),
                },
            );
        }

        let mut onto_edges = HashMap::<UniqueEdgeInfo, EdgeInfo>::new();
        for edgeref in onto.graph.edges_directed(onto_container_index, Outgoing) {
            let target_node_weight = onto.get_node_weight(edgeref.target())?;
            onto_edges.insert(
                UniqueEdgeInfo {
                    kind: edgeref.weight().kind(),
                    target_lineage: target_node_weight.lineage_id(),
                },
                EdgeInfo {
                    target_node_index: edgeref.target(),
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

        let root_seen_as_of_onto = self
            .get_node_weight(self.root_index)?
            .vector_clock_recently_seen()
            .entry_for(onto_change_set);
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
                .entry_for(onto_change_set)
                .is_some()
            {
                if to_rebase_item_weight
                    .vector_clock_write()
                    .entry_for(to_rebase_change_set)
                    > root_seen_as_of_onto
                {
                    // Edge has been modified in `onto` (`onto` item write vector clock > "seen as
                    // of" for `onto` entry in `to_rebase` root): Conflict (ModifyRemovedItem)
                    conflicts.push(Conflict::ModifyRemovedItem(
                        only_to_rebase_edge_info.target_node_index,
                    ))
                } else {
                    // Item not modified & removed by `onto`: No conflict; Update::RemoveEdge
                    updates.push(Update::RemoveEdge(only_to_rebase_edge_info.edge_index));
                }
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
                .entry_for(onto_change_set)
            {
                if let Some(root_seen_as_of) = root_seen_as_of_onto {
                    if onto_first_seen > root_seen_as_of {
                        // Edge first seen by `onto` > "seen as of" on `to_rebase` graph for `onto`'s entry on
                        // root node: Item is new.
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

    fn get_node_index_by_id(&self, id: Ulid) -> WorkspaceSnapshotGraphResult<NodeIndex> {
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

        Err(WorkspaceSnapshotGraphError::NodeWithIdNotFound(id))
    }

    fn get_node_index_by_lineage(
        &self,
        lineage_id: Ulid,
    ) -> WorkspaceSnapshotGraphResult<Vec<NodeIndex>> {
        let mut results = Vec::new();
        for node_index in self.graph.node_indices() {
            if let NodeWeight::Content(node_weight) = self.get_node_weight(node_index)? {
                if node_weight.lineage_id() == lineage_id {
                    results.push(node_index);
                }
            }
        }

        Ok(results)
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

    fn has_path_to_root(&self, node: NodeIndex) -> bool {
        algo::has_path_connecting(&self.graph, self.root_index, node, None)
    }

    fn import_subgraph(
        &mut self,
        other: &WorkspaceSnapshotGraph,
        root_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let mut new_node_indexes = HashMap::new();
        let mut dfs = petgraph::visit::DfsPostOrder::new(&other.graph, root_index);
        while let Some(node_index_to_copy) = dfs.next(&other.graph) {
            let node_weight_copy = other.get_node_weight(node_index_to_copy)?.clone();
            let new_node_index = self.add_node(node_weight_copy)?;
            new_node_indexes.insert(node_index_to_copy, new_node_index);

            for edge in other.graph.edges_directed(node_index_to_copy, Outgoing) {
                self.graph.update_edge(
                    new_node_index,
                    new_node_indexes
                        .get(&edge.target())
                        .copied()
                        .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?,
                    edge.weight().clone(),
                );
            }
        }

        new_node_indexes
            .get(&root_index)
            .copied()
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)
    }

    fn is_acyclic_directed(&self) -> bool {
        // Using this because "is_cyclic_directed" is recursive.
        algo::toposort(&self.graph, None).is_ok()
    }

    fn is_on_path_between(&self, start: NodeIndex, end: NodeIndex, node: NodeIndex) -> bool {
        algo::has_path_connecting(&self.graph, start, node, None)
            && algo::has_path_connecting(&self.graph, node, end, None)
    }

    /// [`StableGraph`] guarantees the stability of [`NodeIndex`] across removals, however there
    /// are **NO** guarantees around the stability of [`EdgeIndex`] across removals. If
    /// [`Self::cleanup()`] has been called, then any [`EdgeIndex`] should be considered invalid.
    fn remove_edge(
        &mut self,
        change_set: &ChangeSet,
        source_node_index: NodeIndex,
        target_node_index: NodeIndex,
        edge_kind: EdgeWeightKind,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let mut edges_to_remove = Vec::new();
        let new_source_node_index = dbg!(self.copy_node_index(source_node_index)?);
        self.replace_references(change_set, dbg!(source_node_index), new_source_node_index)?;

        for edgeref in self
            .graph
            .edges_connecting(new_source_node_index, target_node_index)
        {
            dbg!(&edgeref);
            if edgeref.weight().kind() == edge_kind {
                edges_to_remove.push(edgeref.id());
            }
        }
        for edge_to_remove in edges_to_remove {
            self.graph.remove_edge(edge_to_remove);
        }

        self.update_merkle_tree_hash(new_source_node_index)?;

        Ok(())
    }

    fn replace_references(
        &mut self,
        change_set: &ChangeSet,
        original_node_index: NodeIndex,
        new_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
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
                        let new_node_index = self.copy_node_index(old_node_index)?;
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
                        edges_to_create.push((edge_weight.clone(), destination_node_index));
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
}

fn ordering_node_indexes_for_node_index(
    snapshot: &WorkspaceSnapshotGraph,
    node_index: NodeIndex,
) -> Vec<NodeIndex> {
    snapshot
        .graph
        .edges_directed(node_index, Outgoing)
        .filter_map(|edge_reference| {
            if edge_reference.weight().kind() == EdgeWeightKind::Ordering {
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
    use pretty_assertions_sorted::assert_eq;

    #[derive(Debug, PartialEq)]
    struct ConflictsAndUpdates {
        conflicts: Vec<Conflict>,
        updates: Vec<Update>,
    }

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
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                change_set,
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
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
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                func_index,
            )
            .expect("Unable to add root -> func edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
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
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                initial_component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                change_set,
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                initial_schema_node_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Cannot find NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                initial_schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Cannot find NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
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
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
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
                    ContentAddress::Schema(ContentHash::from("Constellation")),
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
                        "Freestar Collective".as_bytes(),
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
                    ContentAddress::Component(ContentHash::from("Crimson Fleet")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");

        graph
            .add_edge(
                change_set,
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                change_set,
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                change_set,
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        graph.dot();

        // Ensure that the root node merkle tree hash looks as we expect before the update.
        let pre_update_root_node_merkle_tree_hash: ContentHash =
            serde_json::from_value(serde_json::json![
                "66e2b07b8a9a5f94a0ea18618a57b3264c850ea6cfeb81f5c9a42c4397f2f49d"
            ])
            .expect("could not deserialize");
        assert_eq!(
            pre_update_root_node_merkle_tree_hash, // expected
            graph
                .get_node_weight(graph.root_index)
                .expect("could not get node weight")
                .merkle_tree_hash(), // actual
        );

        let updated_content_hash = ContentHash::from("new_content");
        graph
            .update_content(change_set, component_id.into(), updated_content_hash)
            .expect("Unable to update Component content hash");

        graph.dot();

        let post_update_root_node_merkle_tree_hash: ContentHash =
            serde_json::from_value(serde_json::json![
                "0b9b79be9c1b4107bd32dc9fb7accde544dc10171e37847e53c4d16a9efd2da1"
            ])
            .expect("could not deserialize");
        assert_eq!(
            post_update_root_node_merkle_tree_hash, // expected
            graph
                .get_node_weight(graph.root_index)
                .expect("could not get node weight")
                .merkle_tree_hash(), // actual
        );
        assert_eq!(
            updated_content_hash, // expected
            graph
                .get_node_weight(
                    graph
                        .get_node_index_by_id(component_id)
                        .expect("could not get node index by id")
                )
                .expect("could not get node weight")
                .content_hash(), // actual
        );

        graph.cleanup();

        graph.dot();

        // Ensure that there are not more nodes than the ones that should be in use.
        assert_eq!(4, graph.node_count());

        // The hashes must not change upon cleanup.
        assert_eq!(
            post_update_root_node_merkle_tree_hash, // expected
            graph
                .get_node_weight(graph.root_index)
                .expect("could not get node weight")
                .merkle_tree_hash(), // actual
        );
        assert_eq!(
            updated_content_hash, // expected
            graph
                .get_node_weight(
                    graph
                        .get_node_index_by_id(component_id)
                        .expect("could not get node index by id")
                )
                .expect("could not get node weight")
                .content_hash(), // actual
        );
    }

    #[test]
    fn detect_conflicts_and_updates_simple_no_conflicts_no_updates_in_base() {
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
                    ContentAddress::Schema(ContentHash::from("Schema A")),
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
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        initial_graph
            .add_edge(
                initial_change_set,
                initial_graph.root_index,
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
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
                    ContentAddress::Schema(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        new_graph
            .add_edge(
                new_change_set,
                new_graph.root_index,
                EdgeWeight::new(new_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        new_graph
            .add_edge(
                new_change_set,
                new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(new_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                new_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        new_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(new_change_set, &initial_graph, initial_change_set)
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);
        assert_eq!(Vec::<Update>::new(), updates);
    }

    #[test]
    fn detect_conflicts_and_updates_simple_no_conflicts_with_purely_new_content_in_base() {
        let initial_change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        base_graph
            .add_edge(
                base_change_set,
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_change_set,
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        let new_change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();

        let new_onto_component_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let new_onto_component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    new_onto_component_id,
                    ContentAddress::Component(ContentHash::from("Component B")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component B");
        let new_onto_root_component_edge_index = base_graph
            .add_edge(
                base_change_set,
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                new_onto_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_change_set,
                base_graph
                    .get_node_index_by_id(new_onto_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        println!("Updated base graph (Root: {:?}):", base_graph.root_index);
        base_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(new_change_set, &base_graph, base_change_set)
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);

        let new_onto_component_index = base_graph
            .get_node_index_by_id(new_onto_component_id)
            .expect("Unable to get NodeIndex");
        match updates.as_slice() {
            [Update::NewEdge {
                source,
                destination,
                edge_weight,
            }] => {
                assert_eq!(new_graph.root_index, *source);
                assert_eq!(new_onto_component_index, *destination);
                assert_eq!(EdgeWeightKind::Uses, edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_no_conflicts_with_updates_on_both_sides() {
        let initial_change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        base_graph
            .add_edge(
                base_change_set,
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_change_set,
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        let new_change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();

        let component_id = new_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let component_index = new_graph
            .add_node(
                NodeWeight::new_content(
                    new_change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        new_graph
            .add_edge(
                new_change_set,
                new_graph.root_index,
                EdgeWeight::new(new_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        new_graph
            .add_edge(
                new_change_set,
                new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(new_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                new_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        println!("new graph (Root {:?}):", new_graph.root_index);
        new_graph.dot();

        let new_onto_component_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let new_onto_component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    new_onto_component_id,
                    ContentAddress::Component(ContentHash::from("Component B")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component B");
        base_graph
            .add_edge(
                base_change_set,
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                new_onto_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_change_set,
                base_graph
                    .get_node_index_by_id(new_onto_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        println!("Updated base graph (Root: {:?}):", base_graph.root_index);
        base_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(new_change_set, &base_graph, base_change_set)
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);

        let new_onto_component_index = base_graph
            .get_node_index_by_id(new_onto_component_id)
            .expect("Unable to get NodeIndex");
        match updates.as_slice() {
            [Update::NewEdge {
                source,
                destination,
                edge_weight,
            }] => {
                assert_eq!(new_graph.root_index, *source);
                assert_eq!(new_onto_component_index, *destination);
                assert_eq!(EdgeWeightKind::Uses, edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_with_content_conflict() {
        let initial_change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        base_graph
            .add_edge(
                base_change_set,
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_change_set,
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let component_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        base_graph
            .add_edge(
                base_change_set,
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_change_set,
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.cleanup();
        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        let new_change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();

        new_graph
            .update_content(
                new_change_set,
                component_id,
                ContentHash::from("Updated Component A"),
            )
            .expect("Unable to update Component A");

        new_graph.cleanup();
        println!("new graph (Root {:?}):", new_graph.root_index);
        new_graph.dot();

        base_graph
            .update_content(
                base_change_set,
                component_id,
                ContentHash::from("Base Updated Component A"),
            )
            .expect("Unable to update Component A");

        base_graph.cleanup();
        println!("Updated base graph (Root: {:?}):", base_graph.root_index);
        base_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(new_change_set, &base_graph, base_change_set)
            .expect("Unable to detect conflicts and updates");

        assert_eq!(
            vec![Conflict::NodeContent {
                onto: base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get component NodeIndex"),
                to_rebase: new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get component NodeIndex"),
            }],
            conflicts
        );
        assert_eq!(Vec::<Update>::new(), updates);
    }

    #[test]
    fn detect_conflicts_and_updates_simple_with_modify_removed_item_conflict() {
        let initial_change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        base_graph
            .add_edge(
                base_change_set,
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_change_set,
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let component_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        base_graph
            .add_edge(
                base_change_set,
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_change_set,
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.cleanup();
        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        let new_change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();

        base_graph
            .remove_edge(
                base_change_set,
                base_graph.root_index,
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeightKind::Uses,
            )
            .expect("Unable to remove Component A");

        base_graph.cleanup();
        println!("Updated base graph (Root: {:?}):", base_graph.root_index);
        base_graph.dot();

        new_graph
            .update_content(
                new_change_set,
                component_id,
                ContentHash::from("Updated Component A"),
            )
            .expect("Unable to update Component A");

        new_graph.cleanup();
        println!("new graph (Root {:?}):", new_graph.root_index);
        new_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(new_change_set, &base_graph, base_change_set)
            .expect("Unable to detect conflicts and updates");

        assert_eq!(
            vec![Conflict::ModifyRemovedItem(
                new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex")
            )],
            conflicts
        );
        assert_eq!(Vec::<Update>::new(), updates);
    }

    #[test]
    fn detect_conflicts_and_updates_complex() {
        let initial_change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        // Docker Image Schema
        let docker_image_schema_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let docker_image_schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    docker_image_schema_id,
                    ContentAddress::Schema(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        base_graph
            .add_edge(
                base_change_set,
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                docker_image_schema_index,
            )
            .expect("Unable to add root -> schema edge");

        // Docker Image Schema Variant
        let docker_image_schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let docker_image_schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    docker_image_schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");
        base_graph
            .add_edge(
                base_change_set,
                base_graph
                    .get_node_index_by_id(docker_image_schema_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                docker_image_schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        // Nginx Docker Image Component
        let nginx_docker_image_component_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let nginx_docker_image_component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    nginx_docker_image_component_id,
                    ContentAddress::Component(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        base_graph
            .add_edge(
                base_change_set,
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                nginx_docker_image_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_change_set,
                base_graph
                    .get_node_index_by_id(nginx_docker_image_component_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(docker_image_schema_variant_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        // Alpine Component
        let alpine_component_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let alpine_component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    alpine_component_id,
                    ContentAddress::Component(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        base_graph
            .add_edge(
                base_change_set,
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                alpine_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_change_set,
                base_graph
                    .get_node_index_by_id(alpine_component_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(docker_image_schema_variant_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        // Butane Schema
        let butane_schema_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let butane_schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    butane_schema_id,
                    ContentAddress::Schema(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        base_graph
            .add_edge(
                base_change_set,
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                butane_schema_index,
            )
            .expect("Unable to add root -> schema edge");

        // Butane Schema Variant
        let butane_schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let butane_schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    butane_schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");
        base_graph
            .add_edge(
                base_change_set,
                base_graph
                    .get_node_index_by_id(butane_schema_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                butane_schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        // Nginx Butane Component
        let nginx_butane_component_id = base_change_set
            .generate_ulid()
            .expect("Cannot generate Ulid");
        let nginx_butane_node_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    nginx_butane_component_id,
                    ContentAddress::Component(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");
        base_graph
            .add_edge(
                base_change_set,
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                nginx_butane_node_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_change_set,
                base_graph
                    .get_node_index_by_id(nginx_butane_component_id)
                    .expect("Cannot get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Uses)
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(butane_schema_variant_id)
                    .expect("Cannot get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.cleanup();
        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        // Create a new change set to cause some problems!
        let new_change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();

        // Create a modify removed item conflict.
        base_graph
            .remove_edge(
                base_change_set,
                base_graph.root_index,
                base_graph
                    .get_node_index_by_id(nginx_butane_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeightKind::Uses,
            )
            .expect("Unable to update the component");
        new_graph
            .update_content(
                new_change_set,
                nginx_butane_component_id,
                ContentHash::from("second"),
            )
            .expect("Unable to update the component");

        // Create a node content conflict.
        base_graph
            .update_content(
                base_change_set,
                docker_image_schema_variant_id,
                ContentHash::from("oopsie"),
            )
            .expect("Unable to update the component");
        new_graph
            .update_content(
                new_change_set,
                docker_image_schema_variant_id,
                ContentHash::from("poopsie"),
            )
            .expect("Unable to update the component");

        // Create a pure update.
        base_graph
            .update_content(
                base_change_set,
                docker_image_schema_id,
                ContentHash::from("bg3"),
            )
            .expect("Unable to update the schema");

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(new_change_set, &base_graph, base_change_set)
            .expect("Unable to detect conflicts and updates");

        println!("base graph current root: {:?}", base_graph.root_index);
        base_graph.dot();
        println!("new graph current root: {:?}", new_graph.root_index);
        new_graph.dot();

        let expected_conflicts = vec![
            Conflict::ModifyRemovedItem(
                new_graph
                    .get_node_index_by_id(nginx_butane_component_id)
                    .expect("Unable to get component NodeIndex"),
            ),
            Conflict::NodeContent {
                onto: base_graph
                    .get_node_index_by_id(docker_image_schema_variant_id)
                    .expect("Unable to get component NodeIndex"),
                to_rebase: new_graph
                    .get_node_index_by_id(docker_image_schema_variant_id)
                    .expect("Unable to get component NodeIndex"),
            },
        ];
        let expected_updates = vec![Update::ReplaceSubgraph {
            new: base_graph
                .get_node_index_by_id(docker_image_schema_id)
                .expect("Unable to get NodeIndex"),
            old: new_graph
                .get_node_index_by_id(docker_image_schema_id)
                .expect("Unable to get NodeIndex"),
        }];

        assert_eq!(
            ConflictsAndUpdates {
                conflicts: expected_conflicts,
                updates: expected_updates,
            },
            ConflictsAndUpdates { conflicts, updates },
        );
    }
}
