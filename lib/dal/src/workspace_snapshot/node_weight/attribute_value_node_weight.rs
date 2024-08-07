use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::{deprecated::v1::DeprecatedAttributeValueNodeWeightV1, LineageId},
    },
    EdgeWeightKindDiscriminants,
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttributeValueNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    /// The unprocessed return value is the "real" result, unprocessed for any other behavior.
    /// This is potentially-maybe-only-kinda-sort-of(?) useful for non-scalar values.
    /// Example: a populated array.
    unprocessed_value: Option<ContentAddress>,
    /// The processed return value.
    /// Example: empty array.
    value: Option<ContentAddress>,
}

impl AttributeValueNodeWeight {
    pub fn new(
        id: Ulid,
        lineage_id: Ulid,
        unprocessed_value: Option<ContentAddress>,
        value: Option<ContentAddress>,
    ) -> Self {
        Self {
            id,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
            unprocessed_value,
            value,
        }
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        let mut hashes = vec![];

        if let Some(hash) = self.unprocessed_value {
            hashes.push(hash.content_hash());
        }
        if let Some(hash) = self.value {
            hashes.push(hash.content_hash());
        }

        hashes
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn unprocessed_value(&self) -> Option<ContentAddress> {
        self.unprocessed_value
    }

    pub fn set_unprocessed_value(&mut self, unprocessed_value: Option<ContentAddress>) {
        self.unprocessed_value = unprocessed_value
    }

    pub fn value(&self) -> Option<ContentAddress> {
        self.value
    }

    pub fn set_value(&mut self, value: Option<ContentAddress>) {
        self.value = value
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
    }

    pub fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "unprocessed_value": self.unprocessed_value,
            "value": self.value,
        }])
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[
            EdgeWeightKindDiscriminants::Prototype,
            EdgeWeightKindDiscriminants::Prop,
            EdgeWeightKindDiscriminants::Socket,
        ]
    }
}

impl std::fmt::Debug for AttributeValueNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("AttributeValueNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("value", &self.value)
            .field("unprocessed_value", &self.unprocessed_value)
            .field("node_hash", &self.node_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl From<DeprecatedAttributeValueNodeWeightV1> for AttributeValueNodeWeight {
    fn from(value: DeprecatedAttributeValueNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            merkle_tree_hash: value.merkle_tree_hash,
            unprocessed_value: value.unprocessed_value,
            value: value.value,
        }
    }
}
