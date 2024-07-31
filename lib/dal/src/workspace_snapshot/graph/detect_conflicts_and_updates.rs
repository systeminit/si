use std::collections::{HashMap, HashSet};

use petgraph::{prelude::*, visit::DfsEvent};
use telemetry::prelude::*;

use si_events::{ulid::Ulid, VectorClockId};

use crate::{
    workspace_snapshot::{
        conflict::Conflict,
        edge_info::EdgeInfo,
        graph::WorkspaceSnapshotGraphError,
        node_weight::{category_node_weight::CategoryNodeKind, NodeWeight},
        update::Update,
        vector_clock::HasVectorClocks,
        NodeInformation,
    },
    EdgeWeightKind, EdgeWeightKindDiscriminants, WorkspaceSnapshotGraphV1,
};

use super::{ConflictsAndUpdates, WorkspaceSnapshotGraphResult};

pub struct DetectConflictsAndUpdates<'a, 'b> {
    to_rebase_graph: &'a WorkspaceSnapshotGraphV1,
    to_rebase_vector_clock_id: VectorClockId,

    onto_graph: &'b WorkspaceSnapshotGraphV1,
    onto_vector_clock_id: VectorClockId,
}

enum ConflictsAndUpdatesControl {
    Continue(Vec<Conflict>, Vec<Update>),
    Prune(Vec<Conflict>, Vec<Update>),
}

impl ConflictsAndUpdatesControl {
    fn into_inners(self) -> (petgraph::visit::Control<()>, Vec<Conflict>, Vec<Update>) {
        match self {
            ConflictsAndUpdatesControl::Continue(conflicts, updates) => {
                (petgraph::visit::Control::Continue, conflicts, updates)
            }
            ConflictsAndUpdatesControl::Prune(conflicts, updates) => {
                (petgraph::visit::Control::Prune, conflicts, updates)
            }
        }
    }
}

impl<'a, 'b> DetectConflictsAndUpdates<'a, 'b> {
    pub fn new(
        to_rebase_graph: &'a WorkspaceSnapshotGraphV1,
        to_rebase_vector_clock_id: VectorClockId,
        onto_graph: &'b WorkspaceSnapshotGraphV1,
        onto_vector_clock_id: VectorClockId,
    ) -> Self {
        Self {
            to_rebase_graph,
            to_rebase_vector_clock_id,
            onto_graph,
            onto_vector_clock_id,
        }
    }

    pub fn detect_conflicts_and_updates(
        &self,
    ) -> WorkspaceSnapshotGraphResult<ConflictsAndUpdates> {
        let mut conflicts: Vec<Conflict> = Vec::new();
        let mut updates: Vec<Update> = Vec::new();
        if let Err(traversal_error) = petgraph::visit::depth_first_search(
            self.onto_graph.graph(),
            Some(self.onto_graph.root()),
            |event| {
                self.detect_conflicts_and_updates_process_dfs_event(
                    event,
                    &mut conflicts,
                    &mut updates,
                )
            },
        ) {
            return Err(WorkspaceSnapshotGraphError::GraphTraversal(traversal_error));
        }

        updates.extend(self.maybe_merge_category_nodes()?);

        // Now that we have the full set of updates to be performed, we can check to see if we'd be
        // breaking any "exclusive edge" constraints. We need to wait until after we've detected
        // all of the updates to be performed as adding a second "exclusive" edge is only a
        // violation of the constraint if we are not also removing the first one. We need to ensure
        // that the net result is that there is only one of that edge kind.

        conflicts.extend(self.detect_exclusive_edge_conflicts_in_updates(&updates)?);

        Ok(ConflictsAndUpdates { conflicts, updates })
    }

    fn detect_conflicts_and_updates_process_dfs_event(
        &self,
        event: DfsEvent<NodeIndex>,
        conflicts: &mut Vec<Conflict>,
        updates: &mut Vec<Update>,
    ) -> Result<petgraph::visit::Control<()>, petgraph::visit::DfsEvent<NodeIndex>> {
        match event {
            DfsEvent::Discover(onto_node_index, _) => {
                let (petgraph_control, node_conflicts, node_updates) = self
                    .detect_conflicts_and_updates_for_node_index(onto_node_index)
                    .map_err(|err| {
                        error!(
                            err=?err,
                            "Error detecting conflicts and updates for onto {:?}",
                            onto_node_index
                        );
                        event
                    })?
                    .into_inners();

                conflicts.extend(node_conflicts);
                updates.extend(node_updates);

                Ok(petgraph_control)
            }
            _ => Ok(petgraph::visit::Control::Continue),
        }
    }

