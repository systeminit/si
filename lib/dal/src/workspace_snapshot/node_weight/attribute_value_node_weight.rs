use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{
    change_set::ChangeSet,
    func::FuncExecutionPk,
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        node_weight::NodeWeightResult,
        vector_clock::{HasVectorClocks, VectorClock},
    },
    EdgeWeightKindDiscriminants,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct AttributeValueNodeWeight {
    id: Ulid,
    lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
    /// The unprocessed return value is the "real" result, unprocessed for any other behavior.
    /// This is potentially-maybe-only-kinda-sort-of(?) useful for non-scalar values.
    /// Example: a populated array.
    unprocessed_value: Option<ContentAddress>,
    /// The processed return value.
    /// Example: empty array.
    value: Option<ContentAddress>,
    // DEPRECATED - this was the old function execution system
    func_execution_pk: Option<FuncExecutionPk>,
}

impl AttributeValueNodeWeight {
    pub fn new(
        change_set: &ChangeSet,
        id: Ulid,
        unprocessed_value: Option<ContentAddress>,
        value: Option<ContentAddress>,
    ) -> NodeWeightResult<Self> {
        Ok(Self {
            id,
            lineage_id: change_set.generate_ulid()?,
            merkle_tree_hash: MerkleTreeHash::default(),
            vector_clock_first_seen: VectorClock::new(change_set.vector_clock_id()),
            vector_clock_recently_seen: VectorClock::new(change_set.vector_clock_id()),
            vector_clock_write: VectorClock::new(change_set.vector_clock_id()),
            unprocessed_value,
            value,
            func_execution_pk: None,
        })
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

    #[deprecated(note = "we no longer use func execution pks")]
    pub fn set_func_execution_pk(&mut self, func_execution_pk: Option<FuncExecutionPk>) {
        self.func_execution_pk = func_execution_pk
    }

    #[deprecated(note = "we no longer use func execution pks")]
    pub fn func_execution_pk(&self) -> Option<FuncExecutionPk> {
        self.func_execution_pk
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

impl HasVectorClocks for AttributeValueNodeWeight {
    fn vector_clock_first_seen(&self) -> &VectorClock {
        &self.vector_clock_first_seen
    }

    fn vector_clock_recently_seen(&self) -> &VectorClock {
        &self.vector_clock_recently_seen
    }

    fn vector_clock_write(&self) -> &VectorClock {
        &self.vector_clock_write
    }

    fn vector_clock_first_seen_mut(&mut self) -> &mut VectorClock {
        &mut self.vector_clock_first_seen
    }

    fn vector_clock_recently_seen_mut(&mut self) -> &mut VectorClock {
        &mut self.vector_clock_recently_seen
    }

    fn vector_clock_write_mut(&mut self) -> &mut VectorClock {
        &mut self.vector_clock_write
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
            .field("vector_clock_first_seen", &self.vector_clock_first_seen)
            .field(
                "vector_clock_recently_seen",
                &self.vector_clock_recently_seen,
            )
            .field("vector_clock_write", &self.vector_clock_write)
            .finish()
    }
}
