use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    Timestamp,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};

use crate::{
    EdgeWeightKindDiscriminants,
    SocketArity,
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        node_weight::{
            NodeWeightDiscriminants,
            traits::{
                CorrectExclusiveOutgoingEdge,
                CorrectTransforms,
                ExclusiveOutgoingEdges,
                SiNodeWeight,
            },
        },
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiNodeWeight, Hash)]
#[si_node_weight(discriminant = NodeWeightDiscriminants::InputSocket)]
pub struct InputSocketNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    #[si_node_weight(node_hash = "self.arity.to_string().as_bytes()")]
    arity: SocketArity,
    #[si_node_weight(node_hash = "self.content_address.content_hash().as_bytes()")]
    content_address: ContentAddress,
    timestamp: Timestamp,
}

impl InputSocketNodeWeightV1 {
    pub fn arity(&self) -> SocketArity {
        self.arity
    }

    pub fn new(id: Ulid, lineage_id: Ulid, arity: SocketArity, content_hash: ContentHash) -> Self {
        Self {
            id,
            lineage_id,
            arity,
            content_address: ContentAddress::InputSocket(content_hash),
            merkle_tree_hash: MerkleTreeHash::default(),
            timestamp: Timestamp::now(),
        }
    }

    pub fn new_content_hash(&mut self, new_content_hash: ContentHash) {
        self.content_address = ContentAddress::InputSocket(new_content_hash);
    }
}

impl CorrectTransforms for InputSocketNodeWeightV1 {}

impl CorrectExclusiveOutgoingEdge for InputSocketNodeWeightV1 {}

impl ExclusiveOutgoingEdges for InputSocketNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}
