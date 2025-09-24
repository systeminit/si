use std::collections::HashSet;

use petgraph::Direction::{
    Incoming,
    Outgoing,
};
use si_split_graph::{
    SplitGraph,
    Update,
};

use super::ComponentNodeWeight;
use crate::{
    EdgeWeight,
    EdgeWeightKindDiscriminants,
    NodeWeightDiscriminants,
    workspace_snapshot::{
        node_weight::NodeWeight,
        split_snapshot::{
            self,
            corrections::CorrectTransformsResult,
        },
    },
};

type Graph = SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>;

impl
    split_snapshot::corrections::CorrectTransforms<
        NodeWeight,
        EdgeWeight,
        EdgeWeightKindDiscriminants,
    > for ComponentNodeWeight
{
    fn correct_transforms(
        &self,
        graph: &Graph,
        mut updates: Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>>
    {
        // Net new components do not need any corrections (currently)
        if !graph.node_exists(self.id) {
            return Ok(updates);
        }

        let mut remove_edges = HashSet::new();
        let mut component_will_be_deleted = false;

        for update in &updates {
            match update {
                // If the component is being deleted, the RemoveEdges may be stale (from an old
                // snapshot) and we need to ensure that we truly delete everything. Detected by
                // noticing an edge was removed from the component category:
                //
                //   Category -> Use: <Self>
                Update::RemoveEdge { .. }
                    if update.destination_has_id(self.id)
                        && update
                            .source_is_of_custom_node_kind(NodeWeightDiscriminants::Category) =>
                {
                    component_will_be_deleted = true;
                }
                // It's impossible for this to happen in a single rebase batch, however,
                // theoretically we could combine rebase batches.
                Update::NewEdge { .. }
                    if update.destination_has_id(self.id)
                        && update
                            .source_is_of_custom_node_kind(NodeWeightDiscriminants::Category) =>
                {
                    component_will_be_deleted = false;
                }

                // If SchemaVariant gets set, we are upgrading a component, which disconnects
                // and reconnects prop and socket values and connections. The disconnects may
                // be stale (based on an old snapshot), so when we detect schema upgrade, we
                // redo the disconnects.
                Update::NewEdge { .. }
                    if update.source_has_id(self.id)
                        && update.destination_is_of_custom_node_kind(
                            NodeWeightDiscriminants::SchemaVariant,
                        ) =>
                {
                    // All outgoing edges from the component have to be removed since they will all be
                    // reconstructed by the sv change
                    remove_edges.extend(
                        graph
                            .edges_directed(self.id, Outgoing)?
                            .map(|edge_ref| edge_ref.triplet()),
                    );

                    // Be sure to delete the root attribute value completely by removing incoming edges to it (it may have
                    // a value subscription edge)
                    if let Some(root_av_id) = graph
                        .outgoing_targets(self.id, EdgeWeightKindDiscriminants::Root)?
                        .next()
                    {
                        remove_edges.extend(
                            graph
                                .edges_directed(root_av_id, Incoming)?
                                .map(|edge_ref| edge_ref.triplet()),
                        );
                    }
                }
                _ => {}
            }
        }

        if component_will_be_deleted {
            // The root attribute value id must be deleted if the component is being deleted
            if let Some(root_av_id) = graph
                .outgoing_targets(self.id, EdgeWeightKindDiscriminants::Root)?
                .next()
            {
                remove_edges.extend(
                    graph
                        .edges_directed(root_av_id, Incoming)?
                        .map(|edge_ref| edge_ref.triplet()),
                );
            }

            // Ensure we delete any incoming edges to the deleted component that might have been
            // added in another change set
            remove_edges.extend(
                graph
                    .edges_directed(self.id, Incoming)?
                    .map(|edge_ref| edge_ref.triplet()),
            );
        }

        for (source_id, kind, target_id) in remove_edges {
            updates.extend(Update::remove_edge_updates(
                graph, source_id, kind, target_id,
            )?);
        }

        Ok(updates)
    }
}
