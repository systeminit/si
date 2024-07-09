use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::workspace_snapshot::graph::LineageId;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::vector_clock::deprecated::DeprecatedVectorClock;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeprecatedCategoryNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    kind: CategoryNodeKind,
    // TODO This should not be a content hash, since it does not point to a value in cas
    content_hash: ContentHash,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: DeprecatedVectorClock,
    vector_clock_recently_seen: DeprecatedVectorClock,
    vector_clock_write: DeprecatedVectorClock,
}
