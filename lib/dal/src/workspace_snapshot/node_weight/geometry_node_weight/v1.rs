use super::NodeWeightDiscriminants;
use crate::workspace_snapshot::{
    content_address::ContentAddress,
    graph::LineageId,
    node_weight::traits::{CorrectExclusiveOutgoingEdge, CorrectTransforms, SiNodeWeight},
    NodeId,
};
use std::collections::HashSet;

use crate::workspace_snapshot::graph::detect_updates::Update;
use crate::workspace_snapshot::node_weight::traits::CorrectTransformsResult;
use crate::{EdgeWeightKindDiscriminants, Timestamp, WorkspaceSnapshotGraphVCurrent};
use dal_macros::SiNodeWeight;
use jwt_simple::prelude::{Deserialize, Serialize};
use petgraph::prelude::EdgeRef;
use petgraph::Incoming;
use si_events::merkle_tree_hash::MerkleTreeHash;
use si_events::ulid::Ulid;
use si_events::ContentHash;

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

impl CorrectTransforms for GeometryNodeWeightV1 {
    fn correct_transforms(
        &self,
        graph: &WorkspaceSnapshotGraphVCurrent,
        mut updates: Vec<Update>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        let mut valid_view: Option<(usize, NodeId)> = None;
        let mut existing_remove_edges = vec![];
        let mut updates_to_remove = vec![];

        let maybe_self_idx = graph.get_node_index_by_id_opt(self.id());

        // Initialize views set with existing view id
        let mut views = HashSet::new();
        {
            if let Some(self_idx) = maybe_self_idx {
                if let Some(view_idx) = graph
                    .edges_directed_for_edge_weight_kind(
                        self_idx,
                        Incoming,
                        EdgeWeightKindDiscriminants::Use,
                    )
                    .pop()
                    .map(|(_, view_node, _)| view_node)
                {
                    if let Some(view_id) = graph.get_node_weight_opt(view_idx).map(|w| w.id()) {
                        views.insert(view_id);
                    }
                }
            }
        }

        for (i, update) in updates.iter().enumerate() {
            match update {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if destination.id.into_inner() == self.id().inner() => {
                    if EdgeWeightKindDiscriminants::from(edge_weight.kind())
                        != EdgeWeightKindDiscriminants::Use
                    {
                        continue;
                    }

                    if source.node_weight_kind == NodeWeightDiscriminants::View {
                        // If we get more than one frame contains edge in the set of
                        // updates we will pick the last one. Although there should
                        // never be more than one in a single batch, this makes it
                        // resilient against replaying multiple transform batches
                        // (in order). Last one wins!
                        views.insert(Ulid::from(source.id.into_inner()));
                        if let Some((last_index, last_id)) = valid_view {
                            views.remove(&Ulid::from(last_id.into_inner()));

                            updates_to_remove.push(last_index);
                        }

                        valid_view = Some((i, source.id));
                    }
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } if destination.id.into_inner() == self.id().inner() => {
                    if *edge_kind == EdgeWeightKindDiscriminants::Use
                        && source.node_weight_kind == NodeWeightDiscriminants::View
                    {
                        if let Some(source_index) =
                            graph.get_node_index_by_id_opt(source.id.into_inner())
                        {
                            views.remove(&Ulid::from(source.id.into_inner()));

                            existing_remove_edges.push(source_index);
                        }
                    }
                }
                _ => {}
            }
        }

        if views.is_empty() {
            // Component will be deleted, delete all outgoing edges
            if let Some(geometry_idx) = maybe_self_idx {
                // Also remove any incoming edges to the geometry in case there
                // is a view edge in another change set
                updates.extend(graph.edges_directed(geometry_idx, Incoming).filter_map(
                    |edge_ref| {
                        graph
                            .get_node_weight_opt(edge_ref.source())
                            .map(|source_weight| Update::RemoveEdge {
                                source: source_weight.into(),
                                destination: self.into(),
                                edge_kind: edge_ref.weight().kind().into(),
                            })
                    },
                ));
            }
        } else {
            if !updates_to_remove.is_empty() {
                let mut idx = 0;
                // Vec::remove is O(n) for the updates, which will likely always be
                // > than the size of updates_to_remove
                updates.retain(|_| {
                    let should_retain = !updates_to_remove.contains(&idx);
                    idx += 1;
                    should_retain
                })
            }

            // Add updates to remove any incoming Use view edges that aren't
            // valid_view. This ensures a geometry can only have one view
            if let Some((_, valid_view_id)) = valid_view {
                if let (Some(valid_source), Some(self_index)) = (
                    graph.get_node_index_by_id_opt(valid_view_id),
                    graph.get_node_index_by_id_opt(self.id()),
                ) {
                    updates.extend(
                        graph
                            .edges_directed(self_index, Incoming)
                            // We only want to find incoming FrameContains edges
                            // that  are not from the current valid source
                            .filter(|edge_ref| {
                                EdgeWeightKindDiscriminants::Use == edge_ref.weight().kind().into()
                                    && edge_ref.source() != valid_source
                                    && !existing_remove_edges.contains(&edge_ref.source())
                            })
                            .filter_map(|edge_ref| {
                                graph
                                    .get_node_weight_opt(edge_ref.source())
                                    .map(|source_weight| Update::RemoveEdge {
                                        source: source_weight.into(),
                                        destination: self.into(),
                                        edge_kind: EdgeWeightKindDiscriminants::Use,
                                    })
                            }),
                    );
                }
            }
        }

        Ok(vec![])
    }
}

impl CorrectExclusiveOutgoingEdge for GeometryNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[EdgeWeightKindDiscriminants::Represents]
    }
}
