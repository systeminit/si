use std::collections::{
    HashMap,
    HashSet,
};

use petgraph::prelude::*;
use si_split_graph::CustomNodeWeight;

use crate::{
    WorkspaceSnapshotGraphVCurrent,
    workspace_snapshot::{
        NodeInformation,
        edge_weight::EdgeWeightKindDiscriminants,
        graph::detector::Update,
        split_snapshot::{
            SplitSnapshotGraphVCurrent,
            UpdateVCurrent,
        },
    },
};

pub trait ExclusiveOutgoingEdges {
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants];
}

pub trait SplitCorrectExclusiveOutgoingEdge
where
    Self: CustomNodeWeight + ExclusiveOutgoingEdges,
{
    fn correct_exclusive_outgoing_edges_split(
        &self,
        graph: &SplitSnapshotGraphVCurrent,
        mut updates: Vec<UpdateVCurrent>,
    ) -> Vec<UpdateVCurrent> {
        let exclusive_edge_kinds = self.exclusive_outgoing_edges();
        if exclusive_edge_kinds.is_empty() {
            return updates;
        }
        let source_id = self.id();

        // A net new node should need no outgoing edge corrections
        if !graph.node_exists(source_id) {
            return updates;
        }

        let mut new_edge_map = HashMap::new();
        let mut removal_set = HashSet::new();

        for update in &updates {
            if !update.source_has_id(source_id) {
                continue;
            }
            let Some(dest_id) = update.destination_id() else {
                continue;
            };
            let Some(kind) = update.edge_weight_kind() else {
                continue;
            };

            match update {
                si_split_graph::Update::NewEdge { .. } => {
                    new_edge_map.insert(kind, dest_id);
                    removal_set.remove(&(kind, dest_id));
                }
                si_split_graph::Update::RemoveEdge { .. } => {
                    removal_set.insert((kind, dest_id));
                }
                _ => {}
            }
        }

        let Some(outgoing_iterator) = graph.edges_directed(source_id, Outgoing).ok() else {
            return updates;
        };

        for outgoing_edge_ref in outgoing_iterator {
            let existing_dest_id = outgoing_edge_ref.target();
            let kind: EdgeWeightKindDiscriminants = outgoing_edge_ref.weight().kind().into();

            let Some(&new_edge_dest_id) = new_edge_map.get(&kind) else {
                continue;
            };

            if new_edge_dest_id != existing_dest_id
                && !removal_set.contains(&(kind, new_edge_dest_id))
            {
                removal_set.insert((kind, existing_dest_id));
                let Some(removals) =
                    UpdateVCurrent::remove_edge_updates(graph, source_id, kind, existing_dest_id)
                        .ok()
                else {
                    continue;
                };

                updates.reserve(removals.len());
                updates.extend(removals);
            }
        }

        updates
    }
}

pub trait CorrectExclusiveOutgoingEdge
where
    NodeInformation: for<'a> From<&'a Self>,
    Self: ExclusiveOutgoingEdges,
{
    /// If a set of updates will produce a new outgoing edge that a node weight
    /// considers "exclusive" (that is, there can only be one of these kinds of
    /// outgoing edges for this node), then we need to ensure we don't add that
    /// new edge to a graph that already has an edge.
    ///
    /// We assume that the set of updates passed in was correct from the start
    /// (that is, the set of updates does not itself include more than one net
    /// outgoing NewEdge update for the exclusive edge)
    fn correct_exclusive_outgoing_edges(
        &self,
        graph: &WorkspaceSnapshotGraphVCurrent,
        mut updates: Vec<Update>,
    ) -> Vec<Update> {
        let exclusive_edge_kinds = self.exclusive_outgoing_edges();
        if exclusive_edge_kinds.is_empty() {
            return updates;
        }
        let source_node_information: NodeInformation = self.into();
        let source_id = source_node_information.id;

        let Some(node_idx) = graph.get_node_index_by_id_opt(source_id) else {
            return updates;
        };

        let mut removal_set = HashSet::new();
        let mut new_edge_map = HashMap::new();

        for update in &updates {
            match update {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if source.id == source_id
                    && exclusive_edge_kinds.contains(&edge_weight.kind().into()) =>
                {
                    let kind: EdgeWeightKindDiscriminants = edge_weight.kind().into();
                    // last new edge for an exclusive wins
                    new_edge_map.insert(kind, destination.id);
                    removal_set.remove(&(kind, destination.id));
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } if source.id == source_id && exclusive_edge_kinds.contains(edge_kind) => {
                    removal_set.insert((*edge_kind, destination.id));
                }
                _ => {}
            }
        }

        updates.extend(
            graph
                .edges_directed(node_idx, Outgoing)
                .filter_map(|edge_ref| {
                    let existing_dest_weight = graph.get_node_weight_opt(edge_ref.target());
                    let edge_kind = edge_ref.weight().kind().into();

                    new_edge_map
                        .get(&edge_kind)
                        .and_then(|&new_edge_destination_id| {
                            existing_dest_weight.and_then(|weight| {
                                let existing_dest: NodeInformation = weight.into();

                                // If this is not the last NewEdge for this
                                // kind, and the edge will not already be
                                // removed, then we need to remove this edge
                                // since it will still be in the graph
                                // otherwise.
                                if existing_dest.id != new_edge_destination_id
                                    && !removal_set.contains(&(edge_kind, existing_dest.id))
                                {
                                    removal_set.insert((edge_kind, existing_dest.id));
                                    Some(Update::RemoveEdge {
                                        source: source_node_information,
                                        destination: existing_dest,
                                        edge_kind,
                                    })
                                } else {
                                    None
                                }
                            })
                        })
                }),
        );

        updates
    }
}
