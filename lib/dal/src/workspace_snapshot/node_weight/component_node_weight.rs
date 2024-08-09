use petgraph::{visit::EdgeRef, Direction::Incoming};
use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{
    workspace_snapshot::{
        content_address::{ContentAddress, ContentAddressDiscriminants},
        graph::{
            deprecated::v1::DeprecatedComponentNodeWeightV1, detect_updates::Update, LineageId,
        },
        node_weight::traits::CorrectTransforms,
        NodeInformation,
    },
    EdgeWeightKindDiscriminants, WorkspaceSnapshotGraphV2,
};

use super::{
    traits::CorrectTransformsResult, NodeWeightDiscriminants::Component, NodeWeightError,
    NodeWeightResult,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    merkle_tree_hash: MerkleTreeHash,
    to_delete: bool,
}

impl ComponentNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, content_address: ContentAddress) -> Self {
        Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            to_delete: false,
        }
    }

    pub fn content_address(&self) -> ContentAddress {
        self.content_address
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![self.content_address.content_hash()]
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_address.content_hash()
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn to_delete(&self) -> bool {
        self.to_delete
    }

    pub fn set_to_delete(&mut self, to_delete: bool) -> &mut Self {
        self.to_delete = to_delete;
        self
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::Component(_) => ContentAddress::Component(content_hash),
            other => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    Into::<ContentAddressDiscriminants>::into(other).to_string(),
                    ContentAddressDiscriminants::Component.to_string(),
                ));
            }
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "content_address": self.content_address,
            "to_delete": self.to_delete,
        }])
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn overwrite_id(&mut self, id: Ulid) {
        self.id = id
    }

    pub fn overwrite_lineage_id(&mut self, id: LineageId) {
        self.lineage_id = id
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[
            EdgeWeightKindDiscriminants::Use,
            EdgeWeightKindDiscriminants::Root,
        ]
    }
}

impl From<DeprecatedComponentNodeWeightV1> for ComponentNodeWeight {
    fn from(value: DeprecatedComponentNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            content_address: value.content_address,
            merkle_tree_hash: value.merkle_tree_hash,
            to_delete: value.to_delete,
        }
    }
}

impl From<&ComponentNodeWeight> for NodeInformation {
    fn from(value: &ComponentNodeWeight) -> Self {
        Self {
            node_weight_kind: Component,
            id: value.id.into(),
        }
    }
}

impl CorrectTransforms for ComponentNodeWeight {
    fn correct_transforms(
        &self,
        graph: &WorkspaceSnapshotGraphV2,
        mut updates: Vec<Update>,
    ) -> CorrectTransformsResult<Vec<Update>> {
        let mut valid_frame_contains_source = None;
        let mut existing_remove_edges = vec![];
        let mut updates_to_remove = vec![];

        for (i, update) in updates.iter().enumerate() {
            match update {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } => {
                    // If we get more than one frame contains edge in the set of
                    // updates we will pick the last one. Although there should
                    // never be more than one in a single batch, this makes it
                    // resilient against replaying multiple transform batches
                    // (in order). Last one wins!
                    if destination.id.into_inner() == self.id.inner()
                        && EdgeWeightKindDiscriminants::FrameContains == edge_weight.kind().into()
                    {
                        valid_frame_contains_source = match valid_frame_contains_source {
                            None => Some((i, source.id)),
                            Some((last_index, _)) => {
                                updates_to_remove.push(last_index);
                                Some((i, source.id))
                            }
                        }
                    }
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } => {
                    if edge_kind == &EdgeWeightKindDiscriminants::FrameContains
                        && destination.id.into_inner() == self.id.inner()
                    {
                        if let Some(source_index) =
                            graph.get_node_index_by_id_opt(source.id.into_inner())
                        {
                            existing_remove_edges.push(source_index);
                        }
                    }
                }
                _ => {}
            }
        }

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

        // Add updates to remove any incoming FrameContains edges that don't
        // have the source in valid_frame_contains_source. This ensures a
        // component can only have one frame parent
        if let Some((_, valid_frame_contains_source)) = valid_frame_contains_source {
            if let (Some(valid_source), Some(self_index)) = (
                graph.get_node_index_by_id_opt(valid_frame_contains_source),
                graph.get_node_index_by_id_opt(self.id),
            ) {
                updates.extend(
                    graph
                        .edges_directed(self_index, Incoming)
                        // We only want to find incoming FrameContains edges
                        // that  are not from the current valid source
                        .filter(|edge_ref| {
                            EdgeWeightKindDiscriminants::FrameContains
                                == edge_ref.weight().kind().into()
                                && edge_ref.source() != valid_source
                                && !existing_remove_edges.contains(&edge_ref.source())
                        })
                        .filter_map(|edge_ref| {
                            graph
                                .get_node_weight_opt(edge_ref.source())
                                .map(|source_weight| Update::RemoveEdge {
                                    source: source_weight.into(),
                                    destination: self.into(),
                                    edge_kind: EdgeWeightKindDiscriminants::FrameContains,
                                })
                        }),
                );
            }
        }

        Ok(updates)
    }
}
