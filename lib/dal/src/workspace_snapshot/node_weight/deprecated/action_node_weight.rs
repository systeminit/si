use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid};

use crate::{
    action::ActionState,
    func::FuncExecutionPk,
    workspace_snapshot::{graph::LineageId, vector_clock::deprecated::DeprecatedVectorClock},
    ChangeSetId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedActionNodeWeightLegacy {
    pub id: Ulid,
    pub state: ActionState,
    pub originating_changeset_id: ChangeSetId,
    // DEPRECATED
    pub func_execution_pk: Option<FuncExecutionPk>,
    pub lineage_id: LineageId,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: DeprecatedVectorClock,
    pub vector_clock_recently_seen: DeprecatedVectorClock,
    pub vector_clock_write: DeprecatedVectorClock,
}
