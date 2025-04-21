use serde::{Deserialize, Serialize};
use si_events::{ContentHash, merkle_tree_hash::MerkleTreeHash, ulid::Ulid};
use strum::{Display, EnumIter};

use crate::EdgeWeightKindDiscriminants;
use crate::workspace_snapshot::graph::LineageId;
use crate::workspace_snapshot::graph::deprecated::v1::DeprecatedCategoryNodeWeightV1;
use crate::workspace_snapshot::node_weight::traits::CorrectTransforms;

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
    View,
    DiagramObject,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategoryNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    kind: CategoryNodeKind,
    merkle_tree_hash: MerkleTreeHash,
}

impl CategoryNodeWeight {
    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
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

    pub fn new(id: Ulid, lineage_id: Ulid, kind: CategoryNodeKind) -> Self {
        Self {
            id,
            lineage_id,
            kind,
            merkle_tree_hash: Default::default(),
        }
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(self.kind.to_string().as_bytes());

        content_hasher.finalize()
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl std::fmt::Debug for CategoryNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("CategoryNodeWeight")
            .field("id", &self.id.to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl From<DeprecatedCategoryNodeWeightV1> for CategoryNodeWeight {
    fn from(value: DeprecatedCategoryNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            kind: value.kind,
            merkle_tree_hash: value.merkle_tree_hash,
        }
    }
}

impl CorrectTransforms for CategoryNodeWeight {}
