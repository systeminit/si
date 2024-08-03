use std::collections::{HashMap, HashSet};

use petgraph::{prelude::*, visit::DfsEvent};
use telemetry::prelude::*;

use si_events::{ulid::Ulid, VectorClockId};

use crate::{
    workspace_snapshot::{
        conflict::Conflict, edge_info::EdgeInfo, graph::WorkspaceSnapshotGraphError,
        node_weight::NodeWeight, update::Update, vector_clock::HasVectorClocks, NodeInformation,
    },
    EdgeWeightKind, WorkspaceSnapshotGraphV1,
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
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum OntoNodeDifference {
    MerkleTreeHash,
    NewNode,
}

impl ConflictsAndUpdatesControl {
    fn into_inners(self) -> (petgraph::visit::Control<()>, Vec<Conflict>, Vec<Update>) {
        match self {
            ConflictsAndUpdatesControl::Continue(conflicts, updates) => {
                (petgraph::visit::Control::Continue, conflicts, updates)
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

        // updates.extend(self.maybe_merge_category_nodes()?);

        // Now that we have the full set of updates to be performed, we can check to see if we'd be
        // breaking any "exclusive edge" constraints. We need to wait until after we've detected
        // all of the updates to be performed as adding a second "exclusive" edge is only a
        // violation of the constraint if we are not also removing the first one. We need to ensure
        // that the net result is that there is only one of that edge kind.

        // conflicts.extend(self.detect_exclusive_edge_conflicts_in_updates(&updates)?);

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
                let node_diff = self
                    .onto_node_difference_from_to_rebase(onto_node_index)
                    .map_err(|err| {
                        error!(
                            err=?err,
                            "Error detecting conflicts and updates for onto {:?}",
                            onto_node_index,
                        );
                        event
                    })?;

                Ok(match node_diff {
                    None => petgraph::visit::Control::Prune,
                    Some(OntoNodeDifference::NewNode) => {
                        let node_weight= self.onto_graph.get_node_weight(onto_node_index)
                            .map_err(|err| {
                                error!(err=?err, "Error detecting conflicts and updates for onto: {:?}", onto_node_index);
                                event
                            })?.to_owned();

                        updates.push(Update::NewNode { node_weight });
                        petgraph::visit::Control::Continue
                    }
                    Some(OntoNodeDifference::MerkleTreeHash) => petgraph::visit::Control::Continue,
                })
            }
            DfsEvent::Finish(onto_node_index, _time) => {
                // Even though we're pruning in `DfsEvent::Discover`, we'll still get a `Finish`
                // for the node where we returned a `petgraph::visit::Control::Prune`. Since we
                // already know that there won't be any conflicts/updates with a nodes that have
                // identical merkle tree hashes, we can `Continue`
                let node_diff = self
                    .onto_node_difference_from_to_rebase(onto_node_index)
                    .map_err(|err| {
                        error!(
                            err=?err,
                            "Error detecting conflicts and updates for onto {:?}",
                            onto_node_index,
                        );
                        event
                    })?;
                if node_diff.is_none() {
                    return Ok(petgraph::visit::Control::Continue);
                }

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

    fn onto_node_difference_from_to_rebase(
        &self,
        onto_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<OntoNodeDifference>> {
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

        if to_rebase_node_indexes.is_empty() {
            return Ok(Some(OntoNodeDifference::NewNode));
        }

        // If everything with the same `lineage_id` is identical, then we can prune the
        // graph traversal, and avoid unnecessary lookups/comparisons.
        let mut any_content_with_lineage_is_different = false;

        for to_rebase_node_index in to_rebase_node_indexes {
            let to_rebase_node_weight =
                self.to_rebase_graph.get_node_weight(to_rebase_node_index)?;
            if onto_node_weight.merkle_tree_hash() == to_rebase_node_weight.merkle_tree_hash() {
                // If the merkle tree hashes are the same, then the entire sub-graph is
                // identical, and we don't need to check any further.
                continue;
            }

            any_content_with_lineage_is_different = true
        }

        Ok(if any_content_with_lineage_is_different {
            Some(OntoNodeDifference::MerkleTreeHash)
        } else {
            None
        })
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

        if to_rebase_node_indexes.is_empty() {
            // this node exists in onto, but not in to rebase. We should have
            // produced a NewNode update already. Now we just need to produce
            // new edge updates for all outgoing edges of this node.
            for edgeref in self.onto_graph.edges_directed(onto_node_index, Outgoing) {
                let edge_info = EdgeInfo {
                    source_node_index: edgeref.source(),
                    target_node_index: edgeref.target(),
                    edge_kind: edgeref.weight().kind().into(),
                    edge_index: edgeref.id(),
                };
                updates.push(Update::new_edge(
                    self.onto_graph,
                    &edge_info,
                    edgeref.weight().to_owned(),
                )?);
            }
            return Ok(ConflictsAndUpdatesControl::Continue(conflicts, updates));
        }

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
                    updates.push(Update::ReplaceNode {
                        node_weight: onto_node_weight.to_owned(),
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
                                id: to_rebase_node_weight.id().into(),
                                node_weight_kind: to_rebase_node_weight.into(),
                            };
                            let onto_node_information = NodeInformation {
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
                            id: to_rebase_node_weight.id().into(),
                            node_weight_kind: to_rebase_node_weight.into(),
                        };
                        let onto_node_information = NodeInformation {
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

        // This function is run in `DfsEvent::Finish`, so regardless of whether there are any
        // updates/conflicts, we need to return `Continue`. We shouldn't ever get here if there
        // aren't any differences at all, as we prune the graph during `DfsEvent::Discover`, but
        // the differences might not result in any changes/conflicts in the direction we're doing
        // the comparison.

        Ok(ConflictsAndUpdatesControl::Continue(conflicts, updates))
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
                            id: to_rebase_item_weight.id().into(),
                            node_weight_kind: to_rebase_item_weight.into(),
                        };
                        let container_node_weight = self
                            .to_rebase_graph
                            .get_node_weight(to_rebase_container_index)?;
                        let container_node_information = NodeInformation {
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
                            id: source_node_weight.id().into(),
                            node_weight_kind: source_node_weight.into(),
                        };
                        let target_node_information = NodeInformation {
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
                            let _container_node_information = NodeInformation {
                                id: container_node_weight.id().into(),
                                node_weight_kind: container_node_weight.into(),
                            };
                            let _removed_item_node_information = NodeInformation {
                                id: onto_node_weight.id().into(),
                                node_weight_kind: onto_node_weight.into(),
                            };

                            // NOTE: The clocks don't actually matter anymore. -- Adam & Fletcher
                            //conflicts.push(Conflict::RemoveModifiedItem {
                            //    container: container_node_information,
                            //    removed_item: removed_item_node_information,
                            //});
                        }
                    }
                    None => {
                        // This edge has never been seen by to_rebase
                        updates.push(Update::new_edge(
                            self.onto_graph,
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
}