    fn detect_conflicts_and_updates_for_node_index(
        &self,
        onto_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<ConflictsAndUpdatesControl> {
        let mut conflicts = vec![];
        let mut updates = vec![];

        let onto_node_weight = self.onto_graph.get_node_weight(onto_node_index)?;

        let mut to_rebase_node_indexes = HashSet::new();
        if onto_node_index == self.onto_graph.root_index {
            // There can only be one (valid/current) `ContentAddress::Root` at any
            // given moment, and the `lineage_id` isn't really relevant as it's not
            // globally stable (even though it is locally stable). This matters as we
            // may be dealing with a `WorkspaceSnapshotGraph` that is coming to us
            // externally from a module that we're attempting to import. The external
            // `WorkspaceSnapshotGraph` will be `self`, and the "local" one will be
            // `onto`.
            to_rebase_node_indexes.insert(self.to_rebase_graph.root());
        } else {
            // Only retain node indexes... or indices... if they are part of the current
            // graph. There may still be garbage from previous updates to the graph
            // laying around.
            let mut potential_to_rebase_node_indexes = self
                .to_rebase_graph
                .get_node_index_by_lineage(onto_node_weight.lineage_id());
            potential_to_rebase_node_indexes
                .retain(|node_index| self.to_rebase_graph.has_path_to_root(*node_index));
            to_rebase_node_indexes.extend(potential_to_rebase_node_indexes);
        }

        // If everything with the same `lineage_id` is identical, then we can prune the
        // graph traversal, and avoid unnecessary lookups/comparisons.
        let mut any_content_with_lineage_has_changed = false;

        for to_rebase_node_index in to_rebase_node_indexes {
            let to_rebase_node_weight =
                self.to_rebase_graph.get_node_weight(to_rebase_node_index)?;
            if onto_node_weight.merkle_tree_hash() == to_rebase_node_weight.merkle_tree_hash() {
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
                    let onto_node_information = NodeInformation {
                        index: onto_node_index,
                        id: onto_node_weight.id().into(),
                        node_weight_kind: onto_node_weight.clone().into(),
                    };
                    let to_rebase_node_information = NodeInformation {
                        index: to_rebase_node_index,
                        id: to_rebase_node_weight.id().into(),
                        node_weight_kind: to_rebase_node_weight.clone().into(),
                    };
                    // `onto` has changes, but has already seen all of the changes in
                    // `to_rebase`. There is no conflict, and we should update to use the
                    // `onto` node.
                    updates.push(Update::ReplaceSubgraph {
                        onto: onto_node_information,
                        to_rebase: to_rebase_node_information,
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
                            let to_rebase_node_information = NodeInformation {
                                index: to_rebase_node_index,
                                id: to_rebase_node_weight.id().into(),
                                node_weight_kind: to_rebase_node_weight.into(),
                            };
                            let onto_node_information = NodeInformation {
                                index: onto_node_index,
                                id: onto_node_weight.id().into(),
                                node_weight_kind: onto_node_weight.into(),
                            };

                            conflicts.push(Conflict::ChildOrder {
                                to_rebase: to_rebase_node_information,
                                onto: onto_node_information,
                            });
                        }
                    } else {
                        let to_rebase_node_information = NodeInformation {
                            index: to_rebase_node_index,
                            id: to_rebase_node_weight.id().into(),
                            node_weight_kind: to_rebase_node_weight.into(),
                        };
                        let onto_node_information = NodeInformation {
                            index: onto_node_index,
                            id: onto_node_weight.id().into(),
                            node_weight_kind: onto_node_weight.into(),
                        };

                        conflicts.push(Conflict::NodeContent {
                            to_rebase: to_rebase_node_information,
                            onto: onto_node_information,
                        });
                    }
                }
            }

            let (container_conflicts, container_updates) = self
                .find_container_membership_conflicts_and_updates(
                    to_rebase_node_index,
                    onto_node_index,
                )?;

            updates.extend(container_updates);
            conflicts.extend(container_conflicts);
        }

        if any_content_with_lineage_has_changed {
            // There was at least one thing with a merkle tree hash difference, so we need
            // to examine further down the tree to see where the difference(s) are, and
            // where there are conflicts, if there are any.
            Ok(ConflictsAndUpdatesControl::Continue(conflicts, updates))
        } else {
            // Everything to be rebased is identical, so there's no need to examine the
            // rest of the tree looking for differences & conflicts that won't be there.
            Ok(ConflictsAndUpdatesControl::Prune(conflicts, updates))
        }
    }

