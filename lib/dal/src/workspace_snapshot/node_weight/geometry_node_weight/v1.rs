use super::{GeometryNodeWeight, NodeWeightDiscriminants};
use crate::workspace_snapshot::graph::detector::Update;
use crate::workspace_snapshot::node_weight::diagram_object_node_weight::DiagramObjectKind;
use crate::workspace_snapshot::node_weight::traits::{
    CorrectTransformsError, CorrectTransformsResult,
};
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::{
    content_address::ContentAddress,
    graph::LineageId,
    node_weight::traits::{CorrectExclusiveOutgoingEdge, CorrectTransforms, SiNodeWeight},
};
use crate::{EdgeWeightKindDiscriminants, Timestamp, WorkspaceSnapshotGraphVCurrent};
use dal_macros::SiNodeWeight;
use jwt_simple::prelude::{Deserialize, Serialize};
use petgraph::Direction;
use si_events::merkle_tree_hash::MerkleTreeHash;
use si_events::ulid::Ulid;
use si_events::ContentHash;
use std::collections::HashMap;
use std::mem;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, SiNodeWeight)]
#[si_node_weight(discriminant = NodeWeightDiscriminants::Geometry)]
pub struct GeometryNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    #[si_node_weight(node_hash = "self.content_address.content_hash().as_bytes()")]
    content_address: ContentAddress,
    timestamp: Timestamp,
}

impl GeometryNodeWeightV1 {
    pub fn new(id: Ulid, lineage_id: Ulid, content_hash: ContentHash) -> Self {
        Self {
            id,
            lineage_id,
            content_address: ContentAddress::Geometry(content_hash),
            merkle_tree_hash: MerkleTreeHash::default(),
            timestamp: Timestamp::now(),
        }
    }

    pub fn new_content_hash(&mut self, new_content_hash: ContentHash) {
        self.content_address = ContentAddress::Geometry(new_content_hash);
    }
}

// IF this geometry is for a DO and its on a view that already has a geometry for this DO
// replace the original geometries content for the content of the new one, and erase other updates for self

// replace new node update with a replaceNode update with the new content
impl CorrectTransforms for GeometryNodeWeightV1 {
    fn correct_transforms(
        &self,
        workspace_snapshot_graph: &WorkspaceSnapshotGraphVCurrent,
        mut updates: Vec<Update>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        let mut maybe_self_creation = None;
        let mut self_updates = vec![];

        let mut maybe_diagram_object_for_self = None;
        let mut maybe_container_view_for_self = None;

        let mut view_id_for_diagram_object = HashMap::new();

        for (update_idx, update) in updates.iter().enumerate() {
            match update {
                Update::NewNode { node_weight } if node_weight.id() == self.id => {
                    maybe_self_creation = Some(update_idx);
                }
                Update::NewNode { node_weight }
                    if NodeWeightDiscriminants::DiagramObject == node_weight.into() =>
                {
                    match node_weight.get_diagram_object_weight()?.object_kind() {
                        DiagramObjectKind::View(view_id) => {
                            view_id_for_diagram_object.insert(node_weight.id(), view_id);
                        }
                    }
                }
                // Geometry --Represents-> DiagramObject
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if EdgeWeightKindDiscriminants::Represents == edge_weight.kind().into()
                    && source.node_weight_kind == NodeWeightDiscriminants::Geometry
                    && destination.node_weight_kind == NodeWeightDiscriminants::DiagramObject =>
                {
                    if source.id == self.id.into() {
                        self_updates.push(update_idx);
                        maybe_diagram_object_for_self = Some(destination.id);
                    }
                }
                // View --Use-> Geometry
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if EdgeWeightKindDiscriminants::Use == edge_weight.kind().into()
                    && source.node_weight_kind == NodeWeightDiscriminants::View
                    && destination.node_weight_kind == NodeWeightDiscriminants::Geometry =>
                {
                    if destination.id == self.id.into() {
                        self_updates.push(update_idx);
                        maybe_container_view_for_self = Some(source.id);
                    }
                }
                // Store all new edge updates to self, in case we need to delete them
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight: _edge_weight,
                } if source.id == self.id.into() || destination.id == self.id.into() => {
                    self_updates.push(update_idx);
                }

                _ => {}
            }
        }

