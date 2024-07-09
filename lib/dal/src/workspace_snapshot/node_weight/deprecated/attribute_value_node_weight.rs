use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid};

use crate::{
    func::FuncExecutionPk,
    workspace_snapshot::{
        content_address::ContentAddress, graph::LineageId,
        vector_clock::deprecated::DeprecatedVectorClock,
    },
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeprecatedAttributeValueNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: DeprecatedVectorClock,
    vector_clock_recently_seen: DeprecatedVectorClock,
    vector_clock_write: DeprecatedVectorClock,
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
