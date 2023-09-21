use chrono::Utc;
use petgraph::{algo, prelude::*, visit::DfsEvent};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::change_set_pointer::{ChangeSetPointer, ChangeSetPointerError};
use crate::{
    content::{self, store::StoreError},
    workspace_snapshot::{
        conflict::Conflict,
        content_address::ContentAddress,
        edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind},
        node_weight::{NodeWeight, NodeWeightError, OrderingNodeWeight},
        update::Update,
    },
    ContentHash,
};

pub type LineageId = Ulid;

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum WorkspaceSnapshotGraphError {
    #[error("Cannot compare ordering of container elements between ordered, and un-ordered container: {0:?}, {1:?}")]
    CannotCompareOrderedAndUnorderedContainers(NodeIndex, NodeIndex),
    #[error("ChangeSet error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("Unable to retrieve content for ContentHash")]
    ContentMissingForContentHash,
    #[error("Content store error: {0}")]
    ContentStore(#[from] StoreError),
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
    #[error("Unable to add node to the graph")]
    UnableToAddNode,
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
    pub fn new(change_set: &ChangeSetPointer) -> WorkspaceSnapshotGraphResult<Self> {
        let mut graph: StableDiGraph<NodeWeight, EdgeWeight> = StableDiGraph::with_capacity(1, 0);
        let root_index = graph.add_node(NodeWeight::new_content(
            change_set,
            change_set.generate_ulid()?,
            ContentAddress::Root,
        )?);

        Ok(Self { root_index, graph })
    }

    pub fn add_edge(
        &mut self,
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
        self.replace_references(from_node_index, new_from_node_index)?;

        Ok(new_edge_index)
    }

    pub fn add_node(&mut self, node: NodeWeight) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let new_node_index = self.graph.add_node(node);
        self.update_merkle_tree_hash(new_node_index)?;

        Ok(new_node_index)
    }

    pub fn add_ordered_edge(
        &mut self,
        change_set: &ChangeSetPointer,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<EdgeIndex> {
        let new_edge_index = self.add_edge(from_node_index, edge_weight, to_node_index)?;

        let (new_from_node_index, _) = self
            .graph
            .edge_endpoints(new_edge_index)
            .ok_or(WorkspaceSnapshotGraphError::EdgeWeightNotFound)?;

        // Find the ordering node of the "container" if there is one, and add the thing pointed to
        // by the `to_node_index` to the ordering.
        if let Some(container_ordering_node_index) =
            self.ordering_node_index_for_container(new_from_node_index)?
        {
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

    fn add_ordered_node(
        &mut self,
        change_set: &ChangeSetPointer,
        node: NodeWeight,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let node_weight_id = node.id();
        let new_node_index = self.add_node(node)?;
        let ordering_node_index =
            self.add_node(NodeWeight::Ordering(OrderingNodeWeight::new(change_set)?))?;
        self.add_edge(
            new_node_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Ordering)?,
            ordering_node_index,
        )?;

        // We can't use `self.get_node_index_by_id` yet, since the node isn't connected to the rest
        // of the graph yet, and `get_node_index_by_id` checks to make sure there's a path from the
        // root to the node before returning it. There should only be one node with an edge
        // pointing to the ordering node we just created, however, and that should be the "new
        // version" of the node we're adding.
        for neighbor_index in self.graph.neighbors_directed(ordering_node_index, Incoming) {
            if self.get_node_weight(neighbor_index)?.id() == node_weight_id {
                return Ok(neighbor_index);
            }
        }

        Err(WorkspaceSnapshotGraphError::UnableToAddNode)
    }

    pub fn attribute_value_view(
        &self,
        content_store: &content::Store,
        root_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<serde_json::Value> {
        let mut view = serde_json::json![{}];
        let mut nodes_to_add = VecDeque::from([(root_index, "".to_string())]);

        while let Some((current_node_index, write_location)) = nodes_to_add.pop_front() {
            let current_node_weight = self.get_node_weight(current_node_index)?;
            let current_node_content: serde_json::Value = content_store
                .get(&current_node_weight.content_hash())?
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
                    dbg!(child_position_index, child_node_index);

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
                        nodes_to_add.push_back(dbg!((child_node_index, child_write_location)));
                    }
                }
            } else {
                // The child nodes aren't explicitly ordered, so we'll need to come up with one of
                // our own. We'll sort the nodes by their `NodeIndex`, which means that when a
                // write last happened to them (or anywhere further towards the leaves) will
                // determine their sorting in oldest to most recent order.
                let mut child_index_to_position = HashMap::new();
                let mut child_indexes = Vec::new();
                let mut outgoing_edges = self.graph.edges_directed(current_node_index, Outgoing);
                while let Some(edge_ref) = outgoing_edges.next() {
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

    pub fn detect_conflicts_and_updates(
        &self,
        to_rebase_change_set: &ChangeSetPointer,
        onto: &WorkspaceSnapshotGraph,
        onto_change_set: &ChangeSetPointer,
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
        to_rebase_change_set: &ChangeSetPointer,
        onto: &WorkspaceSnapshotGraph,
        onto_change_set: &ChangeSetPointer,
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
                            warn!(
                                "Found what appears to be two unordered containers: onto {:?}, to_rebase {:?}",
                                onto_node_index, to_rebase_node_index,
                            );
                            println!(
                                "Comparing unordered containers: {:?}, {:?}",
                                onto_node_index, to_rebase_node_index
                            );

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

    fn find_ordered_container_membership_conflicts_and_updates(
        &self,
        to_rebase_change_set: &ChangeSetPointer,
        to_rebase_container_index: NodeIndex,
        to_rebase_ordering_index: NodeIndex,
        onto: &WorkspaceSnapshotGraph,
        onto_change_set: &ChangeSetPointer,
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
        to_rebase_change_set: &ChangeSetPointer,
        to_rebase_container_index: NodeIndex,
        onto: &WorkspaceSnapshotGraph,
        onto_change_set: &ChangeSetPointer,
        onto_container_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<(Vec<Conflict>, Vec<Update>)> {
        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
                    kind: edgeref.weight().kind().clone(),
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
                    kind: edgeref.weight().kind().clone(),
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

    pub fn mark_graph_seen(
        &mut self,
        change_set: &ChangeSetPointer,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let seen_at = Utc::now();
        for edge in self.graph.edge_weights_mut() {
            edge.mark_seen_at(change_set, seen_at.clone());
        }
        for node in self.graph.node_weights_mut() {
            node.mark_seen_at(change_set, seen_at.clone());
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

    /// [`StableGraph`] guarantees the stability of [`NodeIndex`] across removals, however there
    /// are **NO** guarantees around the stability of [`EdgeIndex`] across removals. If
    /// [`Self::cleanup()`] has been called, then any [`EdgeIndex`] found before
    /// [`Self::cleanup()`] has run should be considered invalid.
    fn remove_edge(
        &mut self,
        change_set: &ChangeSetPointer,
        source_node_index: NodeIndex,
        target_node_index: NodeIndex,
        edge_kind: EdgeWeightKind,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let mut edges_to_remove = Vec::new();
        let new_source_node_index = self.copy_node_index(source_node_index)?;
        self.replace_references(source_node_index, new_source_node_index)?;

        for edgeref in self
            .graph
            .edges_connecting(new_source_node_index, target_node_index)
        {
            if edgeref.weight().kind() == &edge_kind {
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
                    self.replace_references(
                        previous_container_ordering_node_index,
                        new_container_ordering_node_index,
                    )?;
                }
            }
        }

        self.update_merkle_tree_hash(
            // If we updated the ordering node, that means we've invalidated the container's
            // NodeIndex (new_source_node_index), so we need to find the new NodeIndex to be able
            // to update the container's merkle tree hash.
            self.get_node_index_by_id(self.get_node_weight(new_source_node_index)?.id())?,
        )?;

        Ok(())
    }

    fn replace_references(
        &mut self,
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

                // Find all outgoing edges weights and find the edge targets.
                let mut edges_to_create: Vec<(EdgeWeight, NodeIndex)> = Vec::new();
                for edge_reference in self.graph.edges_directed(old_node_index, Outgoing) {
                    edges_to_create
                        .push((edge_reference.weight().clone(), edge_reference.target()));
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
            }
        }

        // Use the new version of the old root node as our root node.
        if let Some(new_root_node_index) = old_to_new_node_indices.get(&self.root_index) {
            self.root_index = *new_root_node_index;
        }

        Ok(())
    }

    pub fn update_content(
        &mut self,
        change_set: &ChangeSetPointer,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let original_node_index = self.get_node_index_by_id(id)?;
        let new_node_index = self.copy_node_index(original_node_index)?;
        let node_weight = self.get_node_weight_mut(new_node_index)?;
        node_weight.increment_vector_clock(change_set)?;
        node_weight.new_content_hash(new_content_hash)?;

        self.replace_references(original_node_index, new_node_index)
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
        let new_node_index = self.copy_node_index(original_node_index)?;
        let node_weight = self.get_node_weight_mut(new_node_index)?;
        node_weight.set_order(change_set, new_order)?;

        self.replace_references(original_node_index, new_node_index)
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
            .unwrap_or_else(Vec::new);

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
                    // This is the name of the argument to the function.
                    EdgeWeightKind::Argument(arg_name) => hasher.update(arg_name.as_bytes()),
                    // This is the key for an entry in a map.
                    EdgeWeightKind::Contain(Some(key)) => hasher.update(key.as_bytes()),

                    // Nothing to do, as these EdgeWeightKind do not encode extra information
                    // in the edge itself.
                    EdgeWeightKind::Contain(None)
                    | EdgeWeightKind::DataProvider
                    | EdgeWeightKind::Ordering
                    | EdgeWeightKind::Prop
                    | EdgeWeightKind::Prototype
                    | EdgeWeightKind::Proxy
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
}

fn ordering_node_indexes_for_node_index(
    snapshot: &WorkspaceSnapshotGraph,
    node_index: NodeIndex,
) -> Vec<NodeIndex> {
    snapshot
        .graph
        .edges_directed(node_index, Outgoing)
        .filter_map(|edge_reference| {
            if edge_reference.weight().kind() == &EdgeWeightKind::Ordering {
                if matches!(
                    snapshot.get_node_weight(edge_reference.target()),
                    Ok(NodeWeight::Ordering(_))
                ) {
                    return Some(edge_reference.target());
                }
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
            if edge_reference.weight().kind() == &EdgeWeightKind::Prop {
                if matches!(
                    snapshot.get_node_weight(edge_reference.target()),
                    Ok(NodeWeight::Prop(_))
                ) {
                    return Some(edge_reference.target());
                }
            }
            None
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ComponentId, ContentHash, FuncId, PropId, PropKind, SchemaId, SchemaVariantId};
    use pretty_assertions_sorted::assert_eq;

    #[derive(Debug, PartialEq)]
    struct ConflictsAndUpdates {
        conflicts: Vec<Conflict>,
        updates: Vec<Update>,
    }

    #[test]
    fn new() {
        let change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        assert!(graph.is_acyclic_directed());
    }

    #[test]
    fn add_nodes_and_edges() {
        let change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let func_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
        let prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                func_index,
            )
            .expect("Unable to add root -> func edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(func_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add prop -> func edge");

        assert!(graph.is_acyclic_directed());
    }

    #[test]
    fn cyclic_failure() {
        let change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                initial_component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                initial_schema_node_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to find NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                initial_schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to find NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to find NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let pre_cycle_root_index = graph.root_index;

        // This should cause a cycle.
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to find NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to find NodeIndex"),
            )
            .expect_err("Created a cycle");

        assert_eq!(pre_cycle_root_index, graph.root_index,);
    }

    #[test]
    fn update_content() {
        let change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
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
        let initial_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let initial_change_set = &initial_change_set;
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
            .expect("Unable to generate Ulid");
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
                initial_graph.root_index,
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        initial_graph.dot();

        let new_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = initial_graph.clone();

        let component_id = new_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
                new_graph.root_index,
                EdgeWeight::new(new_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        new_graph
            .add_edge(
                new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(new_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                new_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
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
        let initial_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
            .expect("Unable to generate Ulid");
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
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        let new_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();

        let new_onto_component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                new_onto_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(new_onto_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
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
                assert_eq!(&EdgeWeightKind::Use, edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_no_conflicts_with_updates_on_both_sides() {
        let initial_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
            .expect("Unable to generate Ulid");
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
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        let new_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();

        let component_id = new_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
                new_graph.root_index,
                EdgeWeight::new(new_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        new_graph
            .add_edge(
                new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(new_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                new_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        println!("new graph (Root {:?}):", new_graph.root_index);
        new_graph.dot();

        let new_onto_component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                new_onto_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(new_onto_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
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
                assert_eq!(&EdgeWeightKind::Use, edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_with_content_conflict() {
        let initial_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
            .expect("Unable to generate Ulid");
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
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.cleanup();
        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        let new_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
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
        let initial_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
            .expect("Unable to generate Ulid");
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
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.cleanup();
        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        let new_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();

        base_graph
            .remove_edge(
                base_change_set,
                base_graph.root_index,
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeightKind::Use,
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
        let initial_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        // Docker Image Schema
        let docker_image_schema_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                docker_image_schema_index,
            )
            .expect("Unable to add root -> schema edge");

        // Docker Image Schema Variant
        let docker_image_schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
                base_graph
                    .get_node_index_by_id(docker_image_schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                docker_image_schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        // Nginx Docker Image Component
        let nginx_docker_image_component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                nginx_docker_image_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(nginx_docker_image_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(docker_image_schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        // Alpine Component
        let alpine_component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                alpine_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(alpine_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(docker_image_schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        // Butane Schema
        let butane_schema_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                butane_schema_index,
            )
            .expect("Unable to add root -> schema edge");

        // Butane Schema Variant
        let butane_schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
                base_graph
                    .get_node_index_by_id(butane_schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                butane_schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        // Nginx Butane Component
        let nginx_butane_component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                nginx_butane_node_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(nginx_butane_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(butane_schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.cleanup();
        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        // Create a new change set to cause some problems!
        let new_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
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
                EdgeWeightKind::Use,
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

    #[test]
    fn add_ordered_node() {
        let change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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

        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let func_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                func_index,
            )
            .expect("Unable to add root -> func edge");

        let prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let prop_index = graph
            .add_ordered_node(
                change_set,
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
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(func_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add prop -> func edge");
        graph.cleanup();
        graph.dot();

        let ordered_prop_1_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_1_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create uses edge weight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add prop -> ordered_prop_1 edge");

        let ordered_prop_2_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_2_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create uses edge weight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add prop -> ordered_prop_2 edge");

        let ordered_prop_3_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_3_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create uses edge weight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add prop -> ordered_prop_3 edge");
        graph.cleanup();
        graph.dot();

        assert_eq!(
            vec![
                ordered_prop_1_index,
                ordered_prop_2_index,
                ordered_prop_3_index,
            ],
            graph
                .ordered_children_for_node(
                    graph
                        .get_node_index_by_id(prop_id)
                        .expect("Unable to get prop NodeIndex")
                )
                .expect("Unable to find ordered children for node")
                .expect("Node is not an ordered node")
        );
    }

    #[test]
    fn reorder_ordered_node() {
        let change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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

        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let func_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                func_index,
            )
            .expect("Unable to add root -> func edge");

        let prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let prop_index = graph
            .add_ordered_node(
                change_set,
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
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(func_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add prop -> func edge");
        graph.cleanup();
        graph.dot();

        let ordered_prop_1_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_1_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create uses edge weight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add prop -> ordered_prop_1 edge");

        let ordered_prop_2_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_2_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create uses edge weight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add prop -> ordered_prop_2 edge");

        let ordered_prop_3_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_3_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create uses edge weight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add prop -> ordered_prop_3 edge");

        let ordered_prop_4_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_4_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_4_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_4_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create uses edge weight"),
                ordered_prop_4_index,
            )
            .expect("Unable to add prop -> ordered_prop_4 edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            vec![
                ordered_prop_1_index,
                ordered_prop_2_index,
                ordered_prop_3_index,
                ordered_prop_4_index,
            ],
            graph
                .ordered_children_for_node(
                    graph
                        .get_node_index_by_id(prop_id)
                        .expect("Unable to get prop NodeIndex")
                )
                .expect("Unable to find ordered children for node")
                .expect("Node is not an ordered node")
        );

        let new_order = vec![
            ordered_prop_2_id,
            ordered_prop_1_id,
            ordered_prop_4_id,
            ordered_prop_3_id,
        ];

        graph
            .update_order(change_set, prop_id, new_order)
            .expect("Unable to update order of prop's children");

        assert_eq!(
            vec![
                ordered_prop_2_index,
                ordered_prop_1_index,
                ordered_prop_4_index,
                ordered_prop_3_index,
            ],
            graph
                .ordered_children_for_node(
                    graph
                        .get_node_index_by_id(prop_id)
                        .expect("Unable to get prop NodeIndex")
                )
                .expect("Unable to find ordered children for node")
                .expect("Node is not an ordered node")
        );
    }

    #[test]
    fn remove_ordered_node() {
        let change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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

        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let func_id = change_set.generate_ulid().expect("Unable to generate Ulid");
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
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                func_index,
            )
            .expect("Unable to add root -> func edge");

        let prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let prop_index = graph
            .add_ordered_node(
                change_set,
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
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(func_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add prop -> func edge");
        graph.cleanup();
        graph.dot();

        let ordered_prop_1_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_1_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create uses edge weight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add prop -> ordered_prop_1 edge");

        let ordered_prop_2_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_2_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create uses edge weight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add prop -> ordered_prop_2 edge");

        let ordered_prop_3_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_3_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create uses edge weight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add prop -> ordered_prop_3 edge");

        let ordered_prop_4_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_4_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_4_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_4_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create uses edge weight"),
                ordered_prop_4_index,
            )
            .expect("Unable to add prop -> ordered_prop_4 edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            vec![
                ordered_prop_1_index,
                ordered_prop_2_index,
                ordered_prop_3_index,
                ordered_prop_4_index,
            ],
            graph
                .ordered_children_for_node(
                    graph
                        .get_node_index_by_id(prop_id)
                        .expect("Unable to get prop NodeIndex")
                )
                .expect("Unable to find ordered children for node")
                .expect("Node is not an ordered node")
        );

        graph
            .remove_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeIndex for prop"),
                ordered_prop_2_index,
                EdgeWeightKind::Use,
            )
            .expect("Unable to remove prop -> ordered_prop_2 edge");

        assert_eq!(
            vec![
                ordered_prop_1_index,
                ordered_prop_3_index,
                ordered_prop_4_index,
            ],
            graph
                .ordered_children_for_node(
                    graph
                        .get_node_index_by_id(prop_id)
                        .expect("Unable to get prop NodeIndex")
                )
                .expect("Unable to find ordered children for node")
                .expect("Node is not an ordered node")
        );
        if let NodeWeight::Ordering(ordering_weight) = graph
            .get_node_weight(
                graph
                    .ordering_node_index_for_container(
                        graph
                            .get_node_index_by_id(prop_id)
                            .expect("Unable to find ordering node for prop"),
                    )
                    .expect("Error getting ordering NodeIndex for prop")
                    .expect("Unable to find ordering NodeIndex"),
            )
            .expect("Unable to get ordering NodeWeight for ordering node")
        {
            assert_eq!(
                &vec![ordered_prop_1_id, ordered_prop_3_id, ordered_prop_4_id],
                ordering_weight.order()
            );
        } else {
            panic!("Unable to destructure ordering node weight");
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_ordering_no_conflicts_no_updates_in_base() {
        let initial_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let initial_change_set = &initial_change_set;
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
            .expect("Unable to generate Ulid");
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
                initial_graph.root_index,
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let container_prop_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let container_prop_index = initial_graph
            .add_ordered_node(
                initial_change_set,
                NodeWeight::new_content(
                    initial_change_set,
                    container_prop_id,
                    ContentAddress::Prop(ContentHash::new(
                        container_prop_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add container prop");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                container_prop_index,
            )
            .expect("Unable to add schema variant -> container prop edge");

        let ordered_prop_1_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_1_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 1");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add container prop -> ordered prop 1 edge");

        let ordered_prop_2_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_2_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 2");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add container prop -> ordered prop 2 edge");

        let ordered_prop_3_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_3_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 3");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add container prop -> ordered prop 3 edge");

        let ordered_prop_4_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_4_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_4_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_4_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 4");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_4_index,
            )
            .expect("Unable to add container prop -> ordered prop 4 edge");

        initial_graph.cleanup();
        initial_graph.dot();

        let new_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = initial_graph.clone();

        let ordered_prop_5_id = new_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_5_index = new_graph
            .add_node(
                NodeWeight::new_content(
                    new_change_set,
                    ordered_prop_5_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_5_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 5");
        new_graph
            .add_ordered_edge(
                new_change_set,
                new_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(new_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_5_index,
            )
            .expect("Unable to add container prop -> ordered prop 5 edge");

        new_graph.cleanup();
        new_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(new_change_set, &initial_graph, initial_change_set)
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);
        assert_eq!(Vec::<Update>::new(), updates);
    }

    #[test]
    fn detect_conflicts_and_updates_simple_ordering_no_conflicts_with_updates_in_base() {
        let initial_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let initial_change_set = &initial_change_set;
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
            .expect("Unable to generate Ulid");
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
                initial_graph.root_index,
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let container_prop_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let container_prop_index = initial_graph
            .add_ordered_node(
                initial_change_set,
                NodeWeight::new_content(
                    initial_change_set,
                    container_prop_id,
                    ContentAddress::Prop(ContentHash::new(
                        container_prop_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add container prop");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                container_prop_index,
            )
            .expect("Unable to add schema variant -> container prop edge");

        let ordered_prop_1_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_1_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 1");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add container prop -> ordered prop 1 edge");

        let ordered_prop_2_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_2_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 2");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add container prop -> ordered prop 2 edge");

        let ordered_prop_3_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_3_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 3");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add container prop -> ordered prop 3 edge");

        let ordered_prop_4_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_4_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_4_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_4_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 4");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_4_index,
            )
            .expect("Unable to add container prop -> ordered prop 4 edge");

        initial_graph.dot();

        let new_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let new_graph = initial_graph.clone();

        let ordered_prop_5_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_5_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_5_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_5_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 5");
        let new_edge_weight = EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
            .expect("Unable to create EdgeWeight");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                new_edge_weight.clone(),
                ordered_prop_5_index,
            )
            .expect("Unable to add container prop -> ordered prop 5 edge");

        new_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(new_change_set, &initial_graph, initial_change_set)
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);
        assert_eq!(
            vec![
                Update::NewEdge {
                    source: new_graph
                        .get_node_index_by_id(container_prop_id)
                        .expect("Unable to get NodeIndex"),
                    destination: initial_graph
                        .get_node_index_by_id(ordered_prop_5_id)
                        .expect("Unable to get NodeIndex"),
                    edge_weight: new_edge_weight,
                },
                Update::ReplaceSubgraph {
                    new: initial_graph
                        .ordering_node_index_for_container(
                            initial_graph
                                .get_node_index_by_id(container_prop_id)
                                .expect("Unable to get container NodeIndex")
                        )
                        .expect("Unable to get new ordering NodeIndex")
                        .expect("Ordering NodeIndex not found"),
                    old: new_graph
                        .ordering_node_index_for_container(
                            new_graph
                                .get_node_index_by_id(container_prop_id)
                                .expect("Unable to get container NodeIndex")
                        )
                        .expect("Unable to get old ordering NodeIndex")
                        .expect("Ordering NodeIndex not found"),
                },
            ],
            updates
        );
    }

    #[test]
    fn detect_conflicts_and_updates_simple_ordering_with_conflicting_ordering_updates() {
        let initial_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let initial_change_set = &initial_change_set;
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
            .expect("Unable to generate Ulid");
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
                initial_graph.root_index,
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let container_prop_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let container_prop_index = initial_graph
            .add_ordered_node(
                initial_change_set,
                NodeWeight::new_content(
                    initial_change_set,
                    container_prop_id,
                    ContentAddress::Prop(ContentHash::new(
                        container_prop_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add container prop");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                container_prop_index,
            )
            .expect("Unable to add schema variant -> container prop edge");

        let ordered_prop_1_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_1_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 1");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add container prop -> ordered prop 1 edge");

        let ordered_prop_2_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_2_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 2");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add container prop -> ordered prop 2 edge");

        let ordered_prop_3_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_3_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 3");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add container prop -> ordered prop 3 edge");

        let ordered_prop_4_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_4_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_4_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_4_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 4");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_4_index,
            )
            .expect("Unable to add container prop -> ordered prop 4 edge");

        initial_graph.dot();

        let new_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = initial_graph.clone();

        let new_order = vec![
            ordered_prop_2_id,
            ordered_prop_1_id,
            ordered_prop_4_id,
            ordered_prop_3_id,
        ];
        new_graph
            .update_order(new_change_set, container_prop_id, new_order)
            .expect("Unable to update order of container prop's children");

        let ordered_prop_5_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_5_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_5_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_5_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 5");
        let new_edge_weight = EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
            .expect("Unable to create EdgeWeight");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                new_edge_weight.clone(),
                ordered_prop_5_index,
            )
            .expect("Unable to add container prop -> ordered prop 5 edge");

        new_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(new_change_set, &initial_graph, initial_change_set)
            .expect("Unable to detect conflicts and updates");

        assert_eq!(
            vec![Conflict::ChildOrder {
                onto: initial_graph
                    .ordering_node_index_for_container(
                        initial_graph
                            .get_node_index_by_id(container_prop_id)
                            .expect("Unable to get container NodeIndex")
                    )
                    .expect("Unable to get ordering NodeIndex")
                    .expect("Ordering NodeIndex not found"),
                to_rebase: new_graph
                    .ordering_node_index_for_container(
                        new_graph
                            .get_node_index_by_id(container_prop_id)
                            .expect("Unable to get container NodeIndex")
                    )
                    .expect("Unable to get ordering NodeIndex")
                    .expect("Ordering NodeIndex not found"),
            }],
            conflicts
        );
        assert_eq!(
            vec![Update::NewEdge {
                source: new_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get new_graph container NodeIndex"),
                destination: initial_graph
                    .get_node_index_by_id(ordered_prop_5_id)
                    .expect("Unable to get ordered prop 5 NodeIndex"),
                edge_weight: new_edge_weight,
            }],
            updates
        );
    }

    #[test]
    fn detect_conflicts_and_updates_simple_ordering_with_no_conflicts_add_in_onto_remove_in_to_rebase(
    ) {
        let initial_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let initial_change_set = &initial_change_set;
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
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
            .expect("Unable to generate Ulid");
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
                initial_graph.root_index,
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let container_prop_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let container_prop_index = initial_graph
            .add_ordered_node(
                initial_change_set,
                NodeWeight::new_content(
                    initial_change_set,
                    container_prop_id,
                    ContentAddress::Prop(ContentHash::new(
                        container_prop_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add container prop");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                container_prop_index,
            )
            .expect("Unable to add schema variant -> container prop edge");

        let ordered_prop_1_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_1_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 1");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add container prop -> ordered prop 1 edge");

        let ordered_prop_2_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_2_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 2");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add container prop -> ordered prop 2 edge");

        let ordered_prop_3_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_3_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 3");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add container prop -> ordered prop 3 edge");

        let ordered_prop_4_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_4_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_4_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_4_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 4");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_4_index,
            )
            .expect("Unable to add container prop -> ordered prop 4 edge");

        initial_graph.cleanup();
        initial_graph
            .mark_graph_seen(initial_change_set)
            .expect("Unable to update recently seen information");
        // initial_graph.dot();

        let new_change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = initial_graph.clone();

        new_graph
            .remove_edge(
                new_change_set,
                new_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get container NodeIndex"),
                ordered_prop_2_index,
                EdgeWeightKind::Use,
            )
            .expect("Unable to remove container prop -> prop 2 edge");

        let ordered_prop_5_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_5_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_5_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_5_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 5");

        let new_edge_weight = EdgeWeight::new(initial_change_set, EdgeWeightKind::Use)
            .expect("Unable to create EdgeWeight");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                new_edge_weight.clone(),
                ordered_prop_5_index,
            )
            .expect("Unable to add container prop -> ordered prop 5 edge");

        initial_graph.cleanup();
        initial_graph.dot();

        new_graph.cleanup();
        new_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(new_change_set, &initial_graph, initial_change_set)
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);
        assert_eq!(
            vec![Update::NewEdge {
                source: new_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get new_graph container NodeIndex"),
                destination: initial_graph
                    .get_node_index_by_id(ordered_prop_5_id)
                    .expect("Unable to get ordered prop 5 NodeIndex"),
                edge_weight: new_edge_weight,
            }],
            updates
        );
    }

    #[test]
    fn attribute_value_build_view() {
        let change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let mut content_store = crate::content::Store::new();

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (schema_content_hash, _) = content_store
            .add(serde_json::json!("Schema A"))
            .expect("Unable to add to content store");
        let schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(schema_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_node_index,
            )
            .expect("Unable to add root -> schema edge");

        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (schema_variant_content_hash, _) = content_store
            .add(serde_json::json!("Schema Variant A"))
            .expect("Unable to add to content store");
        let schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(schema_variant_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (root_prop_content_hash, _) = content_store
            .add(serde_json::json!("Root prop"))
            .expect("Unable to add to content store");
        let root_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    root_prop_id,
                    PropKind::Object,
                    "root",
                    root_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                root_prop_node_index,
            )
            .expect("Unable to add schema variant -> root prop edge");

        let si_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (si_prop_content_hash, _) = content_store
            .add(serde_json::json!("SI Prop Content"))
            .expect("Unable to add to content store");
        let si_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    si_prop_id,
                    PropKind::Object,
                    "si",
                    si_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add si prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                si_prop_node_index,
            )
            .expect("Unable to add root prop -> si prop edge");

        let name_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (name_prop_content_hash, _) = content_store
            .add(serde_json::json!("Name Prop Content"))
            .expect("Unable to add to content store");
        let name_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    name_prop_id,
                    PropKind::Object,
                    "name",
                    name_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add name prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                name_prop_node_index,
            )
            .expect("Unable to add si prop -> name prop edge");

        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (component_content_hash, _) = content_store
            .add(serde_json::json!("Component Content"))
            .expect("Unable to add to content store");
        let component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(component_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let root_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (root_av_content_hash, _) = content_store
            .add(serde_json::json!({}))
            .expect("Unable to add to content store");
        let root_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    root_av_id,
                    ContentAddress::AttributeValue(root_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                root_av_node_index,
            )
            .expect("Unable to add component -> root av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add root av -> root prop edge");

        let si_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (si_av_content_hash, _) = content_store
            .add(serde_json::json!({}))
            .expect("Unable to add to content store");
        let si_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    si_av_id,
                    ContentAddress::AttributeValue(si_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add si av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                si_av_node_index,
            )
            .expect("Unable to add root av -> si av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add si av -> si prop edge");

        let name_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (name_av_content_hash, _) = content_store
            .add(serde_json::json!("component name"))
            .expect("Unable to add to content store");
        let name_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    name_av_id,
                    ContentAddress::AttributeValue(name_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add name av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeWeight"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                name_av_node_index,
            )
            .expect("Unable to add si av -> name av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(name_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(name_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to create name av -> name prop edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            serde_json::json![{"si": {"name": "component name"}}],
            graph
                .attribute_value_view(
                    &content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex")
                )
                .expect("Unable to generate attribute value view"),
        );
    }

    #[test]
    fn attribute_value_build_view_unordered_object() {
        let change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let mut content_store = crate::content::Store::new();

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (schema_content_hash, _) = content_store
            .add(serde_json::json!("Schema A"))
            .expect("Unable to add to content store");
        let schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(schema_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_node_index,
            )
            .expect("Unable to add root -> schema edge");

        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (schema_variant_content_hash, _) = content_store
            .add(serde_json::json!("Schema Variant A"))
            .expect("Unable to add to content store");
        let schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(schema_variant_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (root_prop_content_hash, _) = content_store
            .add(serde_json::json!("Root prop"))
            .expect("Unable to add to content store");
        let root_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    root_prop_id,
                    PropKind::Object,
                    "root",
                    root_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                root_prop_node_index,
            )
            .expect("Unable to add schema variant -> root prop edge");

        let si_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (si_prop_content_hash, _) = content_store
            .add(serde_json::json!("SI Prop Content"))
            .expect("Unable to add to content store");
        let si_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    si_prop_id,
                    PropKind::Object,
                    "si",
                    si_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add si prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                si_prop_node_index,
            )
            .expect("Unable to add root prop -> si prop edge");

        let name_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (name_prop_content_hash, _) = content_store
            .add(serde_json::json!("Name Prop Content"))
            .expect("Unable to add to content store");
        let name_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    name_prop_id,
                    PropKind::Object,
                    "name",
                    name_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add name prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                name_prop_node_index,
            )
            .expect("Unable to add si prop -> name prop edge");

        let description_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (description_prop_content_hash, _) = content_store
            .add(serde_json::json!("Description Prop Content"))
            .expect("Unable to add to content store");
        let description_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    description_prop_id,
                    PropKind::String,
                    "description",
                    description_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add description prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                description_prop_node_index,
            )
            .expect("Unable to add si prop -> description prop edge");

        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (component_content_hash, _) = content_store
            .add(serde_json::json!("Component Content"))
            .expect("Unable to add to content store");
        let component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(component_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let root_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (root_av_content_hash, _) = content_store
            .add(serde_json::json!({}))
            .expect("Unable to add to content store");
        let root_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    root_av_id,
                    ContentAddress::AttributeValue(root_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                root_av_node_index,
            )
            .expect("Unable to add component -> root av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add root av -> root prop edge");

        let si_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (si_av_content_hash, _) = content_store
            .add(serde_json::json!({}))
            .expect("Unable to add to content store");
        let si_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    si_av_id,
                    ContentAddress::AttributeValue(si_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add si av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                si_av_node_index,
            )
            .expect("Unable to add root av -> si av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add si av -> si prop edge");

        let name_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (name_av_content_hash, _) = content_store
            .add(serde_json::json!("component name"))
            .expect("Unable to add to content store");
        let name_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    name_av_id,
                    ContentAddress::AttributeValue(name_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add name av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                name_av_node_index,
            )
            .expect("Unable to add si av -> name av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(name_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(name_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to create name av -> name prop edge");

        let description_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (description_av_content_hash, _) = content_store
            .add(serde_json::json!("Component description"))
            .expect("Unable to add to content store");
        let description_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    description_av_id,
                    ContentAddress::AttributeValue(description_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add description av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                description_av_node_index,
            )
            .expect("Unable to add si av -> description av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(description_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(description_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add description av -> description prop edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            serde_json::json![{
                "si": {
                    "description": "Component description",
                    "name": "component name",
                }
            }],
            graph
                .attribute_value_view(
                    &content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex")
                )
                .expect("Unable to generate attribute value view"),
        );
    }

    #[test]
    fn attribute_value_build_view_ordered_array() {
        let change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let mut content_store = crate::content::Store::new();

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (schema_content_hash, _) = content_store
            .add(serde_json::json!("Schema A"))
            .expect("Unable to add to content store");
        let schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(schema_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_node_index,
            )
            .expect("Unable to add root -> schema edge");

        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (schema_variant_content_hash, _) = content_store
            .add(serde_json::json!("Schema Variant A"))
            .expect("Unable to add to content store");
        let schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(schema_variant_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (root_prop_content_hash, _) = content_store
            .add(serde_json::json!("Root prop"))
            .expect("Unable to add to content store");
        let root_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    root_prop_id,
                    PropKind::Object,
                    "root",
                    root_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                root_prop_node_index,
            )
            .expect("Unable to add schema variant -> root prop edge");

        let domain_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (domain_prop_content_hash, _) = content_store
            .add(serde_json::json!("domain Prop Content"))
            .expect("Unable to add to content store");
        let domain_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    domain_prop_id,
                    PropKind::Object,
                    "domain",
                    domain_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add domain prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                domain_prop_node_index,
            )
            .expect("Unable to add root prop -> domain prop edge");

        let ports_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (ports_prop_content_hash, _) = content_store
            .add(serde_json::json!("ports Prop Content"))
            .expect("Unable to add to content store");
        let ports_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    ports_prop_id,
                    PropKind::Array,
                    "ports",
                    ports_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ports prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                ports_prop_node_index,
            )
            .expect("Unable to add domain prop -> ports prop edge");

        let port_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (port_prop_content_hash, _) = content_store
            .add(serde_json::json!("port Prop Content"))
            .expect("Unable to add to content store");
        let port_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    port_prop_id,
                    PropKind::String,
                    "port",
                    port_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(ports_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                port_prop_node_index,
            )
            .expect("Unable to add ports prop -> port prop edge");

        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (component_content_hash, _) = content_store
            .add(serde_json::json!("Component Content"))
            .expect("Unable to add to content store");
        let component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(component_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let root_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (root_av_content_hash, _) = content_store
            .add(serde_json::json!({}))
            .expect("Unable to add to content store");
        let root_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    root_av_id,
                    ContentAddress::AttributeValue(root_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                root_av_node_index,
            )
            .expect("Unable to add component -> root av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add root av -> root prop edge");

        let domain_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (domain_av_content_hash, _) = content_store
            .add(serde_json::json!({}))
            .expect("Unable to add to content store");
        let domain_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    domain_av_id,
                    ContentAddress::AttributeValue(domain_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add domain av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                domain_av_node_index,
            )
            .expect("Unable to add root av -> domain av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(domain_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add domain av -> domain prop edge");

        let ports_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (ports_av_content_hash, _) = content_store
            .add(serde_json::json!([]))
            .expect("Unable to add to content store");
        let ports_av_node_index = graph
            .add_ordered_node(
                change_set,
                NodeWeight::new_content(
                    change_set,
                    ports_av_id,
                    ContentAddress::AttributeValue(ports_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ports av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                ports_av_node_index,
            )
            .expect("Unable to add domain av -> ports av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(ports_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to create ports av -> ports prop edge");

        let port1_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (port1_av_content_hash, _) = content_store
            .add(serde_json::json!("Port 1"))
            .expect("Unable to add to content store");
        let port1_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port1_av_id,
                    ContentAddress::AttributeValue(port1_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 1 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port1_av_node_index,
            )
            .expect("Unable to add ports av -> port 1 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port1_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 1 av -> port prop edge");

        let port2_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (port2_av_content_hash, _) = content_store
            .add(serde_json::json!("Port 2"))
            .expect("Unable to add to content store");
        let port2_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port2_av_id,
                    ContentAddress::AttributeValue(port2_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 2 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port2_av_node_index,
            )
            .expect("Unable to add ports av -> port 2 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port2_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 2 av -> port prop edge");

        let port3_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (port3_av_content_hash, _) = content_store
            .add(serde_json::json!("Port 3"))
            .expect("Unable to add to content store");
        let port3_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port3_av_id,
                    ContentAddress::AttributeValue(port3_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 3 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port3_av_node_index,
            )
            .expect("Unable to add ports av -> port 3 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port3_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 3 av -> port prop edge");

        let port4_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (port4_av_content_hash, _) = content_store
            .add(serde_json::json!("Port 4"))
            .expect("Unable to add to content store");
        let port4_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port4_av_id,
                    ContentAddress::AttributeValue(port4_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 4 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port4_av_node_index,
            )
            .expect("Unable to add ports av -> port 4 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port4_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 4 av -> port prop edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            serde_json::json![{
                "domain": {
                    "ports": [
                        "Port 1",
                        "Port 2",
                        "Port 3",
                        "Port 4",
                    ],
                }
            }],
            graph
                .attribute_value_view(
                    &content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex")
                )
                .expect("Unable to generate attribute value view"),
        );

        let new_order = vec![port3_av_id, port1_av_id, port4_av_id, port2_av_id];
        graph
            .update_order(change_set, ports_av_id, new_order)
            .expect("Unable to update order of ports attribute value's children");
        assert_eq!(
            serde_json::json![{
                "domain": {
                    "ports": [
                        "Port 3",
                        "Port 1",
                        "Port 4",
                        "Port 2",
                    ]
                }
            }],
            graph
                .attribute_value_view(
                    &content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex")
                )
                .expect("Unable to generate attribute value view"),
        );

        let port5_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (port5_av_content_hash, _) = content_store
            .add(serde_json::json!("Port 5"))
            .expect("Unable to add to content store");
        let port5_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port5_av_id,
                    ContentAddress::AttributeValue(port5_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 5 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port5_av_node_index,
            )
            .expect("Unable to add ports av -> port 5 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port5_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 5 av -> port prop edge");

        assert_eq!(
            serde_json::json![{
                "domain": {
                    "ports": [
                        "Port 3",
                        "Port 1",
                        "Port 4",
                        "Port 2",
                        "Port 5",
                    ]
                }
            }],
            graph
                .attribute_value_view(
                    &content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex")
                )
                .expect("Unable to generate attribute value view"),
        );
    }

    #[test]
    fn attribute_value_build_view_ordered_map() {
        let change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let mut content_store = crate::content::Store::new();

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (schema_content_hash, _) = content_store
            .add(serde_json::json!("Schema A"))
            .expect("Unable to add to content store");
        let schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(schema_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_node_index,
            )
            .expect("Unable to add root -> schema edge");

        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (schema_variant_content_hash, _) = content_store
            .add(serde_json::json!("Schema Variant A"))
            .expect("Unable to add to content store");
        let schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(schema_variant_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (root_prop_content_hash, _) = content_store
            .add(serde_json::json!("Root prop"))
            .expect("Unable to add to content store");
        let root_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    root_prop_id,
                    PropKind::Object,
                    "root",
                    root_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                root_prop_node_index,
            )
            .expect("Unable to add schema variant -> root prop edge");

        let domain_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (domain_prop_content_hash, _) = content_store
            .add(serde_json::json!("domain Prop Content"))
            .expect("Unable to add to content store");
        let domain_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    domain_prop_id,
                    PropKind::Object,
                    "domain",
                    domain_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add domain prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                domain_prop_node_index,
            )
            .expect("Unable to add root prop -> domain prop edge");

        let environment_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (environment_prop_content_hash, _) = content_store
            .add(serde_json::json!("environment Prop Content"))
            .expect("Unable to add to content store");
        let environment_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    environment_prop_id,
                    PropKind::Array,
                    "environment",
                    environment_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add environment prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                environment_prop_node_index,
            )
            .expect("Unable to add domain prop -> environment prop edge");

        let env_var_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (env_var_prop_content_hash, _) = content_store
            .add(serde_json::json!("port Prop Content"))
            .expect("Unable to add to content store");
        let env_var_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    env_var_prop_id,
                    PropKind::String,
                    "port",
                    env_var_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(environment_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                env_var_prop_node_index,
            )
            .expect("Unable to add environment prop -> env var prop edge");

        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (component_content_hash, _) = content_store
            .add(serde_json::json!("Component Content"))
            .expect("Unable to add to content store");
        let component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(component_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let root_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (root_av_content_hash, _) = content_store
            .add(serde_json::json!({}))
            .expect("Unable to add to content store");
        let root_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    root_av_id,
                    ContentAddress::AttributeValue(root_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)
                    .expect("Unable to create EdgeWeight"),
                root_av_node_index,
            )
            .expect("Unable to add component -> root av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add root av -> root prop edge");

        let domain_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (domain_av_content_hash, _) = content_store
            .add(serde_json::json!({}))
            .expect("Unable to add to content store");
        let domain_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    domain_av_id,
                    ContentAddress::AttributeValue(domain_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add domain av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                domain_av_node_index,
            )
            .expect("Unable to add root av -> domain av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(domain_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add domain av -> domain prop edge");

        let envrionment_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (ports_av_content_hash, _) = content_store
            .add(serde_json::json!({}))
            .expect("Unable to add to content store");
        let environment_av_node_index = graph
            .add_ordered_node(
                change_set,
                NodeWeight::new_content(
                    change_set,
                    envrionment_av_id,
                    ContentAddress::AttributeValue(ports_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add environment av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                environment_av_node_index,
            )
            .expect("Unable to add domain av -> environment av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(environment_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to create environment av -> environment prop edge");

        let env_var1_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (env_var1_av_content_hash, _) = content_store
            .add(serde_json::json!("1111"))
            .expect("Unable to add to content store");
        let port1_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var1_av_id,
                    ContentAddress::AttributeValue(env_var1_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env_var 1 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_1".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                port1_av_node_index,
            )
            .expect("Unable to add environment av -> env var 1 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var1_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 1 av -> env var prop edge");

        let env_var2_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (env_var2_av_content_hash, _) = content_store
            .add(serde_json::json!("2222"))
            .expect("Unable to add to content store");
        let env_var2_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var2_av_id,
                    ContentAddress::AttributeValue(env_var2_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var 2 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_2".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                env_var2_av_node_index,
            )
            .expect("Unable to add environment av -> env var 2 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var2_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 2 av -> env var prop edge");

        let env_var3_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (env_var3_av_content_hash, _) = content_store
            .add(serde_json::json!("3333"))
            .expect("Unable to add to content store");
        let port3_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var3_av_id,
                    ContentAddress::AttributeValue(env_var3_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var 3 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_3".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                port3_av_node_index,
            )
            .expect("Unable to add environment av -> env var 3 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var3_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 3 av -> env var prop edge");

        let env_var4_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (env_var4_av_content_hash, _) = content_store
            .add(serde_json::json!("4444"))
            .expect("Unable to add to content store");
        let env_var4_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var4_av_id,
                    ContentAddress::AttributeValue(env_var4_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var 4 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_4".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                env_var4_av_node_index,
            )
            .expect("Unable to add environment av -> env var 4 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var4_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 4 av -> env var prop edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            serde_json::json![{
                "domain": {
                    "environment": {
                        "PORT_1": "1111",
                        "PORT_2": "2222",
                        "PORT_3": "3333",
                        "PORT_4": "4444",
                    },
                }
            }],
            graph
                .attribute_value_view(
                    &content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex")
                )
                .expect("Unable to generate attribute value view"),
        );

        let new_order = vec![
            env_var3_av_id,
            env_var1_av_id,
            env_var4_av_id,
            env_var2_av_id,
        ];
        graph
            .update_order(change_set, envrionment_av_id, new_order)
            .expect("Unable to update order of environment attribute value's children");
        assert_eq!(
            serde_json::json![{
                "domain": {
                    "environment": {
                        "PORT_3": "3333",
                        "PORT_1": "1111",
                        "PORT_4": "4444",
                        "PORT_2": "2222",
                    },
                }
            }],
            graph
                .attribute_value_view(
                    &content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex")
                )
                .expect("Unable to generate attribute value view"),
        );

        let env_var5_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let (env_var5_av_content_hash, _) = content_store
            .add(serde_json::json!("5555"))
            .expect("Unable to add to content store");
        let env_var5_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var5_av_id,
                    ContentAddress::AttributeValue(env_var5_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var 5 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_5".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                env_var5_av_node_index,
            )
            .expect("Unable to add environment av -> env var 5 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var5_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 5 av -> env var prop edge");

        assert_eq!(
            serde_json::json![{
                "domain": {
                    "environment": {
                        "PORT_3": "3333",
                        "PORT_1": "1111",
                        "PORT_4": "4444",
                        "PORT_2": "2222",
                        "PORT_5": "5555",
                    },
                }
            }],
            graph
                .attribute_value_view(
                    &content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex")
                )
                .expect("Unable to generate attribute value view"),
        );
    }
}
