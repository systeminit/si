use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use super::NodeWeightError;
use crate::workspace_snapshot::vector_clock::{HasVectorClocks, VectorClockId};
use crate::workspace_snapshot::{node_weight::NodeWeightResult, vector_clock::VectorClock};
use crate::EdgeWeightKindDiscriminants;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct OrderingNodeWeight {
    pub id: Ulid,
    pub lineage_id: Ulid,
    /// The `id` of the items, in the order that they should appear in the container.
    order: Vec<Ulid>,
    content_hash: ContentHash,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl OrderingNodeWeight {
    pub fn content_hash(&self) -> ContentHash {
        self.content_hash
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![]
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn new(
        id: Ulid,
        lineage_id: Ulid,
        vector_clock_id: VectorClockId,
    ) -> NodeWeightResult<Self> {
        Ok(Self {
            id,
            lineage_id,
            vector_clock_write: VectorClock::new(vector_clock_id),
            vector_clock_first_seen: VectorClock::new(vector_clock_id),
            vector_clock_recently_seen: VectorClock::new(vector_clock_id),
            ..Default::default()
        })
    }

    pub fn node_hash(&self) -> ContentHash {
        self.content_hash()
    }

    pub fn order(&self) -> &Vec<Ulid> {
        &self.order
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn set_order(&mut self, vector_clock_id: VectorClockId, order: Vec<Ulid>) {
        self.set_order_without_inc_clocks(order);
        self.increment_vector_clocks(vector_clock_id);
    }

    fn set_order_without_inc_clocks(&mut self, order: Vec<Ulid>) {
        self.order = order;
        self.update_content_hash();
    }

    fn update_content_hash(&mut self) {
        let mut content_hasher = ContentHash::hasher();
        let concat_elements = self
            .order
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        let content_bytes = concat_elements.as_bytes();
        content_hasher.update(content_bytes);

        self.content_hash = content_hasher.finalize();
    }

    pub fn push_to_order(&mut self, vector_clock_id: VectorClockId, id: Ulid) {
        let mut order = self.order().to_owned();
        order.push(id);
        self.set_order(vector_clock_id, order);
    }

    /// Returns `true` if the id passed was actually removed, `false` if not (because not in the order)
    pub fn remove_from_order(
        &mut self,
        vector_clock_id: VectorClockId,
        id: Ulid,
        inc_clocks: bool,
    ) -> bool {
        let mut order = self.order.to_owned();
        order.retain(|&item_id| item_id != id);
        if order.len() != self.order().len() {
            if inc_clocks {
                self.set_order(vector_clock_id, order);
            } else {
                self.set_order_without_inc_clocks(order);
            }

            true
        } else {
            false
        }
    }
    pub fn get_index_for_id(&self, id: Ulid) -> NodeWeightResult<i64> {
        let index = &self
            .order
            .iter()
            .position(|&key| key == id)
            .ok_or(NodeWeightError::MissingKeytForChildEntry(id))?;

        let ret: i64 = (*index)
            .try_into()
            .map_err(NodeWeightError::TryFromIntError)?;
        Ok(ret)
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl HasVectorClocks for OrderingNodeWeight {
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

impl std::fmt::Debug for OrderingNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("OrderingNodeWeight")
            .field("id", &self.id.to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field(
                "order",
                &self
                    .order
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>(),
            )
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
