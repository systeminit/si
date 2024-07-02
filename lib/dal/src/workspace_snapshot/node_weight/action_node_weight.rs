use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{
    action::ActionState,
    func::FuncExecutionPk,
    workspace_snapshot::{
        graph::LineageId,
        vector_clock::{HasVectorClocks, VectorClock},
    },
    ChangeSet, ChangeSetId, EdgeWeightKindDiscriminants,
};

use super::NodeWeightResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionNodeWeight {
    pub id: Ulid,
    state: ActionState,
    originating_changeset_id: ChangeSetId,
    // DEPRECATED
    func_execution_pk: Option<FuncExecutionPk>,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl ActionNodeWeight {
    pub fn new(
        change_set: &ChangeSet,
        originating_change_set_id: ChangeSetId,
        id: Ulid,
    ) -> NodeWeightResult<Self> {
        let new_vector_clock = VectorClock::new(change_set.vector_clock_id());

        Ok(Self {
            id,
            state: ActionState::Queued,
            func_execution_pk: None,
            originating_changeset_id: originating_change_set_id,
            lineage_id: change_set.generate_ulid()?,
            merkle_tree_hash: MerkleTreeHash::default(),
            vector_clock_first_seen: new_vector_clock.clone(),
            vector_clock_recently_seen: new_vector_clock.clone(),
            vector_clock_write: new_vector_clock,
        })
    }

    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![]
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn set_state(&mut self, state: ActionState) {
        self.state = state;
    }

    pub fn state(&self) -> ActionState {
        self.state
    }

    pub fn originating_changeset_id(&self) -> ChangeSetId {
        self.originating_changeset_id
    }

    #[deprecated(note = "use set_function_run_id instead")]
    pub fn set_func_execution_pk(&mut self, func_execution_pk: Option<FuncExecutionPk>) {
        self.func_execution_pk = func_execution_pk
    }

    #[deprecated(note = "use function_run_id instead")]
    pub fn func_execution_pk(&self) -> Option<FuncExecutionPk> {
        self.func_execution_pk
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "id": self.id,
            "lineage_id": self.lineage_id,
            "state": self.state,
            "originating_changeset_id": self.originating_changeset_id,
            "func_execution_pk": self.func_execution_pk,
        }])
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[EdgeWeightKindDiscriminants::Use]
    }
}

impl HasVectorClocks for ActionNodeWeight {
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
