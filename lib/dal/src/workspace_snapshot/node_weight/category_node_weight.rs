use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};
use strum::{Display, EnumIter};

use crate::change_set::ChangeSet;
use crate::workspace_snapshot::vector_clock::HasVectorClocks;
use crate::workspace_snapshot::{node_weight::NodeWeightResult, vector_clock::VectorClock};
use crate::EdgeWeightKindDiscriminants;

/// NOTE: adding new categories can be done in a backwards compatible way, so long as we don't
/// assume the new categories already exists on the graph. In places where you need to access the
/// category, check if it exists, and if it doesn't exist, create it (if it makes sense to do so in
/// the given context). Note that a race to create the category will result in a broken graph(since
/// having two of the same category would leave the graph in an inconsistent state), so you should
/// implement the ability to merge your category nodes together.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumIter)]
pub enum CategoryNodeKind {
    Action,
    Component,
    DeprecatedActionBatch,
    Func,
    Module,
    Schema,
    Secret,
    DependentValueRoots,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CategoryNodeWeight {
    id: Ulid,
    lineage_id: Ulid,
    kind: CategoryNodeKind,
    // TODO This should not be a content hash, since it does not point to a value in cas
    content_hash: ContentHash,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl CategoryNodeWeight {
    pub fn content_hash(&self) -> ContentHash {
        self.content_hash
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![]
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn kind(&self) -> CategoryNodeKind {
        self.kind
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn new(change_set: &ChangeSet, kind: CategoryNodeKind) -> NodeWeightResult<Self> {
        Ok(Self {
            id: change_set.generate_ulid()?,
            lineage_id: change_set.generate_ulid()?,
            kind,
            vector_clock_write: VectorClock::new(change_set.vector_clock_id()),
            vector_clock_first_seen: VectorClock::new(change_set.vector_clock_id()),
            content_hash: ContentHash::from(&serde_json::json![kind]),
            merkle_tree_hash: Default::default(),
            vector_clock_recently_seen: Default::default(),
        })
    }

    pub fn node_hash(&self) -> ContentHash {
        self.content_hash()
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl HasVectorClocks for CategoryNodeWeight {
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

impl std::fmt::Debug for CategoryNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("CategoryNodeWeight")
            .field("id", &self.id.to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("content_hash", &self.content_hash)
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
