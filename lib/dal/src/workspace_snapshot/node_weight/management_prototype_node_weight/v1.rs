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
    NodeWeightDiscriminants,
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        node_weight::traits::{
            CorrectExclusiveOutgoingEdge,
            CorrectTransforms,
            ExclusiveOutgoingEdges,
            SiNodeWeight,
        },
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ManagementPrototypeNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    content_address: ContentAddress,
    timestamp: Timestamp,
}

impl ManagementPrototypeNodeWeightV1 {
    pub fn new(id: Ulid, lineage_id: Ulid, content_hash: ContentHash) -> Self {
        Self {
            id,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
            content_address: ContentAddress::ManagementPrototype(content_hash),
            timestamp: Timestamp::now(),
        }
    }

    pub fn new_content_hash(&mut self, new_content_hash: ContentHash) {
        self.content_address = ContentAddress::ManagementPrototype(new_content_hash);
    }
}

impl SiNodeWeight for ManagementPrototypeNodeWeightV1 {
    fn content_hash(&self) -> ContentHash {
        self.content_address.content_hash()
    }

    fn id(&self) -> Ulid {
        self.id
    }

    fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(self.content_address.content_hash().as_bytes());

        content_hasher.finalize()
    }

    fn node_weight_discriminant(&self) -> NodeWeightDiscriminants {
        NodeWeightDiscriminants::ManagementPrototype
    }

    fn set_id(&mut self, new_id: Ulid) {
        self.id = new_id;
    }

    fn set_lineage_id(&mut self, new_lineage_id: Ulid) {
        self.lineage_id = new_lineage_id;
    }

    fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash
    }
}

impl CorrectTransforms for ManagementPrototypeNodeWeightV1 {}
impl CorrectExclusiveOutgoingEdge for ManagementPrototypeNodeWeightV1 {}
impl ExclusiveOutgoingEdges for ManagementPrototypeNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[crate::EdgeWeightKindDiscriminants] {
        &[]
    }
}