        let Some(self_creation_idx) = maybe_self_creation else {
            return Ok(updates);
        };

        let Some(container_view_for_self) = maybe_container_view_for_self else {
            return Err(CorrectTransformsError::InvalidUpdates(
                "Trying to create a geometry without a container view".to_string(),
            ));
        };

        // If geometry is not for diagram object, there's no need to run this check
        let Some(diagram_object_for_self) = maybe_diagram_object_for_self else {
            return Ok(updates);
        };

        let view_id_for_self = match view_id_for_diagram_object.get(&diagram_object_for_self.into())
        {
            Some(view_id) => *view_id,
            None => {
                let Some(do_idx) =
                    workspace_snapshot_graph.get_node_index_by_id_opt(diagram_object_for_self)
                else {
                    return Err(CorrectTransformsError::InvalidUpdates(
                        "Diagram object should exist on graph".to_string(),
                    ));
                };

                let diagram_object_node = workspace_snapshot_graph
                    .get_node_weight(do_idx)?
                    .get_diagram_object_weight()?;

                match diagram_object_node.object_kind() {
                    DiagramObjectKind::View(id) => id,
                }
            }
        };

        let mut maybe_geometry_to_replace = None;

        // Go through the graph to see if self is a duplicate of existing geometries
        // Start from the view self represents
        if let Some(view_idx) = workspace_snapshot_graph.get_node_index_by_id_opt(view_id_for_self)
        {
            // if it has a DO already on the graph
            if let Some((_, _, do_idx)) = workspace_snapshot_graph
                .edges_directed_for_edge_weight_kind(
                    view_idx,
                    Direction::Outgoing,
                    EdgeWeightKindDiscriminants::DiagramObject,
                )
                .pop()
            {
                // loop geometries of the DO. If any of them is contained by the self container, replace it.
                for (_, geo_idx, _) in workspace_snapshot_graph.edges_directed_for_edge_weight_kind(
                    do_idx,
                    Direction::Incoming,
                    EdgeWeightKindDiscriminants::Represents,
                ) {
                    let Some((_, container_view_idx, _)) = workspace_snapshot_graph
                        .edges_directed_for_edge_weight_kind(
                            geo_idx,
                            Direction::Incoming,
                            EdgeWeightKindDiscriminants::Use,
                        )
                        .pop()
                    else {
                        continue;
                    };

                    let container_view_node =
                        workspace_snapshot_graph.get_node_weight(container_view_idx)?;

                    if container_view_node.id() == container_view_for_self.into() {
                        let this_geo_node = workspace_snapshot_graph.get_node_weight(geo_idx)?;

                        maybe_geometry_to_replace = Some(this_geo_node.clone());
                    }
                }
            }
        }

        // If geo should not be created, loop through updates again and remove updates related to this node
        if let Some(geometry_to_replace) = maybe_geometry_to_replace {
            let id = geometry_to_replace.id();
            let lineage = geometry_to_replace.lineage_id();
            let content = self.content_address.content_hash();

            let replaced_weight = NodeWeight::Geometry(GeometryNodeWeight::V1(
                GeometryNodeWeightV1::new(id, lineage, content),
            ));

            let replaced_update = Update::ReplaceNode {
                node_weight: replaced_weight,
            };

            // Modify the update by replacing the old duplicate geometry with the new one,
            // instead of creating the new one
            match updates.get_mut(self_creation_idx) {
                Some(update @ Update::NewNode { .. }) => {
                    let _old_update = mem::replace(update, replaced_update);
                }
                // If there, somehow, isn't an entry at that idx, or it is not a
                // Update::NewNode, then something has gone horribly wrong, as we got the
                // idx by iterating over the updates, and looking at what kind of thing was
                // there.
                _ => return Err(CorrectTransformsError::InvalidUpdates("Updates list is no longer what is expected. Expected Update::NewNode at index, but element is either missing, or not the expected variant".to_string())),
            }

            self_updates.sort();
            self_updates.reverse();
            for update_idx in self_updates {
                updates.remove(update_idx);
            }
        }

        Ok(updates)
    }
}

impl CorrectExclusiveOutgoingEdge for GeometryNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[EdgeWeightKindDiscriminants::Represents]
    }
}
