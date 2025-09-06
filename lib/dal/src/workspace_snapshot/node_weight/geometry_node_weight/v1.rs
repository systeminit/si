use std::{
    collections::{
        BTreeMap,
        HashMap,
    },
    mem,
};

use dal_macros::SiNodeWeight;
use jwt_simple::prelude::{
    Deserialize,
    Serialize,
};
use petgraph::Direction::{
    self,
    Incoming,
    Outgoing,
};
use si_events::{
    ContentHash,
    Timestamp,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};

use super::{
    GeometryNodeWeight,
    NodeWeightDiscriminants,
};
use crate::{
    EdgeWeight,
    EdgeWeightKindDiscriminants,
    WorkspaceSnapshotGraphVCurrent,
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::{
            LineageId,
            detector::Update,
        },
        node_weight::{
            NodeWeight,
            diagram_object_node_weight::DiagramObjectKind,
            traits::{
                CorrectExclusiveOutgoingEdge,
                CorrectTransforms,
                CorrectTransformsError,
                CorrectTransformsResult,
                ExclusiveOutgoingEdges,
                SiNodeWeight,
            },
        },
        split_snapshot,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, SiNodeWeight, Hash)]
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

        // If the geometry node is already there, there's nothing to do
        if workspace_snapshot_graph
            .get_node_index_by_id_opt(self.id)
            .is_some()
        {
            return Ok(updates);
        }

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
                .last()
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
                        .last()
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

impl CorrectExclusiveOutgoingEdge for GeometryNodeWeightV1 {}

impl ExclusiveOutgoingEdges for GeometryNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[EdgeWeightKindDiscriminants::Represents]
    }
}

impl
    split_snapshot::corrections::CorrectTransforms<
        NodeWeight,
        EdgeWeight,
        EdgeWeightKindDiscriminants,
    > for GeometryNodeWeightV1
{
    fn correct_transforms(
        &self,
        graph: &si_split_graph::SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
        mut updates: Vec<
            si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
        >,
        _from_different_change_set: bool,
    ) -> split_snapshot::corrections::CorrectTransformsResult<
        Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
    > {
        // This correction only applies to new geometry nodes, so if this node already
        // exists in the graph, we can skip it
        if graph.node_exists(self.id) {
            return Ok(updates);
        }

        let mut view_id_for_diagram_object = BTreeMap::new();
        let mut maybe_diagram_object_id_for_self = None;
        let mut maybe_container_view_id_for_self = None;

        for update in &updates {
            match update {
                si_split_graph::Update::NewNode { node_weight, .. } => {
                    if let Some(NodeWeight::DiagramObject(dobj)) = node_weight.custom() {
                        match dobj.object_kind() {
                            DiagramObjectKind::View(view_id) => {
                                view_id_for_diagram_object.insert(node_weight.id(), view_id);
                            }
                        }
                    }
                }
                // Geometry --Represents--> DiagramObject
                si_split_graph::Update::NewEdge { destination, .. }
                    if update.is_edge_of_sort(
                        NodeWeightDiscriminants::Geometry,
                        EdgeWeightKindDiscriminants::Represents,
                        NodeWeightDiscriminants::DiagramObject,
                    ) && update.source_has_id(self.id)
                        && destination.custom_kind.is_some() =>
                {
                    // We know this is an edge from a geometry to a diagram object,
                    // but we want to store just the id of the real diagram object, not
                    // any possible external target node. If custom_kind is some, then
                    // it can't be an external target node, thus it must be a diagram object.
                    maybe_diagram_object_id_for_self = Some(destination.id);
                }
                // View --Use--> Geometry
                si_split_graph::Update::NewEdge { source, .. }
                    if update.is_edge_of_sort(
                        NodeWeightDiscriminants::View,
                        EdgeWeightKindDiscriminants::Use,
                        NodeWeightDiscriminants::Geometry,
                    ) && update.destination_has_id(self.id)
                        && source.custom_kind.is_some() =>
                {
                    maybe_container_view_id_for_self = Some(source.id);
                }
                _ => {}
            }
        }

        let Some(container_view_for_self) = maybe_container_view_id_for_self else {
            return Ok(updates);
        };

        let Some(diagram_object_id_for_self) = maybe_diagram_object_id_for_self else {
            return Ok(updates);
        };

        let view_id_for_self = match view_id_for_diagram_object.get(&diagram_object_id_for_self) {
            Some(view_id) => *view_id,
            None => {
                let Some(NodeWeight::DiagramObject(dobj_inner)) =
                    graph.node_weight(diagram_object_id_for_self)
                else {
                    return Ok(updates);
                };

                match dobj_inner.object_kind() {
                    DiagramObjectKind::View(view_id) => view_id,
                }
            }
        };

        // If the view is not already on the graph, there's nothing to do
        if !graph.node_exists(view_id_for_self.into()) {
            return Ok(updates);
        }

        let Some(existing_dobj_id) = graph
            .edges_directed_for_edge_weight_kind(
                view_id_for_self.into(),
                Outgoing,
                EdgeWeightKindDiscriminants::DiagramObject,
            )?
            .last()
            .map(|edge_ref| edge_ref.target())
        else {
            return Ok(updates);
        };

        let mut maybe_existing_geometry_node = None;

        for incoming_represents_edge_ref in graph.edges_directed_for_edge_weight_kind(
            existing_dobj_id,
            Incoming,
            EdgeWeightKindDiscriminants::Represents,
        )? {
            let geometry_id = incoming_represents_edge_ref.source();
            let Some(container_view_id) = graph
                .edges_directed_for_edge_weight_kind(
                    geometry_id,
                    Incoming,
                    EdgeWeightKindDiscriminants::Use,
                )?
                .last()
                .map(|edge_ref| edge_ref.source())
            else {
                continue;
            };

            if container_view_id == container_view_for_self {
                if let Some(NodeWeight::Geometry(geo_inner)) = graph.node_weight(geometry_id) {
                    maybe_existing_geometry_node = Some(geo_inner);
                    break;
                }
            }
        }

        // If we found a conflicting geometry node, then just add a remove node update to remove it.
        // The "newest" geometry node should win.
        if let Some(existing_geometry_node) = maybe_existing_geometry_node {
            // Append the replacenode update to the end of the update list
            if let Some(subgraph_root_id) =
                graph.subgraph_root_id_for_node(existing_geometry_node.id())
            {
                updates.push(si_split_graph::Update::RemoveNode {
                    subgraph_root_id,
                    id: existing_geometry_node.id(),
                });
            }
        }

        Ok(updates)
    }
}
