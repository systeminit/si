use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};
use strum::Display;

use crate::change_set::ChangeSet;
use crate::workspace_snapshot::vector_clock::VectorClockId;
use crate::workspace_snapshot::{node_weight::NodeWeightResult, vector_clock::VectorClock};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display)]
pub enum CategoryNodeKind {
    ActionBatch,
    Component,
    Func,
    Module,
    Schema,
    Secret,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CategoryNodeWeight {
    id: Ulid,
    lineage_id: Ulid,
    kind: CategoryNodeKind,
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

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn kind(&self) -> CategoryNodeKind {
        self.kind
    }

    pub fn increment_seen_vector_clock(&mut self, change_set: &ChangeSet) -> NodeWeightResult<()> {
        self.vector_clock_first_seen
            .inc(change_set.vector_clock_id())?;

        Ok(())
    }

    pub fn increment_vector_clock(&mut self, change_set: &ChangeSet) -> NodeWeightResult<()> {
        self.vector_clock_write
            .inc(change_set.vector_clock_id())
            .map_err(Into::into)
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

    pub fn merge_clocks(
        &mut self,
        change_set: &ChangeSet,
        other: &CategoryNodeWeight,
    ) -> NodeWeightResult<()> {
        self.vector_clock_write
            .merge(change_set.vector_clock_id(), other.vector_clock_write())?;
        self.vector_clock_first_seen.merge(
            change_set.vector_clock_id(),
            other.vector_clock_first_seen(),
        )?;

        Ok(())
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn new(change_set: &ChangeSet, kind: CategoryNodeKind) -> NodeWeightResult<Self> {
        Ok(Self {
            id: change_set.generate_ulid()?,
            lineage_id: change_set.generate_ulid()?,
            kind,
            vector_clock_write: VectorClock::new(change_set.vector_clock_id())?,
            vector_clock_first_seen: VectorClock::new(change_set.vector_clock_id())?,
            content_hash: ContentHash::from(&serde_json::json![kind]),
            merkle_tree_hash: Default::default(),
            vector_clock_recently_seen: Default::default(),
        })
    }

    pub fn new_with_incremented_vector_clock(
        &self,
        change_set: &ChangeSet,
    ) -> NodeWeightResult<Self> {
        let mut new_ordering_weight = self.clone();
        new_ordering_weight.increment_vector_clock(change_set)?;

        Ok(new_ordering_weight)
    }

    pub fn node_hash(&self) -> ContentHash {
        self.content_hash()
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
