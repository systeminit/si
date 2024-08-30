use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};
use strum::{Display, EnumIter};

use crate::workspace_snapshot::graph::deprecated::v1::DeprecatedCategoryNodeWeightV1;
use crate::workspace_snapshot::graph::LineageId;
use crate::workspace_snapshot::node_weight::traits::CorrectTransforms;
use crate::EdgeWeightKindDiscriminants;

use super::NodeHash;

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

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategoryNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    kind: CategoryNodeKind,
    pub(super) merkle_tree_hash: MerkleTreeHash,
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

    pub fn new(id: Ulid, lineage_id: Ulid, kind: CategoryNodeKind) -> Self {
        Self {
            id,
            lineage_id,
            kind,
            merkle_tree_hash: Default::default(),
        }
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl NodeHash for CategoryNodeWeight {
    fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![self.kind])
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
