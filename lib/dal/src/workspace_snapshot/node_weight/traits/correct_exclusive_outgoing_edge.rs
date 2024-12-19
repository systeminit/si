use std::collections::{HashMap, HashSet};

use petgraph::prelude::*;

use crate::{
    workspace_snapshot::{
        edge_weight::EdgeWeightKindDiscriminants, graph::detector::Update, NodeInformation,
    },
    WorkspaceSnapshotGraphVCurrent,
};

pub trait CorrectExclusiveOutgoingEdge
where
    NodeInformation: for<'a> From<&'a Self>,
{
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants];

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

        let mut removal_set = HashSet::new();
        let mut new_edge_map = HashMap::new();

        let source_node_information: NodeInformation = self.into();
        let source_id = source_node_information.id;

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

        if let Some(node_idx) = graph.get_node_index_by_id_opt(source_id) {
            updates.extend(
                graph
                    .edges_directed(node_idx, Outgoing)
                    .filter_map(|edge_ref| {
                        let destination_weight = graph.get_node_weight_opt(edge_ref.target());
                        let edge_kind = edge_ref.weight().kind().into();

                        new_edge_map
                            .get(&edge_kind)
                            .and_then(|&new_edge_destination_id| {
                                destination_weight.and_then(|weight| {
                                    let destination: NodeInformation = weight.into();

                                    // If this is not the last NewEdge for this
                                    // kind, and the edge will not already be
                                    // removed, then we need to remove this edge
                                    // since it will still be in the graph
                                    // otherwise.
                                    if destination.id != new_edge_destination_id
                                        && !removal_set.contains(&(edge_kind, destination.id))
                                    {
                                        removal_set.insert((edge_kind, destination.id));
                                        Some(Update::RemoveEdge {
                                            source: source_node_information,
                                            destination,
                                            edge_kind,
                                        })
                                    } else {
                                        None
                                    }
                                })
                            })
                    }),
            );
        }

        updates
    }
}