    fn find_container_membership_conflicts_and_updates(
        &self,
        to_rebase_container_index: NodeIndex,
        onto_container_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<(Vec<Conflict>, Vec<Update>)> {
        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        struct UniqueEdgeInfo {
            pub kind: EdgeWeightKind,
            pub target_lineage: Ulid,
        }

        let mut updates = Vec::new();
        let mut conflicts = Vec::new();

        let mut to_rebase_edges = HashMap::<UniqueEdgeInfo, EdgeInfo>::new();
        for edgeref in self
            .to_rebase_graph
            .graph()
            .edges_directed(to_rebase_container_index, Outgoing)
        {
            let target_node_weight = self.to_rebase_graph.get_node_weight(edgeref.target())?;

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
        for edgeref in self
            .onto_graph
            .graph
            .edges_directed(onto_container_index, Outgoing)
        {
            let target_node_weight = self.onto_graph.get_node_weight(edgeref.target())?;

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

        // This is the last time that to_rebase knows about onto having seen the snapshot
        let to_rebase_root_node = self
            .to_rebase_graph
            .get_node_weight(self.to_rebase_graph.root())?;
        let onto_root_node = self.onto_graph.get_node_weight(self.onto_graph.root())?;

        let to_rebase_last_seen_by_onto_vector_clock_id = to_rebase_root_node
            .vector_clock_recently_seen()
            .entry_for(self.onto_vector_clock_id);

        let onto_last_seen_by_to_rebase_vector_clock = onto_root_node
            .vector_clock_recently_seen()
            .entry_for(self.to_rebase_vector_clock_id);

        for only_to_rebase_edge_info in only_to_rebase_edges.values() {
            let to_rebase_edge_weight = self
                .to_rebase_graph
                .get_edge_weight_opt(only_to_rebase_edge_info.edge_index)?
                .ok_or(WorkspaceSnapshotGraphError::EdgeWeightNotFound)?;
            let to_rebase_item_weight = self
                .to_rebase_graph
                .get_node_weight(only_to_rebase_edge_info.target_node_index)?;

            // This is an edge that is only to_rebase. So either:
            // -- Onto has seen this edge:
            //     -- So, if to_rebase has modified the target of the edge since onto last saw the target,
            //      we should produce a ModifyRemovedItem conflict,
            //     -- OR, if to_rebase has *not* modified the target, we should produce a RemoveEdge update.
            // -- Onto has never seen this edge, becuase it was added *after* onto was forked from to_rebase:
            //     -- So, either we should just silently let the edge stay in to_rebase, OR, we should produce
            //          an exclusive edge conflict if silently adding this edge would lead to an incorrect
            //          graph.

            // This will always be Some(_)
            if let Some(edge_first_seen_by_to_rebase) = to_rebase_edge_weight
                .vector_clock_first_seen()
                .entry_for(self.to_rebase_vector_clock_id)
            {
                let maybe_seen_by_onto_at = if let Some(onto_last_seen_by_to_rebase_vector_clock) =
                    onto_last_seen_by_to_rebase_vector_clock
                {
                    if edge_first_seen_by_to_rebase <= onto_last_seen_by_to_rebase_vector_clock {
                        Some(onto_last_seen_by_to_rebase_vector_clock)
                    } else {
                        None
                    }
                } else {
                    to_rebase_edge_weight
                        .vector_clock_recently_seen()
                        .entry_for(self.onto_vector_clock_id)
                        .or_else(|| {
                            to_rebase_edge_weight
                                .vector_clock_first_seen()
                                .entry_for(self.onto_vector_clock_id)
                        })
                };

                if let Some(seen_by_onto_at) = maybe_seen_by_onto_at {
                    if to_rebase_item_weight
                        .vector_clock_write()
                        .has_entries_newer_than(seen_by_onto_at)
                    {
                        // Item has been modified in `to_rebase` since
                        // `onto` last saw `to_rebase`
                        let node_information = NodeInformation {
                            index: only_to_rebase_edge_info.target_node_index,
                            id: to_rebase_item_weight.id().into(),
                            node_weight_kind: to_rebase_item_weight.into(),
                        };
                        let container_node_weight = self
                            .to_rebase_graph
                            .get_node_weight(to_rebase_container_index)?;
                        let container_node_information = NodeInformation {
                            index: to_rebase_container_index,
                            id: container_node_weight.id().into(),
                            node_weight_kind: container_node_weight.into(),
                        };

                        conflicts.push(Conflict::ModifyRemovedItem {
                            container: container_node_information,
                            modified_item: node_information,
                        });
                    } else {
                        let source_node_weight = self
                            .to_rebase_graph
                            .get_node_weight(only_to_rebase_edge_info.source_node_index)?;
                        let target_node_weight = self
                            .to_rebase_graph
                            .get_node_weight(only_to_rebase_edge_info.target_node_index)?;
                        let source_node_information = NodeInformation {
                            index: only_to_rebase_edge_info.source_node_index,
                            id: source_node_weight.id().into(),
                            node_weight_kind: source_node_weight.into(),
                        };
                        let target_node_information = NodeInformation {
                            index: only_to_rebase_edge_info.target_node_index,
                            id: target_node_weight.id().into(),
                            node_weight_kind: target_node_weight.into(),
                        };
                        updates.push(Update::RemoveEdge {
                            source: source_node_information,
                            destination: target_node_information,
                            edge_kind: only_to_rebase_edge_info.edge_kind,
                        });
                    }
                }
            }
        }

        // - Items unique to `onto`:
        for only_onto_edge_info in only_onto_edges.values() {
            let onto_edge_weight = self
                .onto_graph
                .get_edge_weight_opt(only_onto_edge_info.edge_index)?
                .ok_or(WorkspaceSnapshotGraphError::EdgeWeightNotFound)?;
            let onto_item_weight = self
                .onto_graph
                .get_node_weight(only_onto_edge_info.target_node_index)?;

            // This is an edge that is only in onto, so, either:
            //  -- to_rebase has never seen this edge, so we should produce a New Edge update
            //  -- OR, to_rebase has seen this edge, so:
            //      -- if onto has modified the target of the edge since the last time onto saw to
            //      rebase, we should produce a RemoveModifiedItem conflict
            //      -- OR, onto has not modified the edge, so the edge should stay removed (no
            //      update necessary since it's already gone in to_rebase)

            // this will always be Some(_)
            if let Some(edge_weight_first_seen_by_onto) = onto_edge_weight
                .vector_clock_first_seen()
                .entry_for(self.onto_vector_clock_id)
            {
                let maybe_seen_by_to_rebase_at =
                    if let Some(to_rebase_last_seen_by_onto_vector_clock_id) =
                        to_rebase_last_seen_by_onto_vector_clock_id
                    {
                        if edge_weight_first_seen_by_onto
                            <= to_rebase_last_seen_by_onto_vector_clock_id
                        {
                            Some(to_rebase_last_seen_by_onto_vector_clock_id)
                        } else {
                            None
                        }
                    } else {
                        onto_edge_weight
                            .vector_clock_recently_seen()
                            .entry_for(self.to_rebase_vector_clock_id)
                            .or_else(|| {
                                onto_edge_weight
                                    .vector_clock_first_seen()
                                    .entry_for(self.to_rebase_vector_clock_id)
                            })
                    };

                match maybe_seen_by_to_rebase_at {
                    Some(seen_by_to_rebase_at) => {
                        if onto_item_weight
                            .vector_clock_write()
                            .has_entries_newer_than(seen_by_to_rebase_at)
                        {
                            let container_node_weight = self
                                .to_rebase_graph
                                .get_node_weight(to_rebase_container_index)?;
                            let onto_node_weight = self
                                .onto_graph
                                .get_node_weight(only_onto_edge_info.target_node_index)?;
                            let container_node_information = NodeInformation {
                                index: to_rebase_container_index,
                                id: container_node_weight.id().into(),
                                node_weight_kind: container_node_weight.into(),
                            };
                            let removed_item_node_information = NodeInformation {
                                index: only_onto_edge_info.target_node_index,
                                id: onto_node_weight.id().into(),
                                node_weight_kind: onto_node_weight.into(),
                            };

                            conflicts.push(Conflict::RemoveModifiedItem {
                                container: container_node_information,
                                removed_item: removed_item_node_information,
                            });
                        }
                    }
                    None => {
                        // This edge has never been seen by to_rebase
                        updates.push(Update::new_edge(
                            self.to_rebase_graph,
                            self.onto_graph,
                            to_rebase_container_index,
                            only_onto_edge_info,
                            onto_edge_weight.to_owned(),
                        )?);
                    }
                }
            }
        }

        // - Sets same: No conflicts/updates
        Ok((conflicts, updates))
    }

    fn maybe_merge_category_nodes(&self) -> WorkspaceSnapshotGraphResult<Vec<Update>> {
        Ok(
            match (
                self.to_rebase_graph
                    .get_category_node(None, CategoryNodeKind::DependentValueRoots)?,
                self.onto_graph
                    .get_category_node(None, CategoryNodeKind::DependentValueRoots)?,
            ) {
                (Some((to_rebase_category_id, _)), Some((onto_category_id, _)))
                    if to_rebase_category_id != onto_category_id =>
                {
                    vec![Update::MergeCategoryNodes {
                        to_rebase_category_id,
                        onto_category_id,
                    }]
                }
                _ => vec![],
            },
        )
    }

    fn detect_exclusive_edge_conflicts_in_updates(
        &self,
        updates: &Vec<Update>,
    ) -> WorkspaceSnapshotGraphResult<Vec<Conflict>> {
        let mut conflicts = Vec::new();

        #[derive(Debug, Default, Clone)]
        struct NodeEdgeWeightUpdates {
            additions: Vec<(NodeInformation, NodeInformation)>,
            removals: Vec<(NodeInformation, NodeInformation)>,
        }

        let mut edge_updates: HashMap<
            NodeIndex,
            HashMap<EdgeWeightKindDiscriminants, NodeEdgeWeightUpdates>,
        > = HashMap::new();

        for update in updates {
            match update {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } => {
                    let source_entry = edge_updates.entry(source.index).or_default();
                    let edge_weight_entry = source_entry
                        .entry(edge_weight.kind().clone().into())
                        .or_default();
                    edge_weight_entry.additions.push((*source, *destination));
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } => {
                    let source_entry = edge_updates.entry(source.index).or_default();
                    let edge_weight_entry = source_entry.entry(*edge_kind).or_default();
                    edge_weight_entry.removals.push((*source, *destination));
                }
                _ => { /* Other updates are unused for exclusive edge conflict detection */ }
            }
        }

        for (source_node_index, source_node_updates) in &edge_updates {
            for (edge_weight_kind, edge_kind_updates) in source_node_updates {
                if edge_kind_updates.additions.is_empty() {
                    // There haven't been any new edges added for this EdgeWeightKind, so we can't
                    // have created any Conflict::ExclusiveEdge
                    continue;
                }

                if !self
                    .to_rebase_graph
                    .get_node_weight(*source_node_index)?
                    .is_exclusive_outgoing_edge(*edge_weight_kind)
                {
                    // Nothing to check. This edge weight kind isn't considered exclusive.
                    continue;
                }

                // We can only have (removals.len() + (1 - existing edge count)) additions
                // _at most_, or we'll be creating a Conflict::ExclusiveEdge because there will be
                // multiple of the same edge kind outgoing from the same node.
                let existing_outgoing_edges_of_kind = self
                    .to_rebase_graph
                    .graph()
                    .edges_directed(*source_node_index, Outgoing)
                    .filter(|edge| *edge_weight_kind == edge.weight().kind().into())
                    .count();

                if edge_kind_updates.additions.len()
                    > (edge_kind_updates.removals.len() + (1 - existing_outgoing_edges_of_kind))
                {
                    warn!(
                        "ExclusiveEdgeMismatch: Found {} pre-existing edges. Requested {} removals, {} additions.",
                        existing_outgoing_edges_of_kind,
                        edge_kind_updates.removals.len(),
                        edge_kind_updates.additions.len(),
                    );
                    // The net count of outgoing edges of this kind is >1. Consider *ALL* of the
                    // additions to be in conflict.
                    for (
                        edge_addition_source_node_information,
                        edge_addition_destination_node_information,
                    ) in &edge_kind_updates.additions
                    {
                        conflicts.push(Conflict::ExclusiveEdgeMismatch {
                            source: *edge_addition_source_node_information,
                            destination: *edge_addition_destination_node_information,
                            edge_kind: *edge_weight_kind,
                        });
                    }
                }
            }
        }

        Ok(conflicts)
    }
}
