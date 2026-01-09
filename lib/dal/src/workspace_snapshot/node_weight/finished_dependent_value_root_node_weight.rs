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
    EdgeWeightKindDiscriminants,
    workspace_snapshot::node_weight::traits::CorrectTransforms,
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FinishedDependentValueRootNodeWeight {
    pub id: Ulid,
    pub lineage_id: Ulid,
    value_id: Ulid,
    merkle_tree_hash: MerkleTreeHash,
}

impl FinishedDependentValueRootNodeWeight {
    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![]
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn value_id(&self) -> Ulid {
        self.value_id
    }

    pub fn set_value_id(&mut self, value_id: Ulid) {
        self.value_id = value_id;
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn new(id: Ulid, lineage_id: Ulid, value_id: Ulid) -> Self {
        Self {
            id,
            lineage_id,
            value_id,
            merkle_tree_hash: Default::default(),
        }
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(&self.value_id.inner().to_bytes());

        content_hasher.finalize()
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl std::fmt::Debug for FinishedDependentValueRootNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FinishedDependentValueNodeWeight")
            .field("id", &self.id.to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("value_id", &self.value_id.to_string())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl CorrectTransforms for FinishedDependentValueRootNodeWeight {}
