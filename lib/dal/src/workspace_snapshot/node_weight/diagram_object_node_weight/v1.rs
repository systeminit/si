use std::collections::BTreeMap;

use dal_macros::SiNodeWeight;
use petgraph::Direction::Outgoing;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};

use crate::{
    EdgeWeight,
    EdgeWeightKindDiscriminants,
    workspace_snapshot::{
        graph::{
            LineageId,
            detector::Update,
        },
        node_weight::{
            NodeWeight,
            NodeWeightDiscriminants,
            diagram_object_node_weight::DiagramObjectKind,
            traits::{
                CorrectExclusiveOutgoingEdge,
                CorrectTransforms,
                CorrectTransformsError,
                CorrectTransformsResult,
                SiNodeWeight,
            },
        },
        split_snapshot,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, SiNodeWeight, Hash)]
#[si_node_weight(discriminant = NodeWeightDiscriminants::Geometry)]
pub struct DiagramObjectNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    #[si_node_weight(node_hash = "self.object_kind.to_string().as_bytes()")]
    object_kind: DiagramObjectKind,
}

impl DiagramObjectNodeWeightV1 {
    pub fn new(id: Ulid, lineage_id: Ulid, object_kind: DiagramObjectKind) -> Self {
        Self {
            id,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
            object_kind,
        }
    }

    pub fn object_kind(&self) -> DiagramObjectKind {
        self.object_kind
    }
}

