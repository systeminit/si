use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{
    action::ActionState,
    workspace_snapshot::{
        graph::LineageId,
        vector_clock::{VectorClock, VectorClockId},
    },
    ChangeSet, ChangeSetId,
};

use super::NodeWeightResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionNodeWeight {
    id: Ulid,
    state: ActionState,
    originating_changeset_id: ChangeSetId,
    lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl ActionNodeWeight {
    pub fn new(change_set: &ChangeSet, id: Ulid) -> NodeWeightResult<Self> {
        let new_vector_clock = VectorClock::new(change_set.vector_clock_id())?;

        Ok(Self {
            id,
            state: ActionState::Queued,
            originating_changeset_id: change_set.id,
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

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn state(&self) -> ActionState {
        self.state
    }

    pub fn originating_changeset_id(&self) -> ChangeSetId {
        self.originating_changeset_id
    }

    pub fn increment_vector_clock(&mut self, change_set: &ChangeSet) -> NodeWeightResult<()> {
        self.vector_clock_write.inc(change_set.vector_clock_id())?;
        self.vector_clock_recently_seen
            .inc(change_set.vector_clock_id())?;

        Ok(())
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn mark_seen_at(&mut self, vector_clock_id: VectorClockId, seen_at: DateTime<Utc>) {
        self.vector_clock_recently_seen
            .inc_to(vector_clock_id, seen_at);
        if self
            .vector_clock_first_seen
            .entry_for(vector_clock_id)
            .is_none()
        {
            self.vector_clock_first_seen
                .inc_to(vector_clock_id, seen_at);
        }
    }

    pub fn merge_clocks(&mut self, change_set: &ChangeSet, other: &Self) -> NodeWeightResult<()> {
        self.vector_clock_write
            .merge(change_set.vector_clock_id(), &other.vector_clock_write)?;
        self.vector_clock_first_seen
            .merge(change_set.vector_clock_id(), &other.vector_clock_first_seen)?;
        self.vector_clock_recently_seen.merge(
            change_set.vector_clock_id(),
            &other.vector_clock_recently_seen,
        )?;

        Ok(())
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn new_with_incremented_vector_clock(
        &self,
        change_set: &ChangeSet,
    ) -> NodeWeightResult<Self> {
        let mut new_node_weight = self.clone();
        new_node_weight.increment_vector_clock(change_set)?;

        Ok(new_node_weight)
    }

    pub fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "id": self.id,
            "lineage_id": self.lineage_id,
        }])
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn set_vector_clock_recently_seen_to(
        &mut self,
        change_set: &ChangeSet,
        new_val: DateTime<Utc>,
    ) {
        self.vector_clock_recently_seen
            .inc_to(change_set.vector_clock_id(), new_val);
    }

    pub fn vector_clock_first_seen(&self) -> &VectorClock {
        &self.vector_clock_first_seen
    }

    pub fn vector_clock_recently_seen(&self) -> &VectorClock {
        &self.vector_clock_recently_seen
    }

    pub fn vector_clock_write(&self) -> &VectorClock {
        &self.vector_clock_write
    }
}
