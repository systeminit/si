use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash, VectorClockId};

use crate::{
    workspace_snapshot::{
        content_address::{ContentAddress, ContentAddressDiscriminants},
        graph::LineageId,
        vector_clock::{HasVectorClocks, VectorClock},
    },
    EdgeWeightKindDiscriminants,
};

use super::{deprecated::DeprecatedComponentNodeWeight, NodeWeightError, NodeWeightResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
    to_delete: bool,
}

impl ComponentNodeWeight {
    pub fn new(
        vector_clock_id: VectorClockId,
        id: Ulid,
        lineage_id: Ulid,
        content_address: ContentAddress,
    ) -> NodeWeightResult<Self> {
        let new_vector_clock = VectorClock::new(vector_clock_id);

        Ok(Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            to_delete: false,
            vector_clock_first_seen: new_vector_clock.clone(),
            vector_clock_recently_seen: new_vector_clock.clone(),
            vector_clock_write: new_vector_clock,
        })
    }

    pub fn content_address(&self) -> ContentAddress {
        self.content_address
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![self.content_address.content_hash()]
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_address.content_hash()
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

    pub fn to_delete(&self) -> bool {
        self.to_delete
    }

    pub fn set_to_delete(&mut self, to_delete: bool) -> &mut Self {
        self.to_delete = to_delete;
        self
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::Component(_) => ContentAddress::Component(content_hash),
            other => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    Into::<ContentAddressDiscriminants>::into(other).to_string(),
                    ContentAddressDiscriminants::Component.to_string(),
                ));
            }
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "content_address": self.content_address,
            "to_delete": self.to_delete,
        }])
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn overwrite_id(&mut self, id: Ulid) {
        self.id = id
    }

    pub fn overwrite_lineage_id(&mut self, id: LineageId) {
        self.lineage_id = id
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[
            EdgeWeightKindDiscriminants::Use,
            EdgeWeightKindDiscriminants::Root,
        ]
    }
}

impl HasVectorClocks for ComponentNodeWeight {
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

impl From<DeprecatedComponentNodeWeight> for ComponentNodeWeight {
    fn from(value: DeprecatedComponentNodeWeight) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            content_address: value.content_address,
            merkle_tree_hash: value.merkle_tree_hash,
            vector_clock_first_seen: VectorClock::empty(),
            vector_clock_recently_seen: VectorClock::empty(),
            vector_clock_write: VectorClock::empty(),
            to_delete: value.to_delete,
        }
    }
}
