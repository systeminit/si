use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::workspace_snapshot::vector_clock::{HasVectorClocks, VectorClock};
use crate::{ChangeSet, EdgeWeightKindDiscriminants};

use super::NodeWeightResult;

#[derive(Clone, Serialize, Deserialize)]
pub struct DependentValueRootNodeWeight {
    id: Ulid,
    lineage_id: Ulid,
    value_id: Ulid,
    pub touch_count: u16, // unused
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl DependentValueRootNodeWeight {
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

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merge_clocks(&mut self, change_set: &ChangeSet, other: &Self) {
        self.vector_clock_write
            .merge(change_set.vector_clock_id(), other.vector_clock_write());
        self.vector_clock_first_seen.merge(
            change_set.vector_clock_id(),
            other.vector_clock_first_seen(),
        );
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn new(change_set: &ChangeSet, value_id: Ulid) -> NodeWeightResult<Self> {
        Ok(Self {
            id: change_set.generate_ulid()?,
            lineage_id: change_set.generate_ulid()?,
            value_id,
            touch_count: 0,
            vector_clock_write: VectorClock::new(change_set.vector_clock_id()),
            vector_clock_first_seen: VectorClock::new(change_set.vector_clock_id()),
            merkle_tree_hash: Default::default(),
            vector_clock_recently_seen: Default::default(),
        })
    }

    pub fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "value_id": self.value_id,
            "touch_count": self.touch_count,
        }])
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl HasVectorClocks for DependentValueRootNodeWeight {
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

impl std::fmt::Debug for DependentValueRootNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("DependentValueNodeWeight")
            .field("id", &self.id.to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("value_id", &self.value_id.to_string())
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