impl CorrectTransforms for DiagramObjectNodeWeightV1 {
    fn correct_transforms(
        &self,
        workspace_snapshot_graph: &crate::WorkspaceSnapshotGraphVCurrent,
        mut updates: Vec<Update>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        let mut maybe_diagram_object_creation_idx = None;
        let mut maybe_diagram_object_incoming_view_edge_idx = None;
        let mut maybe_diagram_object_view_id: Option<Ulid> = None;
        let mut maybe_diagram_object_incoming_category_edge_idx = None;
        let mut maybe_diagram_object_incoming_geometry_edge_idx = None;

        // We're searching for a new node update, so if the node is already
        // in the graph, there is nothing to correct
        if workspace_snapshot_graph
            .get_node_index_by_id_opt(self.id)
            .is_some()
        {
            return Ok(updates);
        }

        for (update_idx, update) in updates.iter().enumerate() {
            match update {
                // View --DiagramObject--> DiagramObject
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if self.id == destination.id.into()
                    && source.node_weight_kind == NodeWeightDiscriminants::View
                    && EdgeWeightKindDiscriminants::DiagramObject == edge_weight.kind().into() =>
                {
                    maybe_diagram_object_incoming_view_edge_idx = Some(update_idx);
                    maybe_diagram_object_view_id = Some(source.id.into());
                }
                // Category --Use--> DiagramObject
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if self.id == destination.id.into()
                    && source.node_weight_kind == NodeWeightDiscriminants::Category
                    && EdgeWeightKindDiscriminants::Use == edge_weight.kind().into() =>
                {
                    maybe_diagram_object_incoming_category_edge_idx = Some(update_idx);
                }
                // Geometry --Represents--> DiagramObject
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if self.id == destination.id.into()
                    && source.node_weight_kind == NodeWeightDiscriminants::Geometry
                    && EdgeWeightKindDiscriminants::Represents == edge_weight.kind().into() =>
                {
                    maybe_diagram_object_incoming_geometry_edge_idx = Some(update_idx);
                }
                Update::NewNode { node_weight } if node_weight.id() == self.id => {
                    maybe_diagram_object_creation_idx = Some(update_idx);
                }
                _ => {}
            }
        }

        // If we're creating a DiagramObject for a View, then all of these pieces of information
        // _must_ exist in the updates as well.
        if let (
            Some(diagram_object_creation_idx),
            Some(diagram_object_incoming_view_edge_idx),
            Some(diagram_object_view_id),
            Some(diagram_object_incoming_category_edge_idx),
            Some(diagram_object_incoming_geometry_edge_idx),
        ) = (
            maybe_diagram_object_creation_idx,
            maybe_diagram_object_incoming_view_edge_idx,
            maybe_diagram_object_view_id,
            maybe_diagram_object_incoming_category_edge_idx,
            maybe_diagram_object_incoming_geometry_edge_idx,
        ) {
            // If the associated View doesn't already exist, then it can't possibly already have a
            // DiagramObject that we need to merge this update into.
            if let Some(pre_existing_view_node_index) =
                workspace_snapshot_graph.get_node_index_by_id_opt(diagram_object_view_id)
            {
                let mut maybe_pre_existing_view_diagram_object_node_index = None;
                for (_edge_weight, _source_node_index, diagram_object_node_index) in
                    workspace_snapshot_graph.edges_directed_for_edge_weight_kind(
                        pre_existing_view_node_index,
                        petgraph::Direction::Outgoing,
                        EdgeWeightKindDiscriminants::DiagramObject,
                    )
                {
                    maybe_pre_existing_view_diagram_object_node_index =
                        Some(diagram_object_node_index);
                }

                // Even if the associated View does already exist, it might not already have a
                // `DiagramObject` to merge this update into.
                if let Some(pre_existing_view_diagram_object_node_index) =
                    maybe_pre_existing_view_diagram_object_node_index
                {
                    let pre_existing_view_diagram_object_node_weight = workspace_snapshot_graph
                        .get_node_weight(pre_existing_view_diagram_object_node_index)?;

                    // Modify the update associating the new Geometry to this DiagramObject to
                    // associate it with the already existing DiagramObject, instead.
                    match updates.get_mut(diagram_object_incoming_geometry_edge_idx) {
                        Some(Update::NewEdge { destination, .. }) => {
                            destination.id =
                                pre_existing_view_diagram_object_node_weight.id().into();
                        }
                        // If there, somehow, isn't an entry at that idx, or it is not a
                        // Update::NewEdge, then something has gone horribly wrong, as we got the
                        // idx by iterating over the updates, and looking at what kind of thing was
                        // there.
                        _ => return Err(CorrectTransformsError::InvalidUpdates("Updates list is no longer what is expected. Expected Update::NewEdge at index, but element is either missing, or not the expected variant".to_string())),
                    }

                    // Now that the new Geometry is associated with the already existing
                    // DiagramObject in the updates, we want to get rid of the new DiagramObject
                    // update, and the edges to it from the Category & View. Removing them in
                    // reverse index order means we don't need to account for entries shifting
                    // around in the Vec.
                    let mut updates_to_remove_idxs = vec![
                        diagram_object_creation_idx,
                        diagram_object_incoming_category_edge_idx,
                        diagram_object_incoming_view_edge_idx,
                    ];
                    updates_to_remove_idxs.sort();
                    updates_to_remove_idxs.reverse();

                    for updates_idx_to_remove in updates_to_remove_idxs {
                        updates.remove(updates_idx_to_remove);
                    }
                }
            }
        }

        Ok(updates)
    }
}

