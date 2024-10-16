use super::NodeWeightDiscriminants;
use crate::workspace_snapshot::{
    content_address::ContentAddress,
    graph::LineageId,
    node_weight::traits::{CorrectExclusiveOutgoingEdge, CorrectTransforms, SiNodeWeight},
};

use crate::{EdgeWeightKindDiscriminants, Timestamp};
use dal_macros::SiNodeWeight;
use jwt_simple::prelude::{Deserialize, Serialize};
use si_events::merkle_tree_hash::MerkleTreeHash;
use si_events::ulid::Ulid;
use si_events::ContentHash;

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

impl CorrectTransforms for ViewNodeWeightV1 {}

impl CorrectExclusiveOutgoingEdge for ViewNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}
