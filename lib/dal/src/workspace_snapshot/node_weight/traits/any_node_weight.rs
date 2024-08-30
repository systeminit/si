use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};
use crate::workspace_snapshot::{graph::LineageId, node_weight::traits::CorrectTransforms};

///
/// Everything supported by all node weights (including the top-level NodeWeight).
/// 
pub trait AnyNodeWeight: CorrectTransforms+NodeHash
    +Sized+Clone+PartialEq+Eq
    +std::fmt::Debug+serde::Serialize
    where for<'de> Self: serde::Deserialize<'de>,
{
    /// The stable ID for this node.
    fn id(&self) -> Ulid;
    fn lineage_id(&self) -> Ulid;
    fn set_id_and_lineage(&mut self, id: impl Into<Ulid>, lineage_id: LineageId);

    fn merkle_tree_hash(&self) -> MerkleTreeHash;
    fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash);
}

pub trait NodeHash {
    /// The node hash is used to compare nodes directly, and should be computed based on the data
    /// that is specific to the node weight, *and* the content_hash, so that changes are detected
    /// between nodes whether the content has changed or just the node weight has changed.
    fn node_hash(&self) -> ContentHash;
}