impl CorrectExclusiveOutgoingEdge for DiagramObjectNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl
    split_snapshot::corrections::CorrectTransforms<
        NodeWeight,
        EdgeWeight,
        EdgeWeightKindDiscriminants,
    > for DiagramObjectNodeWeightV1
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
        // These are corrections that apply to *new* diagram objects,
        // so we can short circuit if the dobj is on the graph already
        if graph.node_exists(self.id) {
            return Ok(updates);
        }

        let mut update_idxs_to_remove = vec![];
        let mut maybe_diagram_object_containing_view_id = None;
        let mut maybe_diagram_object_subgraph_root_id = None;
        let mut maybe_geometry_source_id = None;
        let mut maybe_edge_from_geometry_to_diagram_object = None;
        let mut geometry_id_to_subgraph_root_id = BTreeMap::new();

        for (update_idx, update) in updates.iter().enumerate() {
            match update {
                // View --DiagramObject--> DiagramObject
                si_split_graph::Update::NewEdge { source, .. }
                    if update.destination_has_id(self.id)
                        && update.is_edge_of_sort(
                            NodeWeightDiscriminants::View,
                            EdgeWeightKindDiscriminants::DiagramObject,
                            NodeWeightDiscriminants::DiagramObject,
                        ) =>
                {
                    update_idxs_to_remove.push(update_idx);
                    if source.custom_kind.is_some() {
                        // If this is a net new view, we do not need to correct,
                        // *SHORT CIRCUIT* here before continuing to process updates
                        if !graph.node_exists(source.id) {
                            return Ok(updates);
                        }
                        maybe_diagram_object_containing_view_id = Some(source.id);
                    }
                }
                // Category --Use--> DiagramObject
                si_split_graph::Update::NewEdge { .. }
                    if update.destination_has_id(self.id)
                        && update.is_edge_of_sort(
                            NodeWeightDiscriminants::Category,
                            EdgeWeightKindDiscriminants::Use,
                            NodeWeightDiscriminants::DiagramObject,
                        ) =>
                {
                    update_idxs_to_remove.push(update_idx);
                }
                // Geometry --Represents--> DiagramObject
                si_split_graph::Update::NewEdge {
                    source,
                    edge_weight,
                    ..
                } if update.destination_has_id(self.id)
                    && update.is_edge_of_sort(
                        NodeWeightDiscriminants::Geometry,
                        EdgeWeightKindDiscriminants::Represents,
                        NodeWeightDiscriminants::DiagramObject,
                    ) =>
                {
                    update_idxs_to_remove.push(update_idx);
                    if let Some(edge_weight) = edge_weight.custom() {
                        maybe_edge_from_geometry_to_diagram_object = Some(edge_weight.clone());
                    }
                    if source.custom_kind.is_some() {
                        maybe_geometry_source_id = Some(source.id);
                    }
                }
                // We need to know the subgraph of the geometry node so we can produce the correct
                // cross graph new-edge updates for it
                si_split_graph::Update::NewNode {
                    subgraph_root_id,
                    node_weight,
                } if node_weight.custom_kind() == Some(NodeWeightDiscriminants::Geometry) => {
                    geometry_id_to_subgraph_root_id.insert(node_weight.id(), *subgraph_root_id);
                }
                si_split_graph::Update::NewNode {
                    subgraph_root_id,
                    node_weight,
                } if node_weight.id() == self.id => {
                    update_idxs_to_remove.push(update_idx);
                    maybe_diagram_object_subgraph_root_id = Some(*subgraph_root_id);
                }
                _ => {}
            }
        }

        let (
            Some(diagram_object_containing_view_id),
            Some(diagram_object_subgraph_root_id),
            Some(geometry_source_id),
            Some(edge_from_geometry_to_diagram_object),
        ) = (
            maybe_diagram_object_containing_view_id,
            maybe_diagram_object_subgraph_root_id,
            maybe_geometry_source_id,
            maybe_edge_from_geometry_to_diagram_object,
        )
        else {
            return Ok(updates);
        };

        let geometry_subgraph_root_id = match geometry_id_to_subgraph_root_id
            .get(&geometry_source_id)
        {
            Some(subgraph_root_id) => *subgraph_root_id,
            None => {
                // If there wasn't a new node update, then maybe somehow the node is already in the graph?
                // unlikely, but possible
                let Some(subgraph_root_id) = graph.subgraph_root_id_for_node(geometry_source_id)
                else {
                    return Ok(updates);
                };

                subgraph_root_id
            }
        };

        let Some(existing_diagram_object_for_view) = graph
            .edges_directed_for_edge_weight_kind(
                diagram_object_containing_view_id,
                Outgoing,
                EdgeWeightKindDiscriminants::DiagramObject,
            )?
            .last()
            .and_then(|edge_ref| graph.node_weight(edge_ref.target()))
        else {
            return Ok(updates);
        };

        for idx_to_remove in update_idxs_to_remove {
            updates.remove(idx_to_remove);
        }

        let graph_root_id = graph.root_id()?;

        updates.extend(si_split_graph::Update::new_edge_between_nodes_updates(
            geometry_source_id,
            geometry_subgraph_root_id,
            NodeWeightDiscriminants::Geometry,
            existing_diagram_object_for_view.id(),
            diagram_object_subgraph_root_id,
            NodeWeightDiscriminants::DiagramObject,
            edge_from_geometry_to_diagram_object,
            graph_root_id,
        ));

        Ok(updates)
    }
}
