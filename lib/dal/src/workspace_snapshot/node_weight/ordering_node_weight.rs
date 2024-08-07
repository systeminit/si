use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use super::NodeWeightError;
use crate::workspace_snapshot::graph::deprecated::v1::DeprecatedOrderingNodeWeightV1;
use crate::workspace_snapshot::node_weight::NodeWeightResult;
use crate::EdgeWeightKindDiscriminants;

#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct OrderingNodeWeight {
    pub id: Ulid,
    pub lineage_id: Ulid,
    /// The `id` of the items, in the order that they should appear in the container.
    order: Vec<Ulid>,
    merkle_tree_hash: MerkleTreeHash,
}

impl OrderingNodeWeight {
    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
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

    pub fn new(id: Ulid, lineage_id: Ulid) -> Self {
        Self {
            id,
            lineage_id,
            ..Default::default()
        }
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        for id in &self.order {
            let bytes = id.inner().to_bytes();
            content_hasher.update(&bytes);
        }

        content_hasher.finalize()
    }

    pub fn order(&self) -> &Vec<Ulid> {
        &self.order
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn set_order(&mut self, order: Vec<Ulid>) {
        self.order = order;
    }

    pub fn push_to_order(&mut self, id: Ulid) {
        self.order.push(id);
    }

    /// Returns `true` if the id passed was actually removed, `false` if not (because not in the order)
    pub fn remove_from_order(&mut self, id: Ulid) -> bool {
        let order_len = self.order.len();
        self.order.retain(|&item_id| item_id != id);
        order_len != self.order().len()
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
            .field("content_hash", &self.content_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl From<DeprecatedOrderingNodeWeightV1> for OrderingNodeWeight {
    fn from(value: DeprecatedOrderingNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            order: value.order,
            merkle_tree_hash: value.merkle_tree_hash,
        }
    }
}
