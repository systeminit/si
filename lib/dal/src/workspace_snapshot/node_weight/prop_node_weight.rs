use chrono::{DateTime, Utc};
use content_store::ContentHash;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::vector_clock::VectorClockId;
use crate::{
    change_set_pointer::ChangeSetPointer,
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        node_weight::{NodeWeightError, NodeWeightResult},
        vector_clock::VectorClock,
    },
    PropKind,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct PropNodeWeight {
    id: Ulid,
    lineage_id: LineageId,
    content_address: ContentAddress,
    merkle_tree_hash: ContentHash,
    kind: PropKind,
    name: String,
    can_be_used_as_prototype_arg: bool,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl PropNodeWeight {
    pub fn new(
        change_set: &ChangeSetPointer,
        id: Ulid,
        content_address: ContentAddress,
        kind: PropKind,
        name: String,
    ) -> NodeWeightResult<Self> {
        Ok(Self {
            id,
            lineage_id: change_set.generate_ulid()?,
            content_address,
            merkle_tree_hash: ContentHash::default(),
            kind,
            name,
            can_be_used_as_prototype_arg: false,
            vector_clock_first_seen: VectorClock::new(change_set.vector_clock_id())?,
            vector_clock_recently_seen: VectorClock::new(change_set.vector_clock_id())?,
            vector_clock_write: VectorClock::new(change_set.vector_clock_id())?,
        })
    }

    pub fn kind(&self) -> PropKind {
        self.kind
    }

    pub fn content_address(&self) -> ContentAddress {
        self.content_address
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_address.content_hash()
    }

    pub fn can_be_used_as_prototype_arg(&self) -> bool {
        self.can_be_used_as_prototype_arg
    }

    pub fn set_can_be_used_as_prototype_arg(&mut self, can_be_used: bool) {
        self.can_be_used_as_prototype_arg = can_be_used;
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn increment_vector_clock(
        &mut self,
        change_set: &ChangeSetPointer,
    ) -> NodeWeightResult<()> {
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

    pub fn merge_clocks(
        &mut self,
        change_set: &ChangeSetPointer,
        other: &Self,
    ) -> NodeWeightResult<()> {
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::Prop(_) => ContentAddress::Prop(content_hash),
            other => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    Into::<ContentAddressDiscriminants>::into(other).to_string(),
                    ContentAddressDiscriminants::Prop.to_string(),
                ));
            }
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn new_with_incremented_vector_clock(
        &self,
        change_set: &ChangeSetPointer,
    ) -> NodeWeightResult<Self> {
        let mut new_node_weight = self.clone();
        new_node_weight.increment_vector_clock(change_set)?;

        Ok(new_node_weight)
    }

    pub fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "content_address": self.content_address,
            "kind": self.kind,
            "name": self.name,
        }])
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: ContentHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn set_vector_clock_recently_seen_to(
        &mut self,
        change_set: &ChangeSetPointer,
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

impl std::fmt::Debug for PropNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PropNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("kind", &self.kind)
            .field("name", &self.name)
            .field("content_hash", &self.content_hash())
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
