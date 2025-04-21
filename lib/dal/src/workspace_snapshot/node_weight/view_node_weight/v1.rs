use std::collections::{HashMap, HashSet};

use crate::{
    Timestamp,
    workspace_snapshot::{
        EdgeWeightKindDiscriminants,
        content_address::ContentAddress,
        graph::{LineageId, WorkspaceSnapshotGraphError, detector::Update},
        node_weight::{
            NodeWeight, NodeWeightDiscriminants,
            traits::{CorrectExclusiveOutgoingEdge, CorrectTransforms, SiNodeWeight},
        },
    },
};

use dal_macros::SiNodeWeight;
use jwt_simple::prelude::{Deserialize, Serialize};
use petgraph::prelude::*;
use si_events::{ContentHash, merkle_tree_hash::MerkleTreeHash, ulid::Ulid};
use si_id::ViewId;

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
        let mut components_with_new_geometry: HashMap<Ulid, HashSet<Ulid>> = HashMap::new();
        let mut new_geometry_for_other_views: HashSet<Ulid> = HashSet::new();

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
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if source.node_weight_kind == NodeWeightDiscriminants::Geometry
                    && EdgeWeightKindDiscriminants::Represents == edge_weight.kind().into() =>
                {
                    components_with_new_geometry
                        .entry(destination.id.into_inner().into())
                        .and_modify(|entry| {
                            entry.insert(source.id.into_inner().into());
                        })
                        .or_insert_with(|| HashSet::from([source.id.into_inner().into()]));
                }
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if source.node_weight_kind == NodeWeightDiscriminants::View
                    && source.id.into_inner() != self.id.inner()
                    && EdgeWeightKindDiscriminants::Use == edge_weight.kind().into()
                    && destination.node_weight_kind == NodeWeightDiscriminants::Geometry =>
                {
                    // There is a new Geometry for a View other than this one.
                    new_geometry_for_other_views.insert(destination.id.into_inner().into());
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
                    if let Ok(Some(represented_thing_idx)) = workspace_snapshot_graph
                        .get_edge_weight_kind_target_idx_opt(
                            destination,
                            Direction::Outgoing,
                            EdgeWeightKindDiscriminants::Represents,
                        )
                    {
                        if let NodeWeight::Component(component) =
                            workspace_snapshot_graph.get_node_weight(represented_thing_idx)?
                        {
                            if removed_components.contains(&component.id().into()) {
                                // If both the View and the Components represented in the View are being
                                // removed, then there won't be individual Update::RemoveEdge for the
                                // Geometry, so we need to check if the Component itself is being removed.
                                continue;
                            }

                            // If the Component isn't being removed, we need to make sure that it still
                            // exists in at least one other View to make sure we're not orphaning it.
                            let mut appears_in_views = workspace_snapshot_graph
                                .component_contained_in_views(component.id().into())?;
                            let my_id: ViewId = self.id().into();
                            appears_in_views.retain(|&view_id| my_id != view_id);

                            // If the Component is either in other views, or is being added to (at least)
                            // another View in the same set of Transforms, then removing this view is fine
                            // since it won't orphan any Components.
                            if !appears_in_views.is_empty() {
                                continue;
                            }
                            // The Component needs to both be getting new Geometry from the set of Update,
                            // _and_ at least one of those Geometry need to be for a View other than the
                            // one being removed.
                            dbg!(&components_with_new_geometry, &new_geometry_for_other_views);
                            if let Some(component_new_geometry_ids) =
                                components_with_new_geometry.get(&component.id())
                            {
                                if component_new_geometry_ids.iter().any(|new_geometry_id| {
                                    new_geometry_for_other_views.contains(new_geometry_id)
                                }) {
                                    continue;
                                }
                            }
                        }

                        updates.remove(view_removal_update_idx);

                        return Ok(updates);
                    }
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
