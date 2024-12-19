use std::collections::HashSet;

use crate::{
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::{detector::Update, LineageId, WorkspaceSnapshotGraphError},
        node_weight::{
            traits::{CorrectExclusiveOutgoingEdge, CorrectTransforms, SiNodeWeight},
            NodeWeight, NodeWeightDiscriminants,
        },
        EdgeWeightKindDiscriminants,
    },
    Timestamp,
};

use dal_macros::SiNodeWeight;
use jwt_simple::prelude::{Deserialize, Serialize};
use petgraph::prelude::*;
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, SiNodeWeight)]
#[si_node_weight(discriminant = NodeWeightDiscriminants::View)]
pub struct ViewNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    #[si_node_weight(node_hash = "self.content_address.content_hash().as_bytes()")]
    content_address: ContentAddress,
    timestamp: Timestamp,
}

impl ViewNodeWeightV1 {
    pub fn new(id: Ulid, lineage_id: Ulid, content_hash: ContentHash) -> Self {
        Self {
            id,
            lineage_id,
            content_address: ContentAddress::View(content_hash),
            merkle_tree_hash: MerkleTreeHash::default(),
            timestamp: Timestamp::now(),
        }
    }

    pub fn new_content_hash(&mut self, new_content_hash: ContentHash) {
        self.content_address = ContentAddress::View(new_content_hash);
    }
}

impl CorrectTransforms for ViewNodeWeightV1 {
    fn correct_transforms(
        &self,
        workspace_snapshot_graph: &crate::WorkspaceSnapshotGraphVCurrent,
        mut updates: Vec<crate::workspace_snapshot::graph::detector::Update>,
        _from_different_change_set: bool,
    ) -> crate::workspace_snapshot::node_weight::traits::CorrectTransformsResult<
        Vec<crate::workspace_snapshot::graph::detector::Update>,
    > {
        let mut maybe_view_removal_update_idx = None;
        let mut removed_geometries = HashSet::new();
        let mut removed_components = HashSet::new();

        for (update_idx, update) in updates.iter().enumerate() {
            match update {
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } if destination.id.into_inner() == self.id.inner()
                    && *edge_kind == EdgeWeightKindDiscriminants::Use
                    && source.node_weight_kind == NodeWeightDiscriminants::Category =>
                {
                    // This view is being removed.
                    maybe_view_removal_update_idx = Some(update_idx);
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } if source.id.into_inner() == self.id.inner()
                    && *edge_kind == EdgeWeightKindDiscriminants::Use
                    && destination.node_weight_kind == NodeWeightDiscriminants::Geometry =>
                {
                    // A Geometry is being removed from this view.
                    removed_geometries.insert(destination.id);
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } if source.node_weight_kind == NodeWeightDiscriminants::Category
                    && destination.node_weight_kind == NodeWeightDiscriminants::Component
                    && *edge_kind == EdgeWeightKindDiscriminants::Use =>
                {
                    // A Component is being removed from the Workspace.
                    removed_components.insert(destination.id);
                }
                _ => {}
            }
        }

        if let Some(view_removal_update_idx) = maybe_view_removal_update_idx {
            let view_node_index = workspace_snapshot_graph.get_node_index_by_id(self.id())?;

            // Make sure that any pre-existing Geometry has a removal in the set of updates.
            for (_edge_weight, _source, destination) in workspace_snapshot_graph
                .edges_directed_for_edge_weight_kind(
                    view_node_index,
                    Direction::Outgoing,
                    EdgeWeightKindDiscriminants::Use,
                )
            {
                let existing_geometry_id =
                    workspace_snapshot_graph
                        .node_index_to_id(destination)
                        .ok_or_else(|| WorkspaceSnapshotGraphError::NodeWeightNotFound)?;

                // Most of the time, the set of Geometry removal updates should be <= the set of
                // pre-existing Geometries, since if the view is being removed, we'll want to also
                // remove all Geometries from the view (=), but there may have also been new
                // Geometries added that we didn't know about when the Updates were calculated (<).
                //
                // We want the one most likely to have the smaller cardinality to be the one we
                // loop over in the inner loop to try to minimize the number of iterations.
                if !removed_geometries.contains(&existing_geometry_id.into()) {
                    let represented_thing_idx = workspace_snapshot_graph
                        .get_edge_weight_kind_target_idx(
                            destination,
                            Direction::Outgoing,
                            EdgeWeightKindDiscriminants::Represents,
                        )?;
                    if let NodeWeight::Component(component) =
                        workspace_snapshot_graph.get_node_weight(represented_thing_idx)?
                    {
                        if removed_components.contains(&component.id().into()) {
                            // If both the View and the Components represented in the View are being
                            // removed, then there won't be individual Update::RemoveEdge for the
                            // Geometry, so we need to check if the Component itself is being removed.
                            continue;
                        }
                    }

                    updates.remove(view_removal_update_idx);

                    return Ok(updates);
                }
            }
        }

        Ok(updates)
    }
}

impl CorrectExclusiveOutgoingEdge for ViewNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}
