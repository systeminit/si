use itertools::Itertools;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::workspace_snapshot::{
    change_set::ChangeSet, node_weight::NodeWeightResult, vector_clock::VectorClock,
};
use crate::ContentHash;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderingNodeWeight {
    id: Ulid,
    order: Vec<Ulid>,
    content_hash: ContentHash,
    merkle_tree_hash: ContentHash,
    vector_clock_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl OrderingNodeWeight {
    pub fn content_hash(&self) -> ContentHash {
        self.content_hash
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn increment_seen_vector_clock(&mut self, change_set: &ChangeSet) -> NodeWeightResult<()> {
        self.vector_clock_seen.inc(change_set)?;

        Ok(())
    }

    pub fn increment_vector_clocks(&mut self, change_set: &ChangeSet) -> NodeWeightResult<()> {
        let new_vc_entry = change_set.generate_ulid()?;
        self.vector_clock_write.inc_to(change_set, new_vc_entry);
        self.vector_clock_seen.inc_to(change_set, new_vc_entry);

        Ok(())
    }
    pub fn merge_clocks(
        &mut self,
        change_set: &ChangeSet,
        other: &OrderingNodeWeight,
    ) -> NodeWeightResult<()> {
        self.vector_clock_write
            .merge(change_set, other.vector_clock_write())?;
        self.vector_clock_seen
            .merge(change_set, other.vector_clock_seen())?;

        Ok(())
    }

    pub fn merkle_tree_hash(&self) -> ContentHash {
        self.merkle_tree_hash
    }

    pub fn new_with_incremented_vector_clocks(
        &self,
        change_set: &ChangeSet,
    ) -> NodeWeightResult<Self> {
        let mut new_ordering_weight = self.clone();
        new_ordering_weight.increment_vector_clocks(change_set)?;

        Ok(new_ordering_weight)
    }

    pub fn order(&self) -> &Vec<Ulid> {
        &self.order
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: ContentHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn set_order<'a>(
        &mut self,
        change_set: &ChangeSet,
        order: impl AsRef<&'a [Ulid]>,
    ) -> NodeWeightResult<()> {
        self.order = Vec::from(*order.as_ref());
        self.update_content_hash();
        self.increment_seen_vector_clock(change_set)?;

        Ok(())
    }

    fn update_content_hash(&mut self) {
        let mut content_hasher = ContentHash::hasher();
        let concat_elements = self
            .order
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .concat();
        let content_bytes = concat_elements.as_bytes();
        content_hasher.update(content_bytes);

        self.content_hash = content_hasher.finalize();
    }

    pub fn vector_clock_seen(&self) -> &VectorClock {
        &self.vector_clock_seen
    }

    pub fn vector_clock_write(&self) -> &VectorClock {
        &self.vector_clock_write
    }
}
