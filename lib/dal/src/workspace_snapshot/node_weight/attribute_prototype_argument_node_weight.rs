use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_events::ContentHash;
use ulid::Ulid;

use crate::{
    change_set::ChangeSet,
    workspace_snapshot::{
        graph::LineageId, node_weight::NodeWeightResult, vector_clock::VectorClock,
    },
    ComponentId, Timestamp,
};

use crate::workspace_snapshot::vector_clock::VectorClockId;

/// When this `AttributePrototypeArgument` represents a connection between two
/// components, we need to know which components are being connected.
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ArgumentTargets {
    pub source_component_id: ComponentId,
    pub destination_component_id: ComponentId,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AttributePrototypeArgumentNodeWeight {
    id: Ulid,
    lineage_id: LineageId,
    merkle_tree_hash: ContentHash,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
    targets: Option<ArgumentTargets>,
    timestamp: Timestamp,
}

impl AttributePrototypeArgumentNodeWeight {
    pub fn new(
        change_set: &ChangeSet,
        id: Ulid,
        targets: Option<ArgumentTargets>,
    ) -> NodeWeightResult<Self> {
        Ok(Self {
            id,
            lineage_id: change_set.generate_ulid()?,
            merkle_tree_hash: ContentHash::default(),
            targets,
            vector_clock_first_seen: VectorClock::new(change_set.vector_clock_id())?,
            vector_clock_recently_seen: VectorClock::new(change_set.vector_clock_id())?,
            vector_clock_write: VectorClock::new(change_set.vector_clock_id())?,
            timestamp: Timestamp::now(),
        })
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub fn content_hash(&self) -> ContentHash {
        let target_string = self
            .targets
            .map(|targets| {
                format!(
                    "{}{}",
                    targets.source_component_id, targets.destination_component_id
                )
            })
            .unwrap_or("".into());

        ContentHash::new(target_string.as_bytes())
    }

    pub fn node_hash(&self) -> ContentHash {
        self.content_hash()
    }

    pub fn id(&self) -> Ulid {
        self.id
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

    pub fn merkle_tree_hash(&self) -> ContentHash {
        self.merkle_tree_hash
    }

    pub fn targets(&self) -> Option<ArgumentTargets> {
        self.targets
    }

    pub fn new_with_incremented_vector_clock(
        &self,
        change_set: &ChangeSet,
    ) -> NodeWeightResult<Self> {
        let mut new_node_weight = self.clone();
        new_node_weight.increment_vector_clock(change_set)?;

        Ok(new_node_weight)
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: ContentHash) {
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

impl std::fmt::Debug for AttributePrototypeArgumentNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("AttributePrototypeArgumentNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("targets", &self.targets)
            .field("node_hash", &self.node_hash())
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
