use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash, VectorClockId};

use crate::{
    workspace_snapshot::{
        content_address::{ContentAddress, ContentAddressDiscriminants},
        graph::LineageId,
        node_weight::NodeWeightResult,
        vector_clock::{HasVectorClocks, VectorClock},
        NodeWeightError,
    },
    EdgeWeightKindDiscriminants,
};

use super::deprecated::DeprecatedFuncArgumentNodeWeight;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FuncArgumentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
    name: String,
}

impl FuncArgumentNodeWeight {
    pub fn new(
        vector_clock_id: VectorClockId,
        id: Ulid,
        lineage_id: Ulid,
        content_address: ContentAddress,
        name: String,
    ) -> NodeWeightResult<Self> {
        Ok(Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            name,
            vector_clock_first_seen: VectorClock::new(vector_clock_id),
            vector_clock_recently_seen: VectorClock::new(vector_clock_id),
            vector_clock_write: VectorClock::new(vector_clock_id),
        })
    }

    pub fn content_address(&self) -> ContentAddress {
        self.content_address
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_address.content_hash()
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![self.content_address.content_hash()]
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::FuncArg(_) => ContentAddress::FuncArg(content_hash),
            other => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    Into::<ContentAddressDiscriminants>::into(other).to_string(),
                    ContentAddressDiscriminants::FuncArg.to_string(),
                ));
            }
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "content_address": self.content_address,
            "name": self.name,
        }])
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl HasVectorClocks for FuncArgumentNodeWeight {
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

impl std::fmt::Debug for FuncArgumentNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FuncNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
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

impl From<DeprecatedFuncArgumentNodeWeight> for FuncArgumentNodeWeight {
    fn from(value: DeprecatedFuncArgumentNodeWeight) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            content_address: value.content_address,
            merkle_tree_hash: value.merkle_tree_hash,
            vector_clock_first_seen: VectorClock::empty(),
            vector_clock_recently_seen: VectorClock::empty(),
            vector_clock_write: VectorClock::empty(),
            name: value.name,
        }
    }
}
